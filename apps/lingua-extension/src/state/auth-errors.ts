import { Code, ConnectError } from "@connectrpc/connect";

// Auth/account failure categories (add-lingua-account-parity, design D7). The surfaces
// pick their copy from a category and the flow's context; a raw gRPC/Connect message
// never reaches the reader. Mirrors Music's `authErrorFromCode`
// (apps/music/lib/services/auth_service.dart).

export type AuthErrorKind =
  | "unauthenticated"
  | "alreadyExists"
  | "rateLimited"
  | "failedPrecondition"
  | "invalidArgument"
  | "notFound"
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

/** Categorize anything a flow can throw. A failed fetch (offline) counts as unavailable. */
export function authErrorOf(e: unknown): AuthErrorKind {
  if (e instanceof AccountError) return e.kind;
  if (e instanceof ConnectError) {
    if (e.code === Code.Unknown && e.cause instanceof TypeError) return "unavailable";
    return authErrorFromCode(e.code);
  }
  if (e instanceof TypeError) return "unavailable";
  return "unknown";
}
