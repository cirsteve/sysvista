import { renamed as callMe } from "./reexport.js";
export function caller(value: string) { return callMe(value); }
export function dynamic(receiver: any) { return receiver.run(); }
