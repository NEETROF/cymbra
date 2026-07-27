import { Code, ConnectError } from "@connectrpc/connect";
import { t } from "@/i18n";
import { WebAuthError } from "@/lib/web-auth";

// Map any thrown error to a short, user-facing (localized) message. Raw gRPC/Connect
// codes and messages (e.g. "[unauthenticated] invalid credentials") must NEVER reach
// the UI — the technical cause is logged to the console instead.
export function humanError(e: unknown): string {
  // Log the real cause for debugging; never shown to the user.
  console.error("action failed:", e);

  // Web-auth (cookie) HTTP failures map by status to the same messages as gRPC.
  if (e instanceof WebAuthError) {
    switch (e.status) {
      case 401:
        return t("errors.unauthenticated");
      case 403:
        return t("errors.permissionDenied");
      case 404:
        return t("errors.notFound");
      case 412:
        return t("errors.notAvailable");
      case 400:
        return t("errors.invalidArgument");
      case 429:
        return t("errors.resourceExhausted");
      case 503:
        return t("errors.unavailable");
      default:
        return t("errors.generic");
    }
  }

  if (e instanceof ConnectError) {
    switch (e.code) {
      case Code.Unauthenticated:
        return t("errors.unauthenticated");
      case Code.PermissionDenied:
        return t("errors.permissionDenied");
      case Code.NotFound:
        return t("errors.notFound");
      case Code.FailedPrecondition:
        return t("errors.notAvailable");
      case Code.InvalidArgument:
        return t("errors.invalidArgument");
      case Code.ResourceExhausted:
        return t("errors.resourceExhausted");
      case Code.Unavailable:
        return t("errors.unavailable");
      default:
        return t("errors.generic");
    }
  }
  return t("errors.generic");
}
