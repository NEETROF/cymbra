// Roles, as the console handles them. Two different sets, deliberately not one:
//
//  - MANAGED_ROLES is what this console can GRANT — chosen here, closed, and therefore
//    assertable against the message catalogues (see test/i18n.spec.ts).
//  - A role it DISPLAYS comes from the server, which stores it as a free string:
//    `grant_role` guards the scope (`require_admin_in_scope`) and never validates the
//    role itself, so the console can meet a role it has no label for.
//
// That second case is why `roleLabel` exists. `$t('role.' + r)` on an unknown role
// renders the raw key — the roles panel shipped "SCOPE.LINGUA" to production that way
// when a scope label was missing. A role name is already human-readable, so falling
// back to it beats shouting a key at the admin.

/** The roles this console can grant or revoke. */
export const MANAGED_ROLES = ["moderator", "admin"] as const;
export type ManagedRole = (typeof MANAGED_ROLES)[number];

/**
 * Label `role` for display, falling back to the role's own name when no message
 * exists for it.
 *
 * `has`/`translate` are vue-i18n's `te`/`t` (injected rather than imported so this
 * stays a pure function). `te` checks the ACTIVE locale only, but the locales are
 * held to an identical key set by test/i18n.spec.ts, so a label present in one is
 * present in all.
 */
export function roleLabel(role: string, has: (key: string) => boolean, translate: (key: string) => string): string {
  const key = `role.${role}`;
  return has(key) ? translate(key) : role;
}
