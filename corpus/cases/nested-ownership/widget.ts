import * as math from "./math.js";
export * from "./math.js";

export class Widget {
  private count = 0;
  constructor(seed: string) {
    this.count = math.parse(seed);
  }
  onClick = (): number => math.parse(this.total);
  get total(): number {
    return this.count;
  }
  set total(value: number) {
    this.count = value;
  }
  render(): number {
    return this.onClick();
  }
}
