import test from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
test("handshake exposes stable versions", () => {
  const value = JSON.parse(execFileSync(process.execPath, ["dist/analyzer.js", "--handshake"], { encoding: "utf8" }));
  assert.deepEqual(value, { contract_version: 2, analyzer_version: "0.2.0" });
});
