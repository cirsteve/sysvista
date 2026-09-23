import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import fixture from "../../test/fixtures/projection/root-with-three-scopes.json";
import type { Relationship, ScopeId, Snapshot } from "../../types/v2";
import { projectScope } from "./project";
import type { ScopeIndex } from "./types";

const WARM_SCOPE_LIMIT_MS = 300;

describe("projection scale threshold", () => {
  it("projects a warm scope within the measured threshold", () => {
    const fixturePath = process.env.SYSVISTA_BENCH_FIXTURE;
    const requestedCount = Number(process.env.SYSVISTA_BENCH_RELATIONSHIPS ?? "250000");
    const generated = fixturePath
      ? JSON.parse(readFileSync(fixturePath, "utf8")) as { snapshot: Snapshot; index: ScopeIndex; relationship_count: number }
      : undefined;
    const base = fixture.snapshot.relationships as unknown as Relationship[];
    const sourceId = fixture.snapshot.entities[0]?.id ?? "source";
    const targetId = fixture.snapshot.entities[1]?.id ?? "target";
    const relationships = generated?.snapshot.relationships ?? Array.from({ length: requestedCount }, (_, index) => ({
      ...(base.length ? base[index % base.length] : { kind: "calls", source: sourceId, target: targetId, origin: "resolved" }),
      id: `synthetic-${index}`,
    })) as Relationship[];
    const snapshot = generated?.snapshot ?? { ...fixture.snapshot, relationships } as unknown as Snapshot;
    const index = generated?.index ?? {
      scopes: [{
        ...(fixture.index as unknown as ScopeIndex).scopes[0],
        crossing_relationship_ids: relationships.map(({ id }) => id),
      }],
    } as ScopeIndex;
    const relationshipCount = generated?.relationship_count ?? requestedCount;

    const root = generated ? "root" as ScopeId : fixture.snapshot.manifest.root_scope_id as ScopeId;
    projectScope(snapshot, index, root);
    const samples = Array.from({ length: 3 }, () => {
      const start = performance.now();
      projectScope(snapshot, index, root);
      return performance.now() - start;
    }).sort((left, right) => left - right);
    const elapsed = samples[1];

    console.info(`projectScope warm ${relationshipCount} benchmark: ${elapsed.toFixed(1)}ms (hard limit ${WARM_SCOPE_LIMIT_MS}ms at 250000)`);
    if (relationshipCount === 250_000) expect(elapsed).toBeLessThanOrEqual(WARM_SCOPE_LIMIT_MS);
    else expect(elapsed).toBeGreaterThanOrEqual(0);
  }, 10_000);
});
