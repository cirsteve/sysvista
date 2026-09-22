import { sharedValue } from "@fixture/core";

export function answer(): number {
  return sharedValue();
}
