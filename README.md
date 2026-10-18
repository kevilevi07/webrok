# webrok

ngrok-style HTTP tunneling built on Vercel.

- `cli/` — Rust CLI (`webrok.exe`)
- `server/` — Next.js App Router server deployed to Vercel

Public requests reach `https://<your-domain>/t/<tunnel-id>/...`, get routed
through an Upstash Redis pub/sub bridge to the Function holding the Rust
CLI's outbound WebSocket, which forwards them to `http://localhost:<port>`
and streams the response back.

## Quick start

```powershell
# 1. Deploy the server
cd server
npm install
vercel link
vercel integration add upstash-redis
vercel env pull .env.local
vercel deploy --prod

# 2. Build the CLI
cd ..\cli
cargo build --release
# -> target\release\webrok.exe

# 3. Use it
.\target\release\webrok.exe login
.\target\release\webrok.exe http 3000
```

See `docs/` or the plan file for the full architecture.
