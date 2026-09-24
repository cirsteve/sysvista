import { format } from "../lib/format.js";

export function render(value: number): string {
  return format(value).padStart(4) + String(value);
}
