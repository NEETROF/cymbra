import { Code, ConnectError } from "@connectrpc/connect";
import { t } from "@/i18n";
import { WebAuthError } from "@/lib/web-auth";

/** A failed SoundFont HTTP upload, carrying the status so `humanError` can localize
 *  it (the upload route is HTTP, not gRPC — bytes don't fit gRPC-web), and the
 *  response body so a typed refusal reason (e.g. the family-mismatch prefix —
 *  change: add-drum-audio-channel) survives the transport. */
export class SoundFontUploadError extends Error {
  constructor(
    readonly status: number,
    readonly body: string = "",
  ) {
    super(`soundfont upload failed: HTTP ${status}`);
    this.name = "SoundFontUploadError";
  }
}

// Map any thrown error to a short, user-facing (localized) message. Raw gRPC/Connect
// codes and messages (e.g. "[unauthenticated] invalid credentials") must NEVER reach
// the UI — the technical cause is logged to the console instead.
export function humanError(e: unknown): string {
  // Log the real cause for debugging; never shown to the user.
  console.error("action failed:", e);

  // SoundFont upload (HTTP route) failures map by status, with a clear
  // "already exists" for the dedup conflict (same id or identical content).
  if (e instanceof SoundFontUploadError) {
    switch (e.status) {
      case 409:
        return t("errors.soundfontExists");
      case 422:
        return t("errors.soundfontInvalid");
      case 413:
        return t("errors.soundfontTooLarge");
      case 401:
        return t("errors.unauthenticated");
      case 403:
        return t("errors.permissionDenied");
      case 400:
        return t("errors.invalidArgument");
      case 503:
        return t("errors.unavailable");
      default:
        return t("errors.generic");
    }
  }

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
