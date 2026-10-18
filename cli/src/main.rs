mod config;
mod doctor;
mod login;
mod proto;
mod tunnel;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "webrok", version, about = "ngrok-style HTTP tunneling via Vercel")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,

    /// Server base URL (overrides config). Example: https://tunnel.example.com
    #[arg(long, global = true)]
    server: Option<String>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Save a webrok token locally (prompts if --token not given).
    Login {
        /// Token to save (otherwise prompted on stdin).
        #[arg(long)]
        token: Option<String>,
    },
    /// Delete the saved token.
    Logout,
    /// Show saved config.
    Status,
    /// Run connectivity probes (DNS, TCP, TLS, token). Useful for debugging
    /// why `webrok http` can't connect on restricted networks like Termux.
    Doctor,
    /// Forward a public URL to a local HTTP port.
    Http {
        /// Local port to forward to (e.g. 3000). Prompted if omitted.
        port: Option<u16>,
        /// App name — appears in the public URL as /<name>. Prompted if omitted.
        #[arg(long, short)]
        name: Option<String>,
        /// Local host to connect to.
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
}

fn prompt_name(given: Option<String>) -> Result<String> {
    use std::io::{self, Write};
    if let Some(n) = given {
        return Ok(n);
    }
    print!("App name (3-32 chars, lowercase a-z 0-9 -, e.g. my-app): ");
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let s = buf.trim().to_string();
    if s.is_empty() {
        anyhow::bail!("app name is required");
    }
    Ok(s)
}

fn prompt_port(given: Option<u16>) -> Result<u16> {
    use std::io::{self, Write};
    if let Some(p) = given {
        return Ok(p);
    }
    print!("Local port (e.g. 3000): ");
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    buf.trim().parse().map_err(|_| anyhow::anyhow!("invalid port"))
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "webrok=info".into()),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();
    let mut cfg = config::Config::load()?;
    if let Some(s) = &cli.server {
        cfg.server = Some(s.clone());
    }

    match cli.cmd {
        Cmd::Login { token } => login::run(&mut cfg, token).await,
        Cmd::Logout => {
            cfg.token = None;
            cfg.save()?;
            println!("Logged out.");
            Ok(())
        }
        Cmd::Status => {
            println!("Server:  {}", cfg.server.as_deref().unwrap_or("<unset>"));
            println!(
                "Token:   {}",
                if cfg.token.is_some() { "set" } else { "<unset>" }
            );
            println!("Config:  {}", config::path()?.display());
            Ok(())
        }
        Cmd::Doctor => doctor::run(&cfg).await,
        Cmd::Http { port, name, host } => {
            let name = prompt_name(name)?;
            let port = prompt_port(port)?;
            tunnel::run(&cfg, &host, port, Some(name)).await
        }
    }
}
