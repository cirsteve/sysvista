export const CONTRACT_VERSION = 2;
export const ANALYZER_VERSION = "0.2.0";

export interface AnalyzeRequest { contract_version: number; root: string; files: string[]; tsconfig?: string }
export interface Span { file: string; start_line: number; start_column: number; end_line: number; end_column: number }
export interface Entity { name: string; ownership_chain: string; declaration_kind: string; file: string; discriminator: number; owner_key?: string; start_line: number; start_column: number; end_line: number; end_column: number; attributes: Record<string, unknown> }
export interface Relationship { kind: string; source: string; target?: string; origin: "resolved" | "partial"; name?: string; span?: Span }
export interface UnresolvedReference { source: string; name: string; span: Span; reason?: string }
export interface Diagnostic { message: string; severity: "error" | "warning"; span?: Span }
export interface Payload { name: string; producers: string[]; consumers: string[] }
export interface AnalyzeResponse { contract_version: number; analyzer_version: string; entities: Entity[]; relationships: Relationship[]; unresolved: UnresolvedReference[]; diagnostics: Diagnostic[]; payloads: Payload[] }
