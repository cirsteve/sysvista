import { readFileSync } from "node:fs";
import { ANALYZER_VERSION, CONTRACT_VERSION, type AnalyzeRequest, type AnalyzeResponse } from "./contract.js";
import { handshake } from "./handshake.js";

if (process.argv.includes("--handshake")) {
  process.stdout.write(JSON.stringify(handshake()));
} else {
  try {
    const request = JSON.parse(readFileSync(0, "utf8")) as AnalyzeRequest;
    const response: AnalyzeResponse = { contract_version: CONTRACT_VERSION, analyzer_version: ANALYZER_VERSION, entities: [], relationships: [], unresolved: [], diagnostics: request.contract_version === CONTRACT_VERSION ? [] : [{ message: `contract mismatch: ${request.contract_version}`, severity: "error" }], payloads: [] };
    process.stdout.write(JSON.stringify(response));
  } catch (error) {
    process.stderr.write(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
