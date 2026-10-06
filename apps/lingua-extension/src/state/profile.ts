import type { StudiedLanguage } from "../analyzer/types.ts";

// The reader's studied languages as a stored backup holds them, read without an engine
// (generalise-lingua-translation-model-state D2). The background needs them to know which
// translation models the reader's languages need, and a Chromium service worker that wakes for a
// translation must not instantiate the reading engine and its pack to learn them.
//
// The engine writes them as `profile.studied_languages`, under its enum's names; lingua-core's
// decks/backup.rs pins that layout in a test naming this reader. A backup without a profile —
// version 1, every reader of English alone — is English.

/** The engine's names for the studied languages (crates/lingua-core/src/analysis/language.rs). */
const NAMES: Readonly<Record<string, StudiedLanguage>> = { English: "en", Spanish: "es" };

const ENGLISH: StudiedLanguage[] = ["en"];

/** The studied languages of `backup`, the primary first; English when it names none this build knows. */
export function studiedLanguagesOf(backup: string): StudiedLanguage[] {
  try {
    const profile = (JSON.parse(backup) as { profile?: { studied_languages?: unknown } } | null)?.profile;
    const names = Array.isArray(profile?.studied_languages) ? profile.studied_languages : [];
    const languages = names.flatMap((name) => (typeof name === "string" && NAMES[name] ? [NAMES[name]] : []));
    return languages.length > 0 ? languages : ENGLISH;
  } catch {
    return ENGLISH;
  }
}
