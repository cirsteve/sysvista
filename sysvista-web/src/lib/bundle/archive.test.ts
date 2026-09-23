import { strToU8, zipSync } from "fflate";
import { describe, expect, it } from "vitest";
import { openBundleArchive } from "./archive";

describe("openBundleArchive", () => {
  it("rejects an oversized source on lazy read while retaining metadata", async () => {
    const archive = zipSync({
      "source/too-large": strToU8("1234567890123456789012345678901"),
      "manifest.json": strToU8('{"schema_version":"2"}'),
    });
    const opened = await openBundleArchive(archive, 30);
    expect(new TextDecoder().decode(opened.metadata.get("manifest.json"))).toContain("schema_version");
    expect(await opened.read("source/too-large")).toBeUndefined();
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

  it("bounds accepted aggregate bytes and entry count", async () => {
    const archive = zipSync({
      "one.json": strToU8("1234567890"),
      "two.json": strToU8("1234567890"),
      "three.json": strToU8("ok"),
    });
    const aggregateBounded = await openBundleArchive(archive, 20, 12);
    expect(aggregateBounded.entries).toEqual(["one.json", "three.json"]);
    expect(aggregateBounded.diagnostics.some(({ message }) => message.includes("total uncompressed size"))).toBe(true);

    const countBounded = await openBundleArchive(archive, 20, 1_000, 2);
    expect(countBounded.entries).toEqual(["one.json", "two.json"]);
    expect(countBounded.diagnostics.some(({ message }) => message.includes("entry cap exceeded"))).toBe(true);
  });

  it("charges lazy source bytes only when opened", async () => {
    const archive = zipSync({ "manifest.json": strToU8("{}"), "source/one": strToU8("12345678"), "source/two": strToU8("abcdefgh") });
    const opened = await openBundleArchive(archive, 10, 12);
    expect(opened.entries).toContain("source/two");
    expect(new TextDecoder().decode(await opened.read("source/one"))).toBe("12345678");
    expect(await opened.read("source/two")).toBeUndefined();
  });

  it("aborts inflation when an entry understates its expanded size", async () => {
    const bytes = zipSync({ "source/understated": strToU8("x".repeat(100)) });
    const forged = bytes.slice();
    // ZIP local-header uncompressed size; fflate reads this before inflation.
    forged.set([1, 0, 0, 0], 22);
    const opened = await openBundleArchive(forged, 10, 200);
    await expect(opened.read("source/understated")).rejects.toThrow("inflated byte cap");
  });
});
