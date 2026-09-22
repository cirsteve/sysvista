import { describe, expect, it } from "vitest";
import { decodeSource } from "./source";

describe("decodeSource", () => {
  it("decodes UTF-8 source", () => {
    expect(decodeSource(new TextEncoder().encode("const café = true;"))).toEqual({ kind: "text", text: "const café = true;" });
  });

  it("identifies malformed UTF-8 as binary", () => {
    expect(decodeSource(Uint8Array.of(0xff, 0xfe, 0x00))).toEqual({ kind: "binary" });
  });
});
