import type { AsyncStorageArea } from "./storage.ts";

// The review session the background's engine holds, kept across a restart of that engine
// (refine-lingua-review-session D3). On Safari the drawer's review runs on the background page's
// engine, and the browser stops that page between two answers, an iPhone within seconds; the engine
// then wakes, restores the reader's backup — and the session, which no backup holds, is gone: the
// next card said there was nothing to review. So after every call that moves the session the
// background writes it to the browser's session storage, and resumes it when the engine wakes.

/** Where the session under way is kept: storage.session, gone with the browser's session. */
export const REVIEW_SESSION_KEY = "cymbra-lingua-review-session";

/**
 * The calls after which the session is written again: those that move it, and those that replace
 * the engine's state, which keeps the session while it still holds its cards and ends it otherwise.
 */
const TOUCHES = new Set([
  "startReview",
  "reviewReveal",
  "reviewGrade",
  "reviewMarkKnown",
  "reviewIgnore",
  "restore",
  "reset",
  "resetStatuses",
]);

/** What of the engine this needs. */
export interface SessionHolder {
  reviewSnapshot(): Promise<string | null>;
  reviewResume(json: string): Promise<boolean>;
}

export interface ReviewContinuity {
  /** After `method` was answered: keep the session it moved, forget one it ended. */
  after(method: string): Promise<void>;
  /** The engine just woke and restored the backup: resume the session it held, if any. */
  resume(): Promise<void>;
}

/** Keep `engine`'s session in `area`; without an area (no session storage), nothing is kept. */
export function reviewContinuity(engine: SessionHolder, area: AsyncStorageArea | null): ReviewContinuity {
  return {
    async after(method) {
      if (!area) return;
      if (TOUCHES.has(method)) await area.set({ [REVIEW_SESSION_KEY]: await engine.reviewSnapshot() });
    },
    async resume() {
      if (!area) return;
      const kept = (await area.get(REVIEW_SESSION_KEY))[REVIEW_SESSION_KEY];
      if (typeof kept === "string") await engine.reviewResume(kept);
    },
  };
}
