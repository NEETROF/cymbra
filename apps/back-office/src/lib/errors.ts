import { Code, ConnectError } from "@connectrpc/connect";

// Map any thrown error to a short, user-facing message. Raw gRPC/Connect codes and
// messages (e.g. "[unauthenticated] invalid credentials") must NEVER reach the UI —
// the technical cause is logged to the console instead. Kept generic per gRPC code;
// callers that need finer wording can map before calling `run`.
export function humanError(e: unknown): string {
  // Log the real cause for debugging; never shown to the user.
  console.error("action failed:", e);

  if (e instanceof ConnectError) {
    switch (e.code) {
      case Code.Unauthenticated:
        return "Identifiants invalides ou session expirée.";
      case Code.PermissionDenied:
        return "Accès refusé.";
      case Code.NotFound:
        return "Élément introuvable.";
      case Code.InvalidArgument:
        return "Requête invalide.";
      case Code.ResourceExhausted:
        return "Trop de tentatives. Réessaie plus tard.";
      case Code.Unavailable:
        return "Service indisponible. Réessaie.";
      default:
        return "Une erreur est survenue. Réessaie.";
    }
  }
  return "Une erreur est survenue. Réessaie.";
}
