import type { CodeEntity, Relationship, Snapshot } from "../../types/v2";
import type { ScopeIndex, ScopeSlice, SnapshotSlice } from "./types";

export interface SliceSource {
  readScope(snapshotId: string, scopeId: string): Promise<ScopeSlice>;
  readEntities(snapshotId: string, ids: string[]): Promise<CodeEntity[]>;
  readRelationships(snapshotId: string, ids: string[]): Promise<Relationship[]>;
  readManifest(snapshotId: string): Promise<Snapshot["manifest"]>;
}

export function createSliceLoader(source: SliceSource) {
  const cache = new Map<string, Promise<SnapshotSlice>>();
  return (snapshotId: string, scopeId: string): Promise<SnapshotSlice> => {
    const key = `${snapshotId}\0${scopeId}`;
    const cached = cache.get(key);
    if (cached) return cached;
    const pending = source.readScope(snapshotId, scopeId).then(async (scope) => {
      const entities = await source.readEntities(snapshotId, [...new Set([
        ...scope.child_ids.map(String),
        ...Object.keys(scope.owner_map),
        ...Object.values(scope.owner_map).map(String),
      ])]);
      const relationships = await source.readRelationships(snapshotId, scope.crossing_relationship_ids);
      const endpointIds = relationships.flatMap(({ source, target }) => [source, target]);
      const known = new Set(entities.map(({ id }) => id));
      const referenced = await source.readEntities(snapshotId, endpointIds.filter((id) => !known.has(id)));
      return { snapshotId, scope, snapshot: { manifest: await source.readManifest(snapshotId), entities: [...entities, ...referenced], relationships } };
    });
    cache.set(key, pending);
    pending.catch(() => cache.delete(key));
    return pending;
  };
}

export async function readScopeFromIndex(response: Response, scopeId: string): Promise<ScopeSlice> {
  const index = await response.json() as ScopeIndex;
  const scope = index.scopes.find((item) => item.scope_id === scopeId);
  if (!scope) throw new Error(`Scope not found: ${scopeId}`);
  return scope;
}
