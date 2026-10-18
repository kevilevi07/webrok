// Validate a Vercel personal access token (or OAuth bearer token) by hitting
// the Vercel API. Returns the user id on success, null otherwise.

export async function validateVercelToken(token: string): Promise<string | null> {
  try {
    const r = await fetch("https://api.vercel.com/v2/user", {
      headers: { Authorization: `Bearer ${token}` },
    });
    if (!r.ok) return null;
    const j = (await r.json()) as { user?: { id?: string }; id?: string };
    return j.user?.id ?? j.id ?? null;
  } catch {
    return null;
  }
}
