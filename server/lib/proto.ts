// Wire protocol between the Rust CLI and this server (JSON over WS text frames).
// Kept in sync with cli/src/proto.rs.

export type Frame =
  | { type: "ready"; tunnel_id: string; public_url: string }
  | {
      type: "request";
      req_id: string;
      method: string;
      path: string;
      headers: Record<string, string>;
      body_b64: string;
    }
  | {
      type: "response";
      req_id: string;
      status: number;
      headers: Record<string, string>;
      body_b64: string;
    }
  | { type: "error"; req_id: string; message: string }
  | { type: "ping" }
  | { type: "pong" };

export function newId(len = 10): string {
  const bytes = new Uint8Array(len);
  crypto.getRandomValues(bytes);
  const alpha = "abcdefghijkmnopqrstuvwxyz23456789";
  return [...bytes].map((b) => alpha[b % alpha.length]).join("");
}
