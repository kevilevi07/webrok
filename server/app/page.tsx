import Link from "next/link";

export default function Home() {
  return (
    <>
      <nav>
        <div className="brand">webrok</div>
        <div className="links">
          <Link href="/download">Download</Link>
          <Link href="/login">Sign in</Link>
          <Link href="/signup">Sign up</Link>
        </div>
      </nav>

      <section className="hero">
        <h1>Public URLs for your localhost.</h1>
        <p className="lead">
          A small Rust binary that forwards a public HTTPS URL to anything running on
          your machine — like ngrok, but it runs on your own Vercel account.
        </p>
        <div className="cta">
          <Link className="btn btn-primary" href="/signup">Create an account</Link>
          <Link className="btn btn-ghost" href="/download">Install the CLI</Link>
        </div>
      </section>

      <div className="container">
        <pre className="block">{`# Install (Windows PowerShell)
irm https://webrok.vercel.app/install.ps1 | iex

# Share a local port
webrok http 3000 --name my-app
# -> https://webrok.vercel.app/app/my-app`}</pre>

        <div className="grid3">
          <div className="card feature">
            <h3>Named tunnels</h3>
            <p>Pick a human URL slug per app. No random hashes.</p>
          </div>
          <div className="card feature">
            <h3>Runs on your Vercel</h3>
            <p>Self-hosted on your own domain. No webrok middleman.</p>
          </div>
          <div className="card feature">
            <h3>SPA-friendly</h3>
            <p>Client-side routers, redirects, and absolute paths all handled.</p>
          </div>
        </div>
      </div>
    </>
  );
}
