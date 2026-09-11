import type { LinguaPort, NewCard, Rating, ReviewCard } from "@/analyzer/port.ts";
import type { PageAnalysis } from "@/analyzer/types.ts";

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
    trackedCount: async () => calls.setStatus.length + calls.addCard.length,
    addCard: async (c) => void calls.addCard.push(c),
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
    notice: async () => "NOTICE",
    licences: async () => ["L1"],
  };
  return { port, calls };
}
