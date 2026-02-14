import type { DetectedComponent, DetectedEdge } from "../types/schema";

/**
 * Classify components into semantic clusters.
 * - Models/transforms: CamelCase first word (e.g. "SessionPeerConfig" -> "Session")
 * - Transports: first meaningful http_path segment (e.g. "/sessions/{id}" -> "Sessions")
 * - Services: CamelCase first word from name
 * - Fallback: source directory name
 * - Fold clusters with <3 members into "Other"
 */
export function classifyComponents(
  components: DetectedComponent[],
): Map<string, string> {
  const clusterMap = new Map<string, string>();

  for (const comp of components) {
    let cluster: string | null = null;

    if (comp.kind === "transport" && comp.http_path) {
      // Extract first meaningful path segment: "/sessions/{id}" -> "Sessions"
      const segments = comp.http_path
        .split("/")
        .filter((s) => s && !s.startsWith("{") && !s.startsWith(":"));
      if (segments.length > 0) {
        const seg = segments[0];
        cluster = seg.charAt(0).toUpperCase() + seg.slice(1);
      }
    }

    if (!cluster && (comp.kind === "model" || comp.kind === "transform" || comp.kind === "service")) {
      // CamelCase first word: "SessionPeerConfig" -> "Session"
      const match = comp.name.match(/^([A-Z][a-z]+)/);
      if (match) {
        cluster = match[1];
      }
    }

    if (!cluster) {
      // Fallback: source directory name
      const parts = comp.source.file.split("/");
      if (parts.length >= 2) {
        cluster = parts[parts.length - 2];
      }
    }

    clusterMap.set(comp.id, cluster ?? "Other");
  }

  // Fold clusters with <3 members into "Other"
  const clusterCounts = new Map<string, number>();
  for (const c of clusterMap.values()) {
    clusterCounts.set(c, (clusterCounts.get(c) ?? 0) + 1);
  }
  for (const [id, cluster] of clusterMap) {
    if ((clusterCounts.get(cluster) ?? 0) < 3) {
      clusterMap.set(id, "Other");
    }
  }

  return clusterMap;
}

export interface HubInfo {
  tier: "high" | "medium" | "normal";
  degree: number;
}

/**
 * Detect hub nodes based on edge degree.
 * - high: degree > mean + 2*stddev
 * - medium: degree > mean + 1*stddev
 * - normal: everything else
 */
export function detectHubs(
  components: DetectedComponent[],
  edges: DetectedEdge[],
): Map<string, HubInfo> {
  const degreeMap = new Map<string, number>();
  for (const comp of components) {
    degreeMap.set(comp.id, 0);
  }
  for (const e of edges) {
    degreeMap.set(e.from_id, (degreeMap.get(e.from_id) ?? 0) + 1);
    degreeMap.set(e.to_id, (degreeMap.get(e.to_id) ?? 0) + 1);
  }

  const degrees = [...degreeMap.values()];
  if (degrees.length === 0) {
    return new Map();
  }

  const mean = degrees.reduce((a, b) => a + b, 0) / degrees.length;
  const variance =
    degrees.reduce((sum, d) => sum + (d - mean) ** 2, 0) / degrees.length;
  const stddev = Math.sqrt(variance);

  const hubMap = new Map<string, HubInfo>();
  for (const [id, degree] of degreeMap) {
    let tier: HubInfo["tier"] = "normal";
    if (degree > mean + 2 * stddev) {
      tier = "high";
    } else if (degree > mean + stddev) {
      tier = "medium";
    }
    hubMap.set(id, { tier, degree });
  }

  return hubMap;
}
