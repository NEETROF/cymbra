import { describe, expect, it, vi } from "vitest";
import { REVIEW_SESSION_KEY, reviewContinuity } from "@/state/review-continuity.ts";
import type { AsyncStorageArea } from "@/state/storage.ts";

// The background's engine is stopped between two answers on Safari: its session is written after
// every call that moves it and resumed when the engine wakes (refine-lingua-review-session D3).

function area(): AsyncStorageArea & { raw: Record<string, unknown> } {
  const raw: Record<string, unknown> = {};
  return {
    raw,
    async get(keys) {
      const list = keys == null ? Object.keys(raw) : Array.isArray(keys) ? keys : [keys];
      return Object.fromEntries(list.filter((k) => k in raw).map((k) => [k, raw[k]]));
    },
    async set(items) {
      Object.assign(raw, items);
    },
  };
}

function engine(snapshot: string | null = "S1") {
  return { reviewSnapshot: vi.fn(async () => snapshot), reviewResume: vi.fn(async () => true) };
}

describe("the review session across a restart of the background's engine", () => {
  it("keeps the session after each call that moves it", async () => {
    const kept = area();
    const e = engine();
    const c = reviewContinuity(e, kept);
    for (const method of ["startReview", "reviewReveal", "reviewGrade", "reviewMarkKnown", "reviewIgnore"]) {
      kept.raw[REVIEW_SESSION_KEY] = undefined;
      await c.after(method);
      expect(kept.raw[REVIEW_SESSION_KEY]).toBe("S1");
    }
  });

  it("forgets it when the state is replaced, and leaves it alone otherwise", async () => {
    const kept = area();
    const c = reviewContinuity(engine(), kept);
    await c.after("startReview");
    await c.after("analyse");
    expect(kept.raw[REVIEW_SESSION_KEY]).toBe("S1");
    await c.after("restore");
    expect(kept.raw[REVIEW_SESSION_KEY]).toBeNull();
  });

  it("scenario: the iPhone stopped the background page — the next card is still there", async () => {
    const kept = area();
    await reviewContinuity(engine("S2"), kept).after("reviewGrade");
    // A new page, a new engine, restored from the backup: it resumes the session kept.
    const woken = engine(null);
    await reviewContinuity(woken, kept).resume();
    expect(woken.reviewResume).toHaveBeenCalledWith("S2");
  });

  it("resumes nothing when no session was kept, or without session storage", async () => {
    const e = engine();
    await reviewContinuity(e, area()).resume();
    await reviewContinuity(e, null).after("reviewGrade");
    await reviewContinuity(e, null).resume();
    expect(e.reviewResume).not.toHaveBeenCalled();
    expect(e.reviewSnapshot).not.toHaveBeenCalled();
  });
});
