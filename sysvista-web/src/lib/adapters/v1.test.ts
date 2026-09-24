import { describe, expect, expectTypeOf, it } from "vitest";
import sample from "../../test/fixtures/v1/sample-output.json";
import type { SysVistaOutput } from "../../types/schema";
import { adaptV1 } from "./v1";
import type { EntityId, HeuristicTraversalClaimObject, RelationshipId } from "../../types/v2";

describe("adaptV1", () => {
  it("builds deterministic file ownership and unordered traversal claims", () => {
    const snapshot = adaptV1(sample as SysVistaOutput);
    const component = snapshot.entities?.find(({ declaration_kind }) => declaration_kind === "service");
    expect(component?.owner_id).toMatch(/^legacy-file-entity:/);
    expect(snapshot.claims?.[0]).toMatchObject({ predicate: "HeuristicTraversal", object: { entity_ids: expect.any(Array), relationship_ids: expect.any(Array) } });
    expect(snapshot.claims?.[0].object).not.toHaveProperty("order");
    expect(snapshot.source_files).toHaveLength(3);
    expect(snapshot.entities?.filter(({ declaration_kind }) => declaration_kind === "file")).toHaveLength(3);
    const nonPersists = snapshot.relationships?.find(({ kind }) => kind !== "persists");
    expect(nonPersists).not.toHaveProperty("rule");
  });

  it("types heuristic traversal as unordered entity and relationship sets", () => {
    type HasOrder = "order" extends keyof HeuristicTraversalClaimObject ? true : false;
    expectTypeOf<HeuristicTraversalClaimObject["entity_ids"]>().toEqualTypeOf<EntityId[]>();
    expectTypeOf<HeuristicTraversalClaimObject["relationship_ids"]>().toEqualTypeOf<RelationshipId[]>();
    expectTypeOf<HasOrder>().toEqualTypeOf<false>();
  });
});
