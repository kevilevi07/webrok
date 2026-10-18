import type { NextConfig } from "next";

const config: NextConfig = {
  // Keep native/binary-flavored deps out of the bundler so their internal
  // require('bufferutil') / dynamic imports resolve against node_modules
  // at runtime instead of being partially inlined.
  serverExternalPackages: ["ws", "bufferutil", "utf-8-validate", "ioredis"],
  // Don't let Next.js rewrite trailing slashes in either direction. We need
  // this because:
  //   1. /app/<tid>/subdir/ must stay slashed, otherwise python http.server
  //      bounces back a 302 that re-slashes and we loop forever.
  //   2. /api/tunnel must stay unslashed, otherwise the WebSocket upgrade
  //      request gets 308-redirected and the handshake fails.
  // Both URL forms (/login and /login/) reach the matching page.
  skipTrailingSlashRedirect: true,
};

export default config;
