import { describe, expect, it } from "vitest";
import { decodeSource, verifySource } from "./source";

describe("decodeSource", () => {
  it("decodes UTF-8 source", () => {
    expect(decodeSource(new TextEncoder().encode("const café = true;"))).toEqual({ kind: "text", text: "const café = true;" });
  });

  it("identifies malformed UTF-8 as binary", () => {
    expect(decodeSource(Uint8Array.of(0xff, 0xfe, 0x00))).toEqual({ kind: "binary" });
  });
});

describe("verifySource", () => {
  const hash = "2689367b205c16ce32ed4200942b8b8b1e262dfc70d9bc9fbc77c49699a4f1df";
  it("rejects tampered hashes and byte lengths", async () => {
    const bytes = new TextEncoder().encode("ok");
    await expect(verifySource(bytes, hash, 2)).resolves.toBeUndefined();
    await expect(verifySource(bytes, hash, 3)).rejects.toThrow("byte_length");
    await expect(verifySource(new TextEncoder().encode("no"), hash, 2)).rejects.toThrow("SHA-256");
  });
});
