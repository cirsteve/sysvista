import { strToU8, zipSync } from "fflate";
import { describe, expect, it } from "vitest";
import { openBundleArchive } from "./archive";

describe("openBundleArchive", () => {
  it("rejects an oversized entry and continues loading the rest", async () => {
    const archive = zipSync({
      "too-large.json": strToU8("1234567890123456789012345678901"),
      "manifest.json": strToU8('{"schema_version":"2"}'),
    });
    const opened = await openBundleArchive(archive, 30);
    expect(opened.diagnostics.map(({ message }) => message).join(" ")).toContain("too-large.json");
    expect(new TextDecoder().decode(opened.metadata.get("manifest.json"))).toContain("schema_version");
    expect(opened.metadata.has("too-large.json")).toBe(false);
  });

  it("never exposes unsafe entries and lazily reads source by path", async () => {
    const archive = zipSync({
      "../escape": strToU8("bad"),
      "source/hash": strToU8("export const ok = true"),
    });
    const opened = await openBundleArchive(archive, 100);
    expect(opened.entries).not.toContain("../escape");
    expect(opened.metadata.has("source/hash")).toBe(false);
    expect(new TextDecoder().decode(await opened.read("source/hash"))).toContain("export const");
  });
});
