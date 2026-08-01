// Decode a JWT payload WITHOUT verifying it. The signature is verified server-side
// on every call — the client only reads `roles`/`sub` to decide what UI to show
// (gate the console, hide admin actions). A tampered token changes nothing the
// server honours, so this is display-only, never an authorization decision.

export interface TokenClaims {
  sub?: string;
  aud?: string;
  roles: string[];
  /** Roles grouped by the scope they are held in (`global`/`music`/`live`), so the
   * UI can show only the scopes the admin may administer. Empty on legacy tokens. */
  rolesByScope: Record<string, string[]>;
  exp?: number;
}

/** The authorization scopes a role can live in (mirrors the backend's `SCOPES`). */
export const SCOPES = ["global", "music", "live"] as const;
export type Scope = (typeof SCOPES)[number];

function base64UrlDecode(input: string): string {
  const pad = input.length % 4 === 0 ? "" : "=".repeat(4 - (input.length % 4));
  const b64 = input.replaceAll("-", "+").replaceAll("_", "/") + pad;
  // atob is available in the browser and jsdom.
  return atob(b64);
}

export function decodeClaims(accessToken: string): TokenClaims {
  const parts = accessToken.split(".");
  if (parts.length !== 3) return { roles: [], rolesByScope: {} };
  try {
    const json = JSON.parse(base64UrlDecode(parts[1])) as Record<string, unknown>;
    const roles = Array.isArray(json.roles) ? (json.roles as unknown[]).map(String) : [];
    const rolesByScope: Record<string, string[]> = {};
    if (json.roles_by_scope && typeof json.roles_by_scope === "object") {
      for (const [scope, rs] of Object.entries(json.roles_by_scope as Record<string, unknown>)) {
        rolesByScope[scope] = Array.isArray(rs) ? (rs as unknown[]).map(String) : [];
      }
    }
    return {
      sub: typeof json.sub === "string" ? json.sub : undefined,
      aud: typeof json.aud === "string" ? json.aud : undefined,
      exp: typeof json.exp === "number" ? json.exp : undefined,
      roles,
      rolesByScope,
    };
  } catch {
    return { roles: [], rolesByScope: {} };
  }
}

export const isModerator = (roles: string[]): boolean => roles.includes("moderator") || roles.includes("admin");

export const isAdmin = (roles: string[]): boolean => roles.includes("admin");

/** True when the token holds `role` in `scope` — i.e. in the `global` break-glass
 * scope or in `scope` itself (mirrors the backend's `has_role_in_scope`). */
export function hasRoleInScope(rolesByScope: Record<string, string[]>, scope: string, role: string): boolean {
  return (rolesByScope.global ?? []).includes(role) || (rolesByScope[scope] ?? []).includes(role);
}

/** The scopes the caller may administer — those where they hold `admin` (a
 * `global/admin` gets all; a `music/admin` gets just `music`). */
export function adminScopes(rolesByScope: Record<string, string[]>): Scope[] {
  return SCOPES.filter((s) => hasRoleInScope(rolesByScope, s, "admin"));
}
