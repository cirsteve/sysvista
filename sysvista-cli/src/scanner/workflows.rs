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
