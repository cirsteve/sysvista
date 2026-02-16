use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

use crate::output::schema::{
    ComponentKind, DetectedComponent, DetectedEdge, StepType, Workflow, WorkflowStep,
};

/// Infer workflows from components and edges.
/// For each transport component, build a workflow by following edges:
/// 1. Transport is the entry point (Entry)
/// 2. Follow `calls` edges → Call steps
/// 3. From call targets, follow `persists`/`transforms` edges → Persist steps
/// 4. Follow `dispatches` edges → Dispatch steps
/// 5. Match transport's `produces` list to model components → Response steps
/// Skip workflows with only 1 step.
pub fn infer_workflows(
    components: &[DetectedComponent],
    edges: &[DetectedEdge],
) -> Vec<Workflow> {
    // Build adjacency by edge label: from_id → [(to_id, label)]
    let mut outgoing: HashMap<&str, Vec<(&str, &str)>> = HashMap::new();
    for edge in edges {
        if let Some(label) = &edge.label {
            outgoing
                .entry(edge.from_id.as_str())
                .or_default()
                .push((edge.to_id.as_str(), label.as_str()));
        }
    }

    // Build model name→id map for produces matching
    let model_name_to_id: HashMap<&str, &str> = components
        .iter()
        .filter(|c| c.kind == ComponentKind::Model)
        .map(|c| (c.name.as_str(), c.id.as_str()))
        .collect();

    let mut workflows = Vec::new();

    for comp in components {
        if comp.kind != ComponentKind::Transport {
            continue;
        }

        let mut steps: Vec<WorkflowStep> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();

        // Step 0: Entry (the transport itself)
        steps.push(WorkflowStep {
            component_id: comp.id.clone(),
            step_type: StepType::Entry,
            order: 0,
        });
        seen.insert(comp.id.clone());

        let mut order = 1u32;

        // Step 1: Follow `calls` edges from transport
        let call_targets: Vec<&str> = outgoing
            .get(comp.id.as_str())
            .map(|edges| {
                edges
                    .iter()
                    .filter(|(_, label)| *label == "calls")
                    .map(|(to, _)| *to)
                    .collect()
            })
            .unwrap_or_default();

        for target_id in &call_targets {
            if seen.insert(target_id.to_string()) {
                steps.push(WorkflowStep {
                    component_id: target_id.to_string(),
                    step_type: StepType::Call,
                    order,
                });
                order += 1;
            }
        }

        // Step 2: From call targets, follow persists/transforms edges
        for target_id in &call_targets {
            if let Some(target_edges) = outgoing.get(*target_id) {
                for (to_id, label) in target_edges {
                    if (*label == "persists" || *label == "transforms") && seen.insert(to_id.to_string()) {
                        steps.push(WorkflowStep {
                            component_id: to_id.to_string(),
                            step_type: StepType::Persist,
                            order,
                        });
                        order += 1;
                    }
                }
            }
        }

        // Also check transport's own persists/transforms edges
        if let Some(tp_edges) = outgoing.get(comp.id.as_str()) {
            for (to_id, label) in tp_edges {
                if (*label == "persists" || *label == "transforms") && seen.insert(to_id.to_string()) {
                    steps.push(WorkflowStep {
                        component_id: to_id.to_string(),
                        step_type: StepType::Persist,
                        order,
                    });
                    order += 1;
                }
            }
        }

        // Step 3: Follow `dispatches` edges from transport
        if let Some(tp_edges) = outgoing.get(comp.id.as_str()) {
            for (to_id, label) in tp_edges {
                if *label == "dispatches" && seen.insert(to_id.to_string()) {
                    steps.push(WorkflowStep {
                        component_id: to_id.to_string(),
                        step_type: StepType::Dispatch,
                        order,
                    });
                    order += 1;
                }
            }
        }

        // Step 4: Match produces to model components → Response steps
        if let Some(ref produces) = comp.produces {
            for type_name in produces {
                if let Some(&model_id) = model_name_to_id.get(type_name.as_str()) {
                    if seen.insert(model_id.to_string()) {
                        steps.push(WorkflowStep {
                            component_id: model_id.to_string(),
                            step_type: StepType::Response,
                            order,
                        });
                        order += 1;
                    }
                }
            }
        }

        // Skip trivial workflows (only the entry point)
        if steps.len() <= 1 {
            continue;
        }

        // Generate workflow name from transport
        let name = if let (Some(method), Some(path)) = (&comp.http_method, &comp.http_path) {
            format!("{} {}", method, path)
        } else {
            comp.name.clone()
        };

        // Generate deterministic ID
        let mut hasher = Sha256::new();
        hasher.update(format!("workflow:{}", comp.id));
        let hash = format!("{:x}", hasher.finalize());
        let id = hash[..16].to_string();

        workflows.push(Workflow {
            id,
            name,
            entry_point_id: comp.id.clone(),
            steps,
        });
    }

    // Sort by step count descending (most interesting workflows first)
    workflows.sort_by(|a, b| b.steps.len().cmp(&a.steps.len()));

    workflows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::schema::SourceLocation;

    fn make_comp(id: &str, name: &str, kind: ComponentKind, produces: Option<Vec<String>>) -> DetectedComponent {
        DetectedComponent {
            id: id.to_string(),
            name: name.to_string(),
            kind,
            language: "python".to_string(),
            source: SourceLocation { file: "test.py".to_string(), line_start: Some(1), line_end: None },
            metadata: std::collections::HashMap::new(),
            transport_protocol: None,
            http_method: Some("POST".to_string()),
            http_path: Some("/messages".to_string()),
            model_fields: None,
            consumes: None,
            produces,
        }
    }

    fn make_edge(from: &str, to: &str, label: &str) -> DetectedEdge {
        DetectedEdge {
            from_id: from.to_string(),
            to_id: to.to_string(),
            label: Some(label.to_string()),
            payload_type: None,
        }
    }

    #[test]
    fn builds_workflow_from_transport_with_calls_and_persists() {
        let components = vec![
            make_comp("tp1", "create_route", ComponentKind::Transport, None),
            make_comp("svc1", "create_message", ComponentKind::Service, None),
            make_comp("m1", "Message", ComponentKind::Model, None),
        ];
        let edges = vec![
            make_edge("tp1", "svc1", "calls"),
            make_edge("svc1", "m1", "persists"),
        ];

        let workflows = infer_workflows(&components, &edges);
        assert_eq!(workflows.len(), 1);

        let wf = &workflows[0];
        assert_eq!(wf.name, "POST /messages");
        assert_eq!(wf.entry_point_id, "tp1");
        assert_eq!(wf.steps.len(), 3);
        assert_eq!(wf.steps[0].step_type, StepType::Entry);
        assert_eq!(wf.steps[0].component_id, "tp1");
        assert_eq!(wf.steps[1].step_type, StepType::Call);
        assert_eq!(wf.steps[1].component_id, "svc1");
        assert_eq!(wf.steps[2].step_type, StepType::Persist);
        assert_eq!(wf.steps[2].component_id, "m1");
    }

    #[test]
    fn includes_dispatch_steps() {
        let components = vec![
            make_comp("tp1", "create_route", ComponentKind::Transport, None),
            make_comp("w1", "enqueue", ComponentKind::Service, None),
        ];
        let edges = vec![
            make_edge("tp1", "w1", "dispatches"),
        ];

        let workflows = infer_workflows(&components, &edges);
        assert_eq!(workflows.len(), 1);
        assert_eq!(workflows[0].steps.len(), 2);
        assert_eq!(workflows[0].steps[1].step_type, StepType::Dispatch);
    }

    #[test]
    fn includes_response_steps_from_produces() {
        let components = vec![
            make_comp("tp1", "get_route", ComponentKind::Transport, Some(vec!["Message".to_string()])),
            make_comp("m1", "Message", ComponentKind::Model, None),
        ];
        let edges = vec![
            make_edge("tp1", "m1", "produces"),
        ];

        let workflows = infer_workflows(&components, &edges);
        assert_eq!(workflows.len(), 1);
        assert_eq!(workflows[0].steps.len(), 2);
        assert_eq!(workflows[0].steps[1].step_type, StepType::Response);
    }

    #[test]
    fn skips_trivial_single_step_workflows() {
        let components = vec![
            make_comp("tp1", "health_check", ComponentKind::Transport, None),
        ];
        let edges: Vec<DetectedEdge> = vec![];

        let workflows = infer_workflows(&components, &edges);
        assert!(workflows.is_empty());
    }

    #[test]
    fn ignores_non_transport_components() {
        let components = vec![
            make_comp("svc1", "helper", ComponentKind::Service, None),
            make_comp("m1", "Model", ComponentKind::Model, None),
        ];
        let edges = vec![
            make_edge("svc1", "m1", "persists"),
        ];

        let workflows = infer_workflows(&components, &edges);
        assert!(workflows.is_empty());
    }

    #[test]
    fn deterministic_workflow_id() {
        let components = vec![
            make_comp("tp1", "route", ComponentKind::Transport, None),
            make_comp("svc1", "handler", ComponentKind::Service, None),
        ];
        let edges = vec![make_edge("tp1", "svc1", "calls")];

        let wf1 = infer_workflows(&components, &edges);
        let wf2 = infer_workflows(&components, &edges);
        assert_eq!(wf1[0].id, wf2[0].id);
    }

    #[test]
    fn sorted_by_step_count_descending() {
        let components = vec![
            make_comp("tp1", "small_route", ComponentKind::Transport, None),
            make_comp("tp2", "big_route", ComponentKind::Transport, None),
            make_comp("svc1", "svc_a", ComponentKind::Service, None),
            make_comp("svc2", "svc_b", ComponentKind::Service, None),
            make_comp("m1", "Model", ComponentKind::Model, None),
        ];
        let edges = vec![
            make_edge("tp1", "svc1", "calls"),
            make_edge("tp2", "svc2", "calls"),
            make_edge("svc2", "m1", "persists"),
        ];

        let workflows = infer_workflows(&components, &edges);
        assert_eq!(workflows.len(), 2);
        assert!(workflows[0].steps.len() >= workflows[1].steps.len());
    }

    #[test]
    fn step_ordering_is_sequential() {
        let components = vec![
            make_comp("tp1", "route", ComponentKind::Transport, Some(vec!["Resp".to_string()])),
            make_comp("svc1", "handler", ComponentKind::Service, None),
            make_comp("m1", "Resp", ComponentKind::Model, None),
            make_comp("w1", "worker", ComponentKind::Service, None),
        ];
        let edges = vec![
            make_edge("tp1", "svc1", "calls"),
            make_edge("tp1", "w1", "dispatches"),
        ];

        let workflows = infer_workflows(&components, &edges);
        assert_eq!(workflows.len(), 1);
        let orders: Vec<u32> = workflows[0].steps.iter().map(|s| s.order).collect();
        // Check orders are sequential starting from 0
        for (i, &order) in orders.iter().enumerate() {
            assert_eq!(order, i as u32);
        }
    }
}
