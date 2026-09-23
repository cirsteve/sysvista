export function parse(value: string): number;
export function parse(value: number): number;
export function parse(value: string | number): number {
  const normalize = (input: string | number): number => Number(input);
  return normalize(value);
}
