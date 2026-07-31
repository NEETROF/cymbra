// Human label for a flag app scope. The `all` sentinel (a key shared across every
// Cymbra app) reads as "Global"; a specific app keeps its id (music, live, …).
export function appLabel(app: string, t: (key: string) => string): string {
  return app === "all" ? t("flags.appAll") : app;
}
