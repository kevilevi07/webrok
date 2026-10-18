import Link from "next/link";

export default function Download() {
  const base = (process.env.WEBROK_PUBLIC_URL ?? "https://webrok.vercel.app").replace(/\/$/, "");
  return (
    <>
      <nav>
        <div className="brand"><Link href="/" style={{ color: "inherit" }}>webrok</Link></div>
        <div className="links">
          <Link href="/login">Sign in</Link>
          <Link href="/signup">Sign up</Link>
        </div>
      </nav>
      <div className="container">
        <h1>Install webrok</h1>
        <p className="muted">A single Rust binary. ~2.5&nbsp;MB. No runtime dependencies.</p>

        <div className="card" style={{ marginTop: "1.5rem" }}>
          <h2 style={{ marginTop: 0 }}>Windows (PowerShell)</h2>
          <pre className="block">{`irm ${base}/install.ps1 | iex`}</pre>
          <p className="muted" style={{ fontSize: "0.9rem" }}>
            Installs to <code>%USERPROFILE%\\.webrok\\bin\\webrok.exe</code> and adds that
            directory to your <strong>user</strong> PATH. Open a new terminal, then run
            <code> webrok --help</code>.
          </p>
        </div>

        <div className="card" style={{ marginTop: "1.5rem" }}>
          <h2 style={{ marginTop: 0 }}>macOS / Linux</h2>
          <pre className="block">{`curl -sSL ${base}/install.sh | bash`}</pre>
          <p className="muted" style={{ fontSize: "0.9rem" }}>
            Installs to <code>~/.webrok/bin/webrok</code> and adds it to your shell PATH
            (<code>~/.bashrc</code>, <code>~/.zshrc</code>). Open a new terminal, then run
            <code> webrok --help</code>.
          </p>
        </div>

        <div className="card" style={{ marginTop: "1.5rem" }}>
          <h2 style={{ marginTop: 0 }}>Direct downloads</h2>
          <ul>
            <li><a href="/webrok.exe">webrok.exe</a> &mdash; Windows x86_64</li>
          </ul>
          <p className="muted" style={{ fontSize: "0.9rem" }}>
            After downloading, put the binary somewhere on your PATH or run it from
            its folder. Build it yourself from source:{" "}
            <code>cargo build --release --manifest-path cli/Cargo.toml</code>.
          </p>
        </div>

        <div style={{ marginTop: "2rem" }}>
          <Link className="btn btn-primary" href="/signup">Get a token</Link>
          <Link className="btn btn-ghost" href="/login" style={{ marginLeft: "0.5rem" }}>
            Already registered
          </Link>
        </div>
      </div>
    </>
  );
}
