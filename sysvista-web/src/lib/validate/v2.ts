import Ajv2020, { type ErrorObject } from "ajv/dist/2020";
import schema from "../../schema/sysvista-v2.schema.json";
import type { Snapshot } from "../../types/v2";
import type { Result } from "../result";

export interface ValidationError {
  instancePath: string;
  keyword: string;
  message: string;
}

const ajv = new Ajv2020({ allErrors: true });
ajv.addFormat("uint32", {
  type: "number",
  validate: (value: number) =>
    Number.isInteger(value) && value >= 0 && value <= 4_294_967_295,
});
ajv.addFormat("uint64", {
  type: "number",
  validate: (value: number) => Number.isSafeInteger(value) && value >= 0,
});
const validate = ajv.compile<Snapshot>(schema);

const toValidationError = (error: ErrorObject): ValidationError => ({
  instancePath: error.instancePath,
  keyword: error.keyword,
  message: error.message ?? "validation failed",
});

export function validateSnapshot(
  value: unknown,
): Result<Snapshot, ValidationError[]> {
  if (validate(value)) {
    return { ok: true, value };
  }

  return {
    ok: false,
    error: (validate.errors ?? []).map(toValidationError),
  };
}
