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

/** A failed score-preview render (`POST /scores/{id}/preview`), carrying the
 *  status and the response body so the server's typed refusal survives the
 *  transport. The route answers 412 with a precise reason — most often "no
 *  drum kit is configured for percussion previews" — and a generic "something
 *  went wrong" hides exactly the one thing an admin could act on. The body is
 *  never shown as is (see the no-raw-errors rule); it selects a localized
 *  message. */
export class ScorePreviewError extends Error {
  constructor(
    readonly status: number,
    readonly body: string = "",
  ) {
    super(`score preview regeneration failed: HTTP ${status}`);
    this.name = "ScorePreviewError";
  }
}

// Map any thrown error to a short, user-facing (localized) message. Raw gRPC/Connect
// codes and messages (e.g. "[unauthenticated] invalid credentials") must NEVER reach
// the UI — the technical cause is logged to the console instead.
/** Status → message for the SoundFont upload route, with a clear "already
 *  exists" for the dedup conflict (same id or identical content). */
function soundFontUploadMessage(status: number): string {
  switch (status) {
    case 409:
      return t("errors.soundfontExists");
    case 422:
      return t("errors.soundfontInvalid");
    case 413:
      return t("errors.soundfontTooLarge");
    default:
      return httpStatusMessage(status);
  }
}

/** Status → message for a score-preview refusal. The 412 reasons are
 *  preconditions an admin can act on, so they earn their own messages instead
 *  of the generic "not available yet"; the body selects between them and is
 *  never shown as is. */
function scorePreviewMessage(status: number, body: string): string {
  const b = body.toLowerCase();
  if (status === 412 && b.includes("drum_soundfont_id")) {
    return t("errors.previewDrumFontMissing");
  }
  if (status === 412 && b.includes("soundfont_id is unset")) {
    return t("errors.previewFontMissing");
  }
  if (status === 412) return t("errors.previewFontUnusable");
  if (status === 422) return t("errors.previewSilent");
  return httpStatusMessage(status);
}

/** The shared HTTP status → message table every browser-facing route falls
 *  back to, so a 401 reads the same whichever surface produced it. */
function httpStatusMessage(status: number): string {
  switch (status) {
    case 400:
      return t("errors.invalidArgument");
    case 401:
      return t("errors.unauthenticated");
    case 403:
      return t("errors.permissionDenied");
    case 404:
      return t("errors.notFound");
    case 412:
      return t("errors.notAvailable");
    case 429:
      return t("errors.resourceExhausted");
    case 503:
      return t("errors.unavailable");
    default:
      return t("errors.generic");
  }
}

// Map any thrown error to a short, user-facing (localized) message. Raw gRPC/Connect
// codes and messages (e.g. "[unauthenticated] invalid credentials") must NEVER reach
// the UI — the technical cause is logged to the console instead.
export function humanError(e: unknown): string {
  // Log the real cause for debugging; never shown to the user.
  console.error("action failed:", e);

  if (e instanceof SoundFontUploadError) return soundFontUploadMessage(e.status);
  if (e instanceof ScorePreviewError) return scorePreviewMessage(e.status, e.body);
  // Web-auth (cookie) HTTP failures map by status to the same messages as gRPC.
  if (e instanceof WebAuthError) return httpStatusMessage(e.status);

  if (e instanceof ConnectError) return connectMessage(e.code);
  return t("errors.generic");
}

/** gRPC/Connect code → the same localized messages the HTTP surfaces use. */
function connectMessage(code: Code): string {
  switch (code) {
    case Code.Unauthenticated:
      return t("errors.unauthenticated");
    case Code.PermissionDenied:
      return t("errors.permissionDenied");
    case Code.NotFound:
      return t("errors.notFound");
    // The thing being created is already there (a duplicate score, a taken collection
    // name, an existing membership). It fell through to the generic message, which ends
    // in "Try again" — telling the operator to retry something that can never succeed.
    case Code.AlreadyExists:
      return t("errors.alreadyExists");
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
