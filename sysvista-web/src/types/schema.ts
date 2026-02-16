export type ComponentKind = "model" | "service" | "transport" | "transform";
export type TransportProtocol = "http" | "grpc" | "websocket" | "mq" | "graphql" | "unknown";

export interface SourceLocation {
  file: string;
  line_start?: number;
  line_end?: number;
}

export interface DetectedComponent {
  id: string;
  name: string;
  kind: ComponentKind;
  language: string;
  source: SourceLocation;
  metadata: Record<string, string>;
  transport_protocol?: TransportProtocol;
  http_method?: string;
  http_path?: string;
  model_fields?: string[];
  consumes?: string[];
  produces?: string[];
}

export interface DetectedEdge {
  from_id: string;
  to_id: string;
  label?: string;
  payload_type?: string;
}

export type StepType = "entry" | "call" | "persist" | "dispatch" | "response";

export interface WorkflowStep {
  component_id: string;
  step_type: StepType;
  order: number;
}

export interface Workflow {
  id: string;
  name: string;
  entry_point_id: string;
  steps: WorkflowStep[];
}

export interface ScanStats {
  files_scanned: number;
  files_skipped: number;
  scan_duration_ms: number;
}

export interface SysVistaOutput {
  version: string;
  scanned_at: string;
  root_dir: string;
  project_name: string;
  detected_languages: string[];
  components: DetectedComponent[];
  edges: DetectedEdge[];
  workflows: Workflow[];
  scan_stats: ScanStats;
}
