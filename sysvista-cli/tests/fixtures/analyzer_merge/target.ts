export interface Target { value: string }
export interface Payload { target: string }
export function process(payload: Payload): Payload { return payload; }
