use crate::config::Config;
use anyhow::{bail, Context, Result};
use std::io::{self, Write};

pub async fn run(cfg: &mut Config) -> Result<()> {
    if cfg.server.is_none() {
        print!("Server URL (e.g. https://webrok.vercel.app): ");
        io::stdout().flush()?;
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        let s = buf.trim().trim_end_matches('/').to_string();
        if s.is_empty() {
            bail!("server URL is required");
        }
        cfg.server = Some(s);
    }

    println!();
    println!("Create a Vercel token:  https://vercel.com/account/tokens");
    println!("(Scope: Full Account, no expiration if you want a long-lived tunnel.)");
    print!("Paste token: ");
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let token = buf.trim().to_string();
    if token.is_empty() {
        bail!("token is required");
    }

    // Validate against Vercel's API before saving.
    let client = reqwest::Client::new();
    let r = client
        .get("https://api.vercel.com/v2/user")
        .bearer_auth(&token)
        .send()
        .await
        .context("validating token against Vercel API")?;
    if !r.status().is_success() {
        bail!("Vercel rejected the token (HTTP {})", r.status());
    }

    cfg.token = Some(token);
    cfg.save()?;
    println!("Logged in. Token saved to {}", crate::config::path()?.display());
    Ok(())
}
