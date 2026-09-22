import { ANALYZER_VERSION, CONTRACT_VERSION } from "./contract.js";
export function handshake() { return { contract_version: CONTRACT_VERSION, analyzer_version: ANALYZER_VERSION }; }
