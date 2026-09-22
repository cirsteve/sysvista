import { describe, expect, it } from "vitest";
import { validateEntryPath } from "./paths";

describe("validateEntryPath", () => {
  it.each(["../x", "/abs", "a/../../b"])("rejects %s", (path) => {
    expect(validateEntryPath(path).ok).toBe(false);
  });

  it.each(["manifest.json", "index/scopes.json", "source/abc123"])("accepts %s", (path) => {
    expect(validateEntryPath(path)).toEqual({ ok: true, path });
  });

  it.each(["./manifest.json", "a//b", "C:/abs", "C:relative", "a\\b"])("rejects non-portable path %s", (path) => {
    expect(validateEntryPath(path).ok).toBe(false);
  });
});
