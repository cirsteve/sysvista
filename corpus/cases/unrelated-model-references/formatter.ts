import type { AccountRecord } from "./models.js";

// Using a model as a type does not imply that this function persists it.
export function labelFor(record: AccountRecord): string {
  return record.displayName;
}
