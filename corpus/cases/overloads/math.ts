export function convert(value: string): number;
export function convert(value: number): string;
export function convert(value: string | number): string | number {
  return typeof value === "string" ? value.length : String(value);
}
