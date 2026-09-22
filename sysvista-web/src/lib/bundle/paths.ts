export type EntryPathResult =
  | { ok: true; path: string }
  | { ok: false; reason: string };

/** Validate the portable, normalized relative paths used by SysVista bundles. */
export function validateEntryPath(path: string): EntryPathResult {
  if (!path) return { ok: false, reason: "entry path is empty" };
  if (path.startsWith("/") || /^[A-Za-z]:\//.test(path)) {
    return { ok: false, reason: "entry path must be relative" };
  }
  if (path.includes("\\")) return { ok: false, reason: "entry path must use forward slashes" };
  const segments = path.split("/");
  if (segments.some((segment) => segment === "..")) {
    return { ok: false, reason: "entry path must not contain '..'" };
  }
  if (segments.some((segment) => segment === "" || segment === ".")) {
    return { ok: false, reason: "entry path must be normalized" };
  }
  return { ok: true, path };
}
