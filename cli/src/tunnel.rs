use crate::config::Config;
use crate::proto::Frame;
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use futures_util::{SinkExt, StreamExt};
use std::collections::BTreeMap;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::AUTHORIZATION;
use tokio_tungstenite::tungstenite::Message;

pub async fn run(
    cfg: &Config,
    local_host: &str,
    local_port: u16,
    requested_name: Option<String>,
) -> Result<()> {
    let server = cfg.require_server()?.to_string();
    let token = cfg.require_token()?.to_string();
    let mut tunnel_id = requested_name.or_else(|| cfg.tunnel_id.clone());

    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let mut backoff = Duration::from_millis(500);
    loop {
        match one_session(&server, &token, tunnel_id.as_deref(), &http, local_host, local_port)
            .await
        {
            Ok(Some(assigned)) => {
                tunnel_id = Some(assigned);
                backoff = Duration::from_millis(500);
            }
            Ok(None) => {
                backoff = Duration::from_millis(500);
            }
            Err(e) => {
                tracing::warn!(error = ?e, "session ended, will reconnect");
            }
        }
        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(Duration::from_secs(10));
    }
}

/// Returns Ok(Some(tunnel_id)) if a tunnel id was assigned/confirmed this session.
async fn one_session(
    server: &str,
    token: &str,
    reuse_id: Option<&str>,
    http: &reqwest::Client,
    local_host: &str,
    local_port: u16,
) -> Result<Option<String>> {
    let ws_url = {
        let mut u = url::Url::parse(server)?;
        let scheme = if u.scheme() == "https" { "wss" } else { "ws" };
        u.set_scheme(scheme)
            .map_err(|_| anyhow::anyhow!("bad server url scheme"))?;
        u.set_path("/api/tunnel");
        if let Some(id) = reuse_id {
            u.query_pairs_mut().append_pair("name", id);
        }
        u
    };

    let mut req = ws_url.as_str().into_client_request()?;
    req.headers_mut().insert(
        AUTHORIZATION,
        format!("Bearer {token}").parse().expect("valid header"),
    );

    tracing::info!("connecting {}", ws_url);
    let (ws, _resp) = tokio_tungstenite::connect_async(req)
        .await
        .context("websocket connect")?;
    let (mut tx, mut rx) = ws.split();

    let (out_tx, mut out_rx) = mpsc::channel::<Message>(64);

    // Writer task.
    let writer = tokio::spawn(async move {
        while let Some(m) = out_rx.recv().await {
            if tx.send(m).await.is_err() {
                break;
            }
        }
    });

    let mut assigned: Option<String> = None;
    let mut ping_ivl = tokio::time::interval(Duration::from_secs(25));
    ping_ivl.tick().await;

    loop {
        tokio::select! {
            msg = rx.next() => {
                let Some(msg) = msg else { break };
                let msg = msg?;
                match msg {
                    Message::Text(txt) => {
                        let frame = Frame::decode(&txt).context("decoding frame")?;
                        match frame {
                            Frame::Ready { tunnel_id, public_url } => {
                                println!("Forwarding {public_url}  ->  http://{local_host}:{local_port}");
                                assigned = Some(tunnel_id);
                            }
                            Frame::Request { req_id, method, path, headers, body_b64 } => {
                                let out_tx = out_tx.clone();
                                let http = http.clone();
                                let host = local_host.to_string();
                                tokio::spawn(async move {
                                    let reply = match forward(&http, &host, local_port, &method, &path, &headers, &body_b64).await {
                                        Ok((status, headers, body)) => Frame::Response {
                                            req_id: req_id.clone(),
                                            status,
                                            headers,
                                            body_b64: B64.encode(&body),
                                        },
                                        Err(e) => Frame::Error { req_id: req_id.clone(), message: e.to_string() },
                                    };
                                    let _ = out_tx.send(Message::Text(reply.encode())).await;
                                });
                            }
                            Frame::Ping => {
                                let _ = out_tx.send(Message::Text(Frame::Pong.encode())).await;
                            }
                            Frame::Pong => {}
                            Frame::Response { .. } | Frame::Error { .. } => {
                                // Not expected from server; ignore.
                            }
                        }
                    }
                    Message::Binary(_) => { /* reserved for future chunk frames */ }
                    Message::Ping(p) => { let _ = out_tx.send(Message::Pong(p)).await; }
                    Message::Pong(_) => {}
                    Message::Close(_) => break,
                    Message::Frame(_) => {}
                }
            }
            _ = ping_ivl.tick() => {
                if out_tx.send(Message::Text(Frame::Ping.encode())).await.is_err() {
                    break;
                }
            }
        }
    }

    drop(out_tx);
    let _ = writer.await;
    Ok(assigned)
}

async fn forward(
    http: &reqwest::Client,
    local_host: &str,
    local_port: u16,
    method: &str,
    path: &str,
    headers: &BTreeMap<String, String>,
    body_b64: &str,
) -> Result<(u16, BTreeMap<String, String>, bytes::Bytes)> {
    let url = format!("http://{local_host}:{local_port}{path}");
    let method = reqwest::Method::from_bytes(method.as_bytes())
        .unwrap_or(reqwest::Method::GET);
    let body = B64.decode(body_b64).unwrap_or_default();

    let mut rb = http.request(method, &url);
    for (k, v) in headers {
        let kl = k.to_ascii_lowercase();
        // Drop hop-by-hop + "accept-encoding" so the local app returns the
        // body uncompressed. The gateway (webrok server) needs plain bytes
        // to inject the SPA shim into HTML responses, and reqwest here isn't
        // compiled with gzip/brotli support.
        if matches!(
            kl.as_str(),
            "host" | "connection" | "content-length" | "transfer-encoding" | "accept-encoding"
        ) {
            continue;
        }
        rb = rb.header(k, v);
    }
    if !body.is_empty() {
        rb = rb.body(body);
    }

    let r = rb.send().await.map_err(|e| anyhow::anyhow!("local request failed: {e}"))?;
    let status = r.status().as_u16();
    let mut hdrs = BTreeMap::new();
    for (k, v) in r.headers().iter() {
        if let Ok(s) = v.to_str() {
            hdrs.insert(k.as_str().to_string(), s.to_string());
        }
    }
    let body = r.bytes().await?;
    Ok((status, hdrs, body))
}
