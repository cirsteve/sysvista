import { normalize as normalizeAlpha } from "./alpha.js";
import { normalize as normalizeBeta } from "./beta.js";

export function run(input: string): [string, string] {
  return [normalizeAlpha(input), normalizeBeta(input)];
}
