// Rules for tunnel/app names used as the first URL segment.

// Only paths that genuinely collide with the server itself are reserved.
export const RESERVED_NAMES = new Set([
  "api", // our WS endpoint and future control APIs
  "_next", // Next.js assets
  "favicon.ico",
  "robots.txt",
  "sitemap.xml",
]);

const NAME_RE = /^[a-z0-9][a-z0-9-]{1,30}[a-z0-9]$/;

export function isValidName(s: string | null | undefined): s is string {
  if (!s) return false;
  if (RESERVED_NAMES.has(s)) return false;
  return NAME_RE.test(s);
}
