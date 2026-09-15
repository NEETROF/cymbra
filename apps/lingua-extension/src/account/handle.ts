// The handle policy shared with Cymbra ID (backend/user/src/handle_core.rs, Music's
// auth_policy.dart): 1–15 Unicode letters or digits, no spaces or symbols. The server stays
// the authority — it also enforces case-insensitive uniqueness.

export const HANDLE_MAX_LENGTH = 15;

const HANDLE = /^[\p{L}\p{N}]{1,15}$/u;

export function isValidHandle(handle: string): boolean {
  return HANDLE.test(handle);
}

/** Live status of the handle being typed: local checks first, then the server's answer. */
export type HandleStatus = "empty" | "invalid" | "checking" | "available" | "taken" | "error";

/** The status a candidate has before asking the server ("checking" = worth asking). */
export function localHandleStatus(candidate: string): HandleStatus {
  const handle = candidate.trim();
  if (!handle) return "empty";
  return isValidHandle(handle) ? "checking" : "invalid";
}
