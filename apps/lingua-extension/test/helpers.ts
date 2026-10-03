import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import type { LanguagePort, LinguaPort, NewCard, Rating, ReviewCard } from "@/analyzer/port.ts";
import type { PageAnalysis, StudiedLanguage } from "@/analyzer/types.ts";
import {
  DEFAULT_SPEECH_SETTINGS,
  type SpeechEngine,
  type SpeechSettings,
  type VoiceInfo,
  type VoicePreference,
} from "@/reading/speech.ts";

/** A deck entry for the fake review session. */
export interface FakeCard {
  headword: string;
  surface: string;
  sentence: string;
  gloss: string | null;
  /** The card's language, said by the review when set (add-lingua-language-stats-review). */
  language?: StudiedLanguage;
}

/**
 * The fake implements the whole-reader port and the language view in one object: `for` returns
 * a view delegating to the object it was called on, so every call lands in the same `calls`,
 * and a spec that spreads overrides over the fake sees them through its views too.
 */
export type FakePort = LinguaPort & LanguagePort;

export interface FakeCalls {
  /** The studied languages `for` was asked, in order (generalise-lingua-extension-port). */
  languages: StudiedLanguage[];
  /** Every document-language question: its candidates and hint (add-lingua-language-routing). */
  detections: { candidates: StudiedLanguage[]; hint: string | null }[];
  setCalibration: number[];
  setStatus: [string, string | null][];
  addCard: NewCard[];
  grades: Rating[];
  reveals: number;
  markKnown: number;
  restored: string[];
  /** The languages each due count and each review start were asked for (none: every language). */
  reviewLanguages: (StudiedLanguage[] | undefined)[];
}

/** A fake LinguaPort: records calls and simulates a review queue over `deck`. */
export function makeFakePort(deck: FakeCard[] = []): { port: FakePort; calls: FakeCalls } {
  const calls: FakeCalls = {
    languages: [],
    detections: [],
    setCalibration: [],
    setStatus: [],
    addCard: [],
    grades: [],
    reveals: 0,
    markKnown: 0,
    restored: [],
    reviewLanguages: [],
  };
  let queue: FakeCard[] = [];
  let pos = 0;
  let revealed = false;
  let backup = "{}";
  let studied: StudiedLanguage[] = ["en"];
  // A card without a language is English, as every card was before the reader studied several.
  const within = (languages: StudiedLanguage[] | undefined) => (card: FakeCard) =>
    !languages?.length || languages.includes(card.language ?? "en");

  const port: FakePort = {
    language: "en",
    for(this: FakePort, language: StudiedLanguage): LanguagePort {
      calls.languages.push(language);
      return Object.assign(Object.create(this) as FakePort, { language });
    },
    languages: async () => ["en"],
    studiedLanguages: async () => [...studied],
    setStudiedLanguages: async (languages) => {
      studied = [...languages];
    },
    // The first candidate, like a document with nothing to detect and no hint.
    detectLanguage: async (_blocks, candidates, hint) => {
      calls.detections.push({ candidates: [...candidates], hint });
      return candidates[0];
    },
    analyse: async (): Promise<PageAnalysis> => ({
      analyzer_version: "1.0.0",
      analysable: false,
      tokens: [],
      counted: 0,
      known: 0,
      percent: null,
    }),
    setCalibration: async (n) => void calls.setCalibration.push(n),
    calibration: async () => calls.setCalibration.at(-1) ?? 3000,
    setStatus: async (l, s) => void calls.setStatus.push([l, s]),
    gloss: async () => undefined,
    phraseGloss: async () => ({ tokens: [] }),
    wordGrammar: async () => ({ gloss: null, senses: [], readings: [], others: [], pieces: [] }),
    trackedCount: async () => calls.setStatus.length + calls.addCard.length,
    addCard: async (c) => void calls.addCard.push(c),
    retireCard: async () => {},
    deckCount: async () => calls.addCard.length,
    dueCount: async (_now, languages) => {
      calls.reviewLanguages.push(languages);
      return queue.slice(pos).filter(within(languages)).length;
    },
    startReview: async (_now, languages) => {
      calls.reviewLanguages.push(languages);
      queue = deck.filter(within(languages));
      pos = 0;
      revealed = false;
      return queue.length;
    },
    reviewCurrent: async (): Promise<ReviewCard | null> => {
      const c = queue[pos];
      return c ? { ...c, revealed, remaining: queue.length - pos } : null;
    },
    reviewReveal: async () => {
      revealed = true;
      calls.reveals++;
    },
    reviewGrade: async (r) => {
      calls.grades.push(r);
      pos++;
      revealed = false;
    },
    reviewMarkKnown: async () => {
      calls.markKnown++;
      pos++;
      revealed = false;
    },
    backup: async () => backup,
    restore: async (j) => {
      backup = j;
      calls.restored.push(j);
    },
    reset: async () => {
      queue = [];
      pos = 0;
      studied = ["en"];
    },
    resetStatuses: async () => {
      queue = [];
      pos = 0;
    },
    notice: async () => "NOTICE",
    licences: async () => ["L1"],
    setStatusAt: async (l, s) => void calls.setStatus.push([l, s]),
    exportStatusOps: async () => [],
    applyStatusChanges: async () => 0,
    exportCardOps: async () => [],
    applyCardOps: async () => 0,
    setDeclaredLevel: async () => {},
    setDeclaredLevelAt: async () => {},
    declaredLevel: async () => null,
    exportDeclaredLevels: async () => [],
    applyDeclaredLevelChanges: async () => 0,
    hasLevels: async () => false,
    levelLadder: async () => [],
    vocabularyEstimate: async () => ({ estimated: 0, confirmed: 0, universe: 0, basis: "marked" }),
    recordExposures: async () => {},
    promoteByExposure: async () => 0,
    seedLevel: async () => 0,
  };
  return { port, calls };
}

