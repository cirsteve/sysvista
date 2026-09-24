import type { PayloadContract, Snapshot } from "../../types/v2";

export interface PayloadAnnotation {
  argumentPayloads: string[];
  returnPayloads: string[];
}

export function annotatePayloads(snapshot: Snapshot, source: string, target: string): PayloadAnnotation {
  return (snapshot.payload_contracts ?? []).reduce<PayloadAnnotation>((result, contract: PayloadContract) => {
    const producers = new Set<string>((contract.producer_ids ?? []).map(String));
    const consumers = new Set<string>((contract.consumer_ids ?? []).map(String));
    const name = String(contract.name);
    return {
      argumentPayloads: producers.has(source) && consumers.has(target) ? [...result.argumentPayloads, name] : result.argumentPayloads,
      returnPayloads: consumers.has(source) && producers.has(target) ? [...result.returnPayloads, name] : result.returnPayloads,
    };
  }, { argumentPayloads: [], returnPayloads: [] });
}
