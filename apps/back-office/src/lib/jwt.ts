// Decode a JWT payload WITHOUT verifying it. The signature is verified server-side
// on every call — the client only reads `roles`/`sub` to decide what UI to show
// (gate the console, hide admin actions). A tampered token changes nothing the
// server honours, so this is display-only, never an authorization decision.

export interface TokenClaims {
  sub?: string;
  aud?: string;
  roles: string[];
  exp?: number;
}

function base64UrlDecode(input: string): string {
  const pad = input.length % 4 === 0 ? "" : "=".repeat(4 - (input.length % 4));
  const b64 = input.replaceAll("-", "+").replaceAll("_", "/") + pad;
  // atob is available in the browser and jsdom.
  return atob(b64);
}

export function decodeClaims(accessToken: string): TokenClaims {
  const parts = accessToken.split(".");
  if (parts.length !== 3) return { roles: [] };
  try {
    const json = JSON.parse(base64UrlDecode(parts[1])) as Record<string, unknown>;
    const roles = Array.isArray(json.roles) ? (json.roles as unknown[]).map(String) : [];
    return {
      sub: typeof json.sub === "string" ? json.sub : undefined,
      aud: typeof json.aud === "string" ? json.aud : undefined,
      exp: typeof json.exp === "number" ? json.exp : undefined,
      roles,
    };
  } catch {
    return { roles: [] };
  }
}

export const isModerator = (roles: string[]): boolean => roles.includes("moderator") || roles.includes("admin");

export const isAdmin = (roles: string[]): boolean => roles.includes("admin");