/** A voice list captured on a real browser (`test/fixtures/voices/<target>.json`). */
export function voiceFixture(target: string): VoiceInfo[] {
  const path = join(dirname(fileURLToPath(import.meta.url)), "fixtures", "voices", `${target}.json`);
  return (JSON.parse(readFileSync(path, "utf8")) as { voices: VoiceInfo[] }).voices;
}

/** One utterance the fake synthesiser was asked to speak; `done` ends it as the browser would. */
export interface FakeUtterance {
  text: string;
  voice: VoiceInfo;
  done: (error: string | null) => void;
}

/**
 * A scripted synthesiser: records what it is asked to speak, and lets a test announce a voice
 * list late (`list`) and end an utterance in whatever order the browsers do (`spoken[i].done`).
 * jsdom has no `speechSynthesis`, and the orders matter more here than the calls.
 */
export function makeFakeSpeech(voices: VoiceInfo[] = [], initial: Partial<SpeechSettings> = {}) {
  let listed = voices;
  let settings: SpeechSettings = { ...DEFAULT_SPEECH_SETTINGS, ...initial };
  const changed: (() => void)[] = [];
  const watchers: ((settings: SpeechSettings) => void)[] = [];
  const spoken: FakeUtterance[] = [];
  let cancels = 0;
  const engine: SpeechEngine = {
    voices: () => listed,
    onVoicesChanged: (listener) => void changed.push(listener),
    speak: (text, voice, done) => void spoken.push({ text, voice, done }),
    cancel: () => void cancels++,
  };
  const preference: VoicePreference = {
    load: async () => settings,
    watch: (onChange) => void watchers.push(onChange),
  };
  return {
    engine,
    preference,
    spoken,
    cancels: () => cancels,
    /** The browser announces its voices (Chrome: after the first `getVoices()`). */
    list(next: VoiceInfo[]) {
      listed = next;
      for (const listener of changed) listener();
    },
    /** The stored settings changed in another context. */
    prefer(next: Partial<SpeechSettings>) {
      settings = { ...settings, ...next };
      for (const watcher of watchers) watcher(settings);
    },
  };
}
