export type DecodedSource =
  | { kind: "text"; text: string }
  | { kind: "binary" };

/** Decode source while keeping malformed UTF-8 out of the text renderer. */
export function decodeSource(bytes: Uint8Array): DecodedSource {
  try {
    new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  } catch {
    return { kind: "binary" };
  }
  return { kind: "text", text: new TextDecoder("utf-8").decode(bytes) };
}
