// Public catch-all. Any request to /t/<tid>/<rest> is forwarded through Redis
// pub/sub to the Function instance holding the CLI's WebSocket. We await the
// reply on resp:<tid>:<req_id> with a timeout.

import type { Frame } from "@/lib/proto";
import { newId } from "@/lib/proto";
import { keys, redis, subscriber } from "@/lib/redis";
import { injectShim } from "@/lib/html-shim";

export const dynamic = "force-dynamic";

const REPLY_TIMEOUT_MS = 55_000; // under Vercel Hobby 60s route cap

type Ctx = { params: Promise<{ tid: string; path?: string[] }> };

async function handle(req: Request, ctx: Ctx): Promise<Response> {
  const { tid, path } = await ctx.params;

  const r = redis();
  const alive = await r.exists(keys.tunnel(tid));
  if (!alive) {
    return new Response(`tunnel ${tid} not found or offline`, {
      status: 404,
      headers: { "content-type": "text/plain" },
    });
  }

  const reqId = newId(10);

  const url = new URL(req.url);
  // Preserve the "/" + anything after /t/<tid>, plus query string.
  const forwardPath = "/" + (path ?? []).join("/") + (url.search || "");

  const headers: Record<string, string> = {};
  req.headers.forEach((v, k) => {
    const kl = k.toLowerCase();
    if (kl === "host" || kl === "connection") return;
    headers[k] = v;
  });

  const bodyBuf = req.body ? await req.arrayBuffer() : new ArrayBuffer(0);
  const body_b64 = Buffer.from(bodyBuf).toString("base64");

  const outbound: Frame = {
    type: "request",
    req_id: reqId,
    method: req.method,
    path: forwardPath,
    headers,
    body_b64,
  };

  // Subscribe BEFORE publishing to avoid missing the reply.
  const sub = subscriber().duplicate();
  const chan = keys.respChan(tid, reqId);
  const waiter = new Promise<Frame>((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error("timeout")), REPLY_TIMEOUT_MS);
    sub.on("message", (_chan, payload) => {
      clearTimeout(timer);
      try {
        resolve(JSON.parse(payload) as Frame);
      } catch (e) {
        reject(e as Error);
      }
    });
  });
  await sub.subscribe(chan);

  await r.publish(keys.reqChan(tid), JSON.stringify(outbound));

  let reply: Frame;
  try {
    reply = await waiter;
  } catch {
    await sub.quit().catch(() => {});
    return new Response("tunnel timeout", { status: 504 });
  } finally {
    await sub.quit().catch(() => {});
  }

  if (reply.type === "error") {
    return new Response(`local app error: ${reply.message}`, { status: 502 });
  }
  if (reply.type !== "response") {
    return new Response("bad tunnel reply", { status: 502 });
  }

  const resHeaders = new Headers();
  let contentType = "";
  let contentEncoding = "";
  for (const [k, v] of Object.entries(reply.headers)) {
    const kl = k.toLowerCase();
    if (kl === "transfer-encoding" || kl === "connection" || kl === "content-length") {
      continue;
    }
    if (kl === "content-type") contentType = v;
    if (kl === "content-encoding") contentEncoding = v;

    // Rewrite redirect targets so a local "/login" redirect from the app
    // stays inside the tunnel ("/<tid>/login") instead of escaping to our
    // server root.
    let value = v;
    if (kl === "location" && value.startsWith("/") && !value.startsWith("//")) {
      const prefix = `/app/${tid}`;
      if (value !== prefix && !value.startsWith(prefix + "/")) {
        value = prefix + value;
      }
    }

    try {
      resHeaders.set(k, value);
    } catch {
      /* ignore invalid header name */
    }
  }

  // Only inject the SPA shim into HTML we can safely treat as UTF-8 text.
  const encodingOk = !contentEncoding || /^identity$/i.test(contentEncoding);
  const isHtml = /^text\/html\b/i.test(contentType);

  let bodyBytes: Uint8Array;
  if (isHtml && encodingOk) {
    const html = Buffer.from(reply.body_b64, "base64").toString("utf8");
    const injected = injectShim(html, tid);
    bodyBytes = new Uint8Array(Buffer.from(injected, "utf8"));
    resHeaders.delete("content-length");
  } else {
    bodyBytes = new Uint8Array(Buffer.from(reply.body_b64, "base64"));
  }
  return new Response(bodyBytes as BodyInit, {
    status: reply.status,
    headers: resHeaders,
  });
}

export const GET = handle;
export const POST = handle;
export const PUT = handle;
export const PATCH = handle;
export const DELETE = handle;
export const HEAD = handle;
export const OPTIONS = handle;
