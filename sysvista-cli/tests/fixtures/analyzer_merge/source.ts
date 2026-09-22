import { Target } from "./target.js";

@Injectable()
export class Source {
  load(): Target { return { value: "ok" }; }
}
