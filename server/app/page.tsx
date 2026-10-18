export default function Home() {
  return (
    <main style={{ fontFamily: "system-ui", maxWidth: 640, margin: "4rem auto", padding: "0 1rem" }}>
      <h1>webrok</h1>
      <p>ngrok-style HTTP tunneling on Vercel.</p>
      <ol>
        <li>Create a Vercel token at <a href="https://vercel.com/account/tokens">vercel.com/account/tokens</a>.</li>
        <li>Install the CLI: <code>cargo install --path cli</code></li>
        <li>Log in: <code>webrok login</code> &mdash; paste the token when prompted.</li>
        <li>Tunnel: <code>webrok http 3000</code></li>
      </ol>
    </main>
  );
}
