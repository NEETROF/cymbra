import { Code, ConnectError } from "@connectrpc/connect";
import { t } from "@/i18n";

// Map any thrown error to a short, user-facing (localized) message. Raw gRPC/Connect
// codes and messages (e.g. "[unauthenticated] invalid credentials") must NEVER reach
// the UI — the technical cause is logged to the console instead.
export function humanError(e: unknown): string {
  // Log the real cause for debugging; never shown to the user.
  console.error("action failed:", e);

  if (e instanceof ConnectError) {
    switch (e.code) {
      case Code.Unauthenticated:
        return t("errors.unauthenticated");
      case Code.PermissionDenied:
        return t("errors.permissionDenied");
      case Code.NotFound:
        return t("errors.notFound");
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
