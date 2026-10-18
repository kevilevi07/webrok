// `webrok doctor` — run connectivity probes. Useful on Termux / restricted
// networks where the normal connect fails and we need to pinpoint which
// layer (DNS, TCP, TLS, HTTP, auth) is broken.

use crate::config::Config;
use anyhow::Result;
use std::io::Write;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;

pub async fn run(cfg: &Config) -> Result<()> {
    let server = cfg
        .server
        .clone()
        .unwrap_or_else(|| "https://webrok.vercel.app".into());
    let token_set = cfg.token.is_some();

    println!("webrok doctor");
    println!("  server : {server}");
    println!("  token  : {}", if token_set { "set" } else { "<unset>" });
    println!();

    let url = url::Url::parse(&server)?;
    let host = url.host_str().unwrap_or("").to_string();
    let is_tls = url.scheme() == "https";
    let port = url.port().unwrap_or(if is_tls { 443 } else { 80 });

    // Step 1: DoH.
    let doh_addrs = probe_dns(
        "[1/5] DNS over HTTPS to cloudflare-dns.com:443",
        || crate::tunnel::resolve_doh(&host, port),
    )
    .await;

    // Step 2: classic DNS.
    let classic_addrs = probe_dns(
        "[2/5] DNS over UDP/TCP to 1.1.1.1 + 8.8.8.8",
        || crate::tunnel::resolve_classic(&host, port),
    )
    .await;

    let addrs = doh_addrs.or(classic_addrs).unwrap_or_default();
    if addrs.is_empty() {
        println!();
        println!("All DNS methods failed. Your network is blocking both UDP/53 and");
        println!("HTTPS to public resolvers. Try switching to mobile data, or disable");
        println!("any VPN/firewall and retry.");
        return Ok(());
    }
    let addr = addrs[0];

    // Step 3: raw TCP connect.
    {
        let start = Instant::now();
        print!("[3/5] TCP connect {addr} ... ");
        let _ = std::io::stdout().flush();
        match timeout(Duration::from_secs(10), TcpStream::connect(addr)).await {
            Ok(Ok(_)) => println!("OK ({:?})", start.elapsed()),
            Ok(Err(e)) => println!("FAIL ({:?}): {e}", start.elapsed()),
            Err(_) => println!("FAIL ({:?}): timeout after 10s", start.elapsed()),
        }
    }

    // Step 4: HTTPS GET using our resolved address (bypass system resolver).
    {
        let start = Instant::now();
        print!("[4/5] HTTPS GET {server} ... ");
        let _ = std::io::stdout().flush();
        let r: Result<u16> = async {
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .resolve(&host, addr)
                .build()?;
            let resp = client.get(&server).send().await?;
            Ok(resp.status().as_u16())
        }
        .await;
        match r {
            Ok(code) => println!("OK ({:?}): HTTP {code}", start.elapsed()),
            Err(e) => println!("FAIL ({:?}): {e}", start.elapsed()),
        }
    }

    // Step 5: WS handshake with token.
    if token_set {
        let token = cfg.token.clone().unwrap();
        let masked = mask(&token);
        let start = Instant::now();
        print!("[5/5] WebSocket handshake with token {masked} ... ");
        let _ = std::io::stdout().flush();
        let r: Result<String> = async {
            let ws_server = server
                .replace("https://", "wss://")
                .replace("http://", "ws://");
            let ws_url = format!("{ws_server}/api/tunnel?name=doctor-probe");
            use tokio_tungstenite::tungstenite::client::IntoClientRequest;
            let mut req = ws_url.as_str().into_client_request()?;
            req.headers_mut().insert(
                tokio_tungstenite::tungstenite::http::header::AUTHORIZATION,
                format!("Bearer {token}").parse().unwrap(),
            );
            let tcp = TcpStream::connect(addr).await?;
            let _ = tcp.set_nodelay(true);
            match tokio_tungstenite::client_async_tls_with_config(req, tcp, None, None).await {
                Ok(_) => Ok("handshake OK".to_string()),
                Err(tokio_tungstenite::tungstenite::Error::Http(resp)) => {
                    let status = resp.status();
                    let body = resp
                        .into_body()
                        .and_then(|b| String::from_utf8(b).ok())
                        .unwrap_or_default();
                    anyhow::bail!("HTTP {}: {}", status, body.trim());
                }
                Err(e) => Err(e.into()),
            }
        }
        .await;
        match r {
            Ok(v) => println!("OK ({:?}): {v}", start.elapsed()),
            Err(e) => println!("FAIL ({:?}): {e}", start.elapsed()),
        }
    } else {
        println!("[5/5] WebSocket handshake ... SKIPPED (run `webrok login` first)");
    }

    println!();
    println!("Done. Paste this output when asking for help.");
    Ok(())
}

async fn probe_dns<F, Fut>(label: &str, f: F) -> Option<Vec<SocketAddr>>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<Vec<SocketAddr>>>,
{
    let start = Instant::now();
    print!("{label} ... ");
    let _ = std::io::stdout().flush();
    match f().await {
        Ok(addrs) => {
            println!(
                "OK ({:?}): {} address(es), first={}",
                start.elapsed(),
                addrs.len(),
                addrs.first().map(|a| a.to_string()).unwrap_or_default()
            );
            Some(addrs)
        }
        Err(e) => {
            println!("FAIL ({:?}): {e}", start.elapsed());
            None
        }
    }
}

fn mask(t: &str) -> String {
    if t.len() <= 10 {
        return "****".into();
    }
    format!("{}...{}", &t[..6], &t[t.len() - 4..])
}
