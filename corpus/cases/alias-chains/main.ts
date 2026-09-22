import { transform as clean } from "./source.js";

export function run(value: string): string {
  return clean(value);
}
