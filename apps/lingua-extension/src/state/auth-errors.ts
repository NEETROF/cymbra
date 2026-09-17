import { Code, ConnectError } from "@connectrpc/connect";

// Auth/account failure categories (add-lingua-account-parity, design D7). The surfaces
// pick their copy from a category and the flow's context; a raw gRPC/Connect message
// never reaches the reader. Mirrors Music's `authErrorFromCode`
// (apps/music/lib/services/auth_service.dart).

export type AuthErrorKind =
  /** The browser refused a write: this device's extension storage is full. */
  | "storageFull"
  | "unauthenticated"
  | "alreadyExists"
  | "rateLimited"
  | "failedPrecondition"
  | "invalidArgument"
  | "notFound"
  | "conflict"
  | "unavailable"
  | "unknown";

/** Map a gRPC status code (Connect's `Code` shares the numbering) to a category. */
export function authErrorFromCode(code: number): AuthErrorKind {
  switch (code) {
    case Code.InvalidArgument:
      return "invalidArgument";
    // A deadline overrun is indistinguishable from an unreachable backend for the reader.
    case Code.DeadlineExceeded:
    case Code.Unavailable:
      return "unavailable";
    case Code.NotFound:
      return "notFound";
    case Code.AlreadyExists:
      return "alreadyExists";
    // Optimistic-concurrency conflict, e.g. a handle claimed or the account updated mid-flight.
    case Code.Aborted:
      return "conflict";
    case Code.ResourceExhausted:
      return "rateLimited";
    case Code.FailedPrecondition:
      return "failedPrecondition";
    case Code.Unauthenticated:
      return "unauthenticated";
    default:
      return "unknown";
  }
}

/** A categorized account failure, thrown by the Session and carried in replies. */
export class AccountError extends Error {
  constructor(readonly kind: AuthErrorKind) {
    super(`account error: ${kind}`);
    this.name = "AccountError";
  }
}

/**
 * Whether the browser refused a write because the extension's storage area is full. Every
 * engine says so in its own words, and only in words — there is no code to test (Safari:
 * "Exceeded storage quota", Chromium: "QUOTA_BYTES quota exceeded", Firefox:
 * "QuotaExceededError").
 */
export function isStorageFull(e: unknown): boolean {
  const name = (e as { name?: unknown } | null)?.name;
  const message = (e as { message?: unknown } | null)?.message;
  return name === "QuotaExceededError" || (typeof message === "string" && /quota/i.test(message));
}

/** Categorize anything a flow can throw. A failed fetch (offline) counts as unavailable. */
export function authErrorOf(e: unknown): AuthErrorKind {
  if (e instanceof AccountError) return e.kind;
  // Before the TypeError branch: a refused write is not an unreachable server.
  if (isStorageFull(e)) return "storageFull";
  if (e instanceof ConnectError) {
    if (e.code === Code.Unknown && e.cause instanceof TypeError) return "unavailable";
    return authErrorFromCode(e.code);
  }
  if (e instanceof TypeError) return "unavailable";
  return "unknown";
}
