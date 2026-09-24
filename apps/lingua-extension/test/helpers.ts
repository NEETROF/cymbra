import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import type { LinguaPort, NewCard, Rating, ReviewCard } from "@/analyzer/port.ts";
import type { PageAnalysis } from "@/analyzer/types.ts";
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
}

export interface FakeCalls {
  setCalibration: number[];
  setStatus: [string, string | null][];
  addCard: NewCard[];
  grades: Rating[];
  reveals: number;
  markKnown: number;
  restored: string[];
}

/** A fake LinguaPort: records calls and simulates a review queue over `deck`. */
export function makeFakePort(deck: FakeCard[] = []): { port: LinguaPort; calls: FakeCalls } {
  const calls: FakeCalls = {
    setCalibration: [],
    setStatus: [],
    addCard: [],
    grades: [],
    reveals: 0,
    markKnown: 0,
    restored: [],
  };
  let queue: FakeCard[] = [];
  let pos = 0;
  let revealed = false;
  let backup = "{}";

  const port: LinguaPort = {
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
    trackedCount: async () => calls.setStatus.length + calls.addCard.length,
    addCard: async (c) => void calls.addCard.push(c),
    retireCard: async () => {},
    deckCount: async () => calls.addCard.length,
    dueCount: async () => queue.length - pos,
    startReview: async () => {
      queue = [...deck];
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
