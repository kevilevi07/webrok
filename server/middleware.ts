// Path-based tunnels at /app/<name>/<path>. The tunneled app's frontend
// typically calls absolute paths like /api/foo; those would hit the webrok
// gateway instead of the tunneled app. We remember which tunnel the browser
// is viewing (Referer first, then cookie) and rewrite root-absolute paths
// back into /app/<name>/...

import { NextResponse, type NextRequest } from "next/server";
import { isValidName } from "@/lib/names";

const COOKIE = "webrok_tid";
const COOKIE_MAX_AGE = 60 * 60;
const APP_PREFIX = "/app/";

function tunnelFromReferer(req: NextRequest): string | null {
  const ref = req.headers.get("referer");
  if (!ref) return null;
  try {
    const u = new URL(ref);
    if (u.origin !== req.nextUrl.origin) return null;
    if (!u.pathname.startsWith(APP_PREFIX)) return null;
    const first = u.pathname.slice(APP_PREFIX.length).split("/")[0] ?? "";
    return isValidName(first) ? first : null;
  } catch {
    return null;
  }
}

export function middleware(req: NextRequest) {
  const { pathname } = req.nextUrl;

  // Server internals / landing pages — never rewrite.
  if (
    pathname === "/" ||
    pathname === "/favicon.ico" ||
    pathname.startsWith("/_next/") ||
    pathname.startsWith("/api/") ||
    pathname === "/login" ||
    pathname === "/signup" ||
    pathname === "/dashboard" ||
    pathname === "/download" ||
    pathname === "/logout" ||
    pathname.startsWith("/install.") // install.ps1, install.sh
  ) {
    return NextResponse.next();
  }

  // Case 1: already inside /app/<name>/... -> refresh cookie fallback.
  if (pathname.startsWith(APP_PREFIX)) {
    const first = pathname.slice(APP_PREFIX.length).split("/")[0] ?? "";
    if (isValidName(first)) {
      const res = NextResponse.next();
      if (req.cookies.get(COOKIE)?.value !== first) {
        res.cookies.set(COOKIE, first, {
          path: "/",
          maxAge: COOKIE_MAX_AGE,
          sameSite: "lax",
        });
      }
      return res;
    }
  }

  // Case 2: root-absolute request from inside a tunneled page — rewrite.
  const tid = tunnelFromReferer(req) ?? req.cookies.get(COOKIE)?.value;
  if (tid && isValidName(tid)) {
    const url = req.nextUrl.clone();
    url.pathname = `/app/${tid}${pathname}`;
    return NextResponse.rewrite(url);
  }

  return NextResponse.next();
}

export const config = {
  matcher: ["/((?!_next/static|_next/image|favicon.ico).*)"],
};
