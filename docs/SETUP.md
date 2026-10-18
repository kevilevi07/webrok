# webrok setup

Step-by-step: deploy the server on Vercel, register a Vercel OAuth app, build
the CLI, and run your first tunnel.

## 1. Prereqs

- Node.js 22+ and npm (for the server)
- Rust 1.76+ (for the CLI) — install from https://rustup.rs
- Vercel CLI (`npm i -g vercel`)
- A Vercel account

## 2. Deploy the server

```powershell
cd server
npm install
vercel link                     # pick / create a project
vercel integration add upstash-redis   # provisions REDIS_URL + KV_URL
vercel env pull .env.local      # for local dev
```

### Register the Vercel OAuth app

1. Go to https://vercel.com/account/integrations/console
2. Create an **OAuth integration** (not a marketplace integration).
3. Redirect URL: `https://<your-vercel-domain>/api/oauth/callback`
4. Copy the client ID and secret.

Add them to your Vercel project env:

```powershell
vercel env add VERCEL_OAUTH_CLIENT_ID production
vercel env add VERCEL_OAUTH_CLIENT_SECRET production
vercel env add WEBROK_PUBLIC_URL production   # e.g. https://tunnel.your-domain.com
```

Pull them locally too:

```powershell
vercel env pull .env.local
```

### First deploy

```powershell
vercel deploy --prod
```

Note the production URL — that's your `--server` value for the CLI.

## 3. Build the CLI

```powershell
cd ..\cli
cargo build --release
```

Resulting binary: `cli\target\release\webrok.exe` (~5–8 MB).

Optional global install:

```powershell
cargo install --path .
```

## 4. Use it

```powershell
# Point the CLI at your server once; it saves to %APPDATA%\webrok\config.json.
webrok --server https://tunnel.your-domain.com login

# Follow the printed URL in your browser; approve.

# In another terminal, serve something locally:
python -m http.server 3000

# Then tunnel it:
webrok http 3000
# -> Forwarding https://tunnel.your-domain.com/t/abc123xy  ->  http://127.0.0.1:3000
```

Visit the printed URL. Requests hit Vercel -> Redis pub/sub -> your CLI -> localhost.

## 5. Known limits

- Request/response body cap: 4.5 MB (Vercel platform limit). Larger payloads
  need the body-chunking frame type (not yet implemented).
- WebSocket session is bounded by the Function duration (Hobby 300s, Pro 800s).
  The CLI auto-reconnects with the same tunnel id so the public URL is stable.
- HTTP(S) only. No raw TCP (SSH, Postgres, etc.).

## Troubleshooting

- **`webrok login` says "device code expired"** — approve within 10 minutes, or re-run `webrok login`.
- **404 on tunnel URL** — tunnel entry TTL expired because the CLI stopped pinging. Reconnect.
- **504 on tunnel URL** — CLI connected but local app took > 55s to answer, or the local app is down.
- **401 on WebSocket** — Vercel token invalid or expired. Run `webrok logout && webrok login`.
