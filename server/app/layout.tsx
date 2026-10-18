export const metadata = { title: "webrok", description: "HTTP tunneling via Vercel" };

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body style={{ margin: 0, background: "#fafafa", color: "#111" }}>{children}</body>
    </html>
  );
}
