// Path-based tunnels at the ROOT: /<app-name>/<path>.
//
// Problem: a tunneled app's frontend typically calls absolute paths like
// /api/foo, /static/bar — those would hit webrok.vercel.app directly and 404.
//
// Fix: route them back into whichever tunnel the browser is currently
// viewing. We identify "which tunnel" using two signals, in order:
//   1. The `Referer` header  (per-TAB, so two tabs on two apps don't collide)
//   2. A fallback cookie `webrok_tid` (survives when a request has no Referer)
//
// Our own /api/tunnel (WS) and /_next/* are always left alone.

import { NextResponse, type NextRequest } from "next/server";
import { isValidName } from "@/lib/names";

const COOKIE = "webrok_tid";
const COOKIE_MAX_AGE = 60 * 60;

function tunnelFromReferer(req: NextRequest): string | null {
  const ref = req.headers.get("referer");
  if (!ref) return null;
  try {
    const u = new URL(ref);
    // Only trust same-origin referers.
    if (u.origin !== req.nextUrl.origin) return null;
    const first = u.pathname.split("/")[1] ?? "";
    return isValidName(first) ? first : null;
  } catch {
    return null;
  }
}

export function middleware(req: NextRequest) {
  const { pathname } = req.nextUrl;

  if (
    pathname === "/" ||
    pathname === "/favicon.ico" ||
    pathname.startsWith("/_next/") ||
    pathname.startsWith("/api/tunnel")
  ) {
    return NextResponse.next();
  }

  const firstSeg = pathname.split("/")[1] ?? "";

  // Case 1: already /<name>/... -> set/refresh cookie as a fallback for later
  // requests that may lack a Referer (e.g. fetch no-referrer, service workers).
  if (isValidName(firstSeg)) {
    const res = NextResponse.next();
    if (req.cookies.get(COOKIE)?.value !== firstSeg) {
      res.cookies.set(COOKIE, firstSeg, {
        path: "/",
        maxAge: COOKIE_MAX_AGE,
        sameSite: "lax",
      });
    }
    return res;
  }

  // Case 2: root-absolute (e.g. /api/foo). Route into tunnel if we can tell
  // which one the caller is in. Referer first (per-tab), cookie second.
  const tid = tunnelFromReferer(req) ?? req.cookies.get(COOKIE)?.value;
  if (tid && isValidName(tid)) {
    const url = req.nextUrl.clone();
    url.pathname = `/${tid}${pathname}`;
    return NextResponse.rewrite(url);
  }

  return NextResponse.next();
}

export const config = {
  matcher: ["/((?!_next/static|_next/image|favicon.ico).*)"],
};
