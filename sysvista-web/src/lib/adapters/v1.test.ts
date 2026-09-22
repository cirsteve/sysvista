import { describe, expect, it } from "vitest";
import sample from "../../test/fixtures/v1/sample-output.json";
import type { SysVistaOutput } from "../../types/schema";
import { adaptV1 } from "./v1";

describe("adaptV1", () => {
  it("builds deterministic file ownership and unordered traversal claims", () => {
    const snapshot = adaptV1(sample as SysVistaOutput);
    const component = snapshot.entities?.find(({ declaration_kind }) => declaration_kind === "service");
    expect(component?.owner_id).toMatch(/^legacy-file-entity:/);
    expect(snapshot.claims?.[0]).toMatchObject({ predicate: "HeuristicTraversal", object: { entity_ids: expect.any(Array), relationship_ids: expect.any(Array) } });
    expect(snapshot.claims?.[0].object).not.toHaveProperty("order");
  });
});
