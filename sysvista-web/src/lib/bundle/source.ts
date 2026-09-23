export type DecodedSource =
  | { kind: "text"; text: string }
  | { kind: "binary" };

export async function verifySource(bytes: Uint8Array, expectedHash: string, expectedLength: number): Promise<void> {
  if (bytes.byteLength !== expectedLength) throw new Error(`Source byte_length mismatch: expected ${expectedLength}, got ${bytes.byteLength}`);
  const digest = await crypto.subtle.digest("SHA-256", bytes as BufferSource);
  const actual = [...new Uint8Array(digest)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
  if (actual !== expectedHash.toLowerCase()) throw new Error(`Source SHA-256 mismatch for ${expectedHash}`);
}

/** Decode source while keeping malformed UTF-8 out of the text renderer. */
export function decodeSource(bytes: Uint8Array): DecodedSource {
  try {
    new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  } catch {
    return { kind: "binary" };
  }
  return { kind: "text", text: new TextDecoder("utf-8").decode(bytes) };
}
