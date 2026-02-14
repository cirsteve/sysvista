import Fuse from "fuse.js";
import type { DetectedComponent } from "../types/schema";

let fuse: Fuse<DetectedComponent> | null = null;

export function initSearch(components: DetectedComponent[]) {
  fuse = new Fuse(components, {
    keys: ["name", "source.file", "kind", "http_path"],
    threshold: 0.4,
    includeScore: true,
  });
}

export function search(query: string): DetectedComponent[] {
  if (!fuse || !query.trim()) return [];
  return fuse.search(query, { limit: 20 }).map((r) => r.item);
}
