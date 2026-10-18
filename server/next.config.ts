import type { NextConfig } from "next";

const config: NextConfig = {
  // Keep native/binary-flavored deps out of the bundler so their internal
  // require('bufferutil') / dynamic imports resolve against node_modules
  // at runtime instead of being partially inlined.
  serverExternalPackages: ["ws", "bufferutil", "utf-8-validate", "ioredis"],
  // Preserve trailing slashes on URLs. Without this, Next.js strips the
  // slash from /app/<tid>/subdir/ to /app/<tid>/subdir — but Python's
  // http.server (and many static servers) then 302-redirect back to the
  // slashed form for directories, which we rewrite back into the tunnel,
  // Next.js strips again, and so on. Keeping slashes stops that loop.
  trailingSlash: true,
};

export default config;
