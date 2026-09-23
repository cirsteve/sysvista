import { describe, expect, it } from "vitest";
import fixture from "../test/fixtures/projection/root-with-three-scopes.json";
import type { Snapshot } from "../types/v2";
import { owningScopeForEntity, selectEvidenceComposition } from "./selectors";

describe("review selectors", () => {
  it("returns an entity hit's owning scope", () => {
    const snapshot = fixture.snapshot as unknown as Snapshot;
    const entity = snapshot.entities![0];
    expect(owningScopeForEntity(snapshot, entity.id)).toBe(entity.scope_id);
  });

  it("counts aggregate-edge evidence by origin", () => {
    expect(selectEvidenceComposition({ id: "e", presentation: "aggregate-edge", source: "a", target: "b", label: "calls", details: { kind: "calls", origin: "analyzer", origins: ["analyzer", "analyzer", "heuristic"], count: 3, relationshipIds: [] } })).toEqual({ analyzer: 2, heuristic: 1 });
  });
});
