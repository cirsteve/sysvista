import { describe, expect, it } from "vitest";

import { validateSnapshot } from "./v2";

const manifest = {
  schema_version: "3",
  root_scope_id: "root",
  repository: "example/sysvista",
  scanned_at: "2026-09-21T00:00:00Z",
  root: "/repo",
  tool_version: "0.1.0",
  inventory: {
    included: 1,
    excluded: 0,
    unsupported: 0,
    unreadable: 0,
    failed: 0,
  },
};

describe("validateSnapshot", () => {
  it("accepts a minimal v2 snapshot", () => {
    expect(validateSnapshot({ manifest })).toEqual({
      ok: true,
      value: { manifest },
    });
  });

  it("rejects an unknown relationship kind", () => {
    const result = validateSnapshot({
      manifest,
      relationships: [
        {
          id: "relationship-1",
          source: "entity-1",
          target: "entity-2",
          origin: "test",
          kind: "teleports",
        },
      ],
    });

    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(
        result.error.some(({ instancePath }) =>
          instancePath.startsWith("/relationships/0"),
        ),
      ).toBe(true);
    }
  });
});
