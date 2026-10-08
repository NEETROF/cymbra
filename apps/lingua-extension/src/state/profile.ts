import { DEFAULT_NATIVE, pairsOf, SHIPPED_PAIRS } from "../analyzer/pairs.ts";
import type { NativeLanguage, StudiedLanguage } from "../analyzer/types.ts";

// The reader's profile as a stored backup holds it, read without an engine
// (generalise-lingua-translation-model-state D2). The background needs the studied languages to
// know which translation models the reader's languages need, and a Chromium service worker that
// wakes for a translation must not instantiate the reading engine and its pack to learn them.
// Every surface needs the native language before its engine loads its first pack, since the
// engine loads only the pairs glossed in it (generalise-lingua-native-language D7).
//
// The engine writes them as `profile.studied_languages` and `profile.native_language`, under its
// enums' names; lingua-core's decks/backup.rs pins that layout in tests naming this reader. A
// backup without a profile — version 1, every reader of English alone — is English studied,
// French native.

/** The engine's names for the studied languages (crates/lingua-core/src/analysis/language.rs). */
const NAMES: Readonly<Record<string, StudiedLanguage>> = { English: "en", Spanish: "es" };

/** The engine's names for the native languages (crates/lingua-core/src/knowledge/profile.rs). */
const NATIVE_NAMES: ReadonlyMap<unknown, NativeLanguage> = new Map<unknown, NativeLanguage>([
  ["French", "fr"],
  ["English", "en"],
  ["Spanish", "es"],
]);

const ENGLISH: StudiedLanguage[] = ["en"];

/**
 * The profile of `backup`, or undefined when it holds none or does not parse. A backup that does
 * not parse is said, not swallowed: read as empty it makes every reader French and English-studying,
 * and a Spanish reader whose interface turned French would otherwise have nothing to go on.
 */
function profileOf(backup: string): { studied_languages?: unknown; native_language?: unknown } | undefined {
  try {
    return (JSON.parse(backup) as { profile?: { studied_languages?: unknown; native_language?: unknown } } | null)
      ?.profile;
  } catch (e) {
    console.warn("[Cymbra Lingua] the stored backup does not parse; reading its profile as empty:", e);
    return undefined;
  }
}

/** The studied languages of `backup`, the primary first; English when it names none this build knows. */
export function studiedLanguagesOf(backup: string): StudiedLanguage[] {
  const profile = profileOf(backup);
  const names = Array.isArray(profile?.studied_languages) ? profile.studied_languages : [];
  const languages = names.flatMap((name) => (typeof name === "string" && NAMES[name] ? [NAMES[name]] : []));
  return languages.length > 0 ? languages : ENGLISH;
}

/**
 * The native language of `backup`: French when it names none, names one this build does not know,
 * or names one no listed pair is glossed in — an installed extension keeps its reader in French
 * without asking (M22).
 */
export function nativeLanguageOf(backup: string, pairs: readonly string[] = SHIPPED_PAIRS): NativeLanguage {
  const native = NATIVE_NAMES.get(profileOf(backup)?.native_language);
  return native !== undefined && pairsOf(native, pairs).length > 0 ? native : DEFAULT_NATIVE;
}
