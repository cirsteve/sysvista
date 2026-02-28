import type { DetectedComponent, DetectedEdge } from "../types/schema";

/**
 * Classify components into semantic clusters.
 * - Models/transforms: CamelCase first word (e.g. "SessionPeerConfig" -> "Session")
 * - Transports: first meaningful http_path segment (e.g. "/sessions/{id}" -> "Sessions")
 * - Services: CamelCase first word from name
 * - Fallback: source directory name
 * - Fold clusters with <3 members into "Other"
 */

const classifyOne = (comp: DetectedComponent): string => {
  if (comp.kind === "transport" && comp.http_path) {
    const segments = comp.http_path
      .split("/")
      .filter((s) => s && !s.startsWith("{") && !s.startsWith(":"));
    if (segments.length > 0) {
      const seg = segments[0];
      return seg.charAt(0).toUpperCase() + seg.slice(1);
    }
  }

  if (comp.kind === "model" || comp.kind === "transform" || comp.kind === "service") {
    const match = comp.name.match(/^([A-Z][a-z]+)/);
    if (match) return match[1];
  }

  // Fallback: source directory name
  const parts = comp.source.file.split("/");
  return parts.length >= 2 ? parts[parts.length - 2] : "Other";
};

export function classifyComponents(
  components: DetectedComponent[],
): Map<string, string> {
  const raw = new Map(components.map((c) => [c.id, classifyOne(c)]));

  // Fold clusters with <3 members into "Other"
  const counts = [...raw.values()].reduce(
    (acc, c) => acc.set(c, (acc.get(c) ?? 0) + 1),
    new Map<string, number>(),
  );

  return new Map(
    [...raw.entries()].map(([id, cluster]) =>
      [id, (counts.get(cluster) ?? 0) < 3 ? "Other" : cluster],
    ),
  );
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
  const baseDegrees = new Map(components.map((c) => [c.id, 0]));

  const degreeMap = edges.reduce((acc, e) => {
    acc.set(e.from_id, (acc.get(e.from_id) ?? 0) + 1);
    acc.set(e.to_id, (acc.get(e.to_id) ?? 0) + 1);
    return acc;
  }, baseDegrees);

  const degrees = [...degreeMap.values()];
  if (degrees.length === 0) return new Map();

  const mean = degrees.reduce((a, b) => a + b, 0) / degrees.length;
  const variance = degrees.reduce((sum, d) => sum + (d - mean) ** 2, 0) / degrees.length;
  const stddev = Math.sqrt(variance);

  const assignTier = (degree: number): HubInfo["tier"] =>
    degree > mean + 2 * stddev ? "high"
    : degree > mean + stddev ? "medium"
    : "normal";

  return new Map(
    [...degreeMap.entries()].map(([id, degree]) => [id, { tier: assignTier(degree), degree }]),
  );
}
