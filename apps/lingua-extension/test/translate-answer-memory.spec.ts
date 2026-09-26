import { describe, expect, it, vi } from "vitest";
import { ANSWER_MEMORY_SIZE, rememberAnswers } from "@/translate/answer-memory.ts";
import type { TranslationRequest, TranslationResult, TranslatorPort } from "@/translate/port.ts";

// A page does not ask twice for what it has already been answered (add-lingua-translation-android
// D4): measured, the handles asked for the same sentence five times in six seconds.

const sentence = "She gave up after the third attempt.";
const at = (start: number, end: number): TranslationRequest => ({ sentence, selection: { start, end } });
const translated = (text: string): TranslationResult => ({
  kind: "translated",
  translation: { sentence: text, marks: [{ start: 5, end: 16 }] },
});

/** A port that answers what the test says, counting what it is asked. */
function engine(answer: (r: TranslationRequest) => TranslationResult | Promise<TranslationResult>) {
  const port = { translate: vi.fn(async (r: TranslationRequest) => answer(r)), warm: vi.fn() } satisfies TranslatorPort;
  return port;
}

describe("rememberAnswers", () => {
  it("answers the same sentence and selection from memory, without asking again", async () => {
    const port = engine(() => translated("Elle a abandonné après la troisième tentative."));
    const page = rememberAnswers(port);
    const first = await page.translate(at(4, 11));
    const again = await page.translate(at(4, 11));
    expect(again).toEqual(first);
    expect(port.translate).toHaveBeenCalledOnce();
  });

  it("asks for a different selection of the same sentence: the engine marks the selection", async () => {
    const port = engine(() => translated("Elle a abandonné après la troisième tentative."));
    const page = rememberAnswers(port);
    await page.translate(at(4, 11));
    await page.translate(at(4, 17));
    await page.translate({ sentence, selection: null });
    expect(port.translate).toHaveBeenCalledTimes(3);
  });

  it("an identical request made while the first is being answered waits for that answer", async () => {
    let release!: (r: TranslationResult) => void;
    const port = engine(() => new Promise<TranslationResult>((r) => (release = r)));
    const page = rememberAnswers(port);
    const a = page.translate(at(4, 11));
    const b = page.translate(at(4, 11));
    release(translated("Elle a abandonné…"));
    await expect(Promise.all([a, b])).resolves.toEqual([
      translated("Elle a abandonné…"),
      translated("Elle a abandonné…"),
    ]);
    expect(port.translate).toHaveBeenCalledOnce();
  });

  it("keeps no absence of a translation: an unavailable answer is asked again", async () => {
    let ready = false;
    const port = engine(() => (ready ? translated("Elle a abandonné…") : { kind: "unavailable" }));
    const page = rememberAnswers(port);
    await expect(page.translate(at(4, 11))).resolves.toEqual({ kind: "unavailable" });
    ready = true;
    await expect(page.translate(at(4, 11))).resolves.toEqual(translated("Elle a abandonné…"));
    expect(port.translate).toHaveBeenCalledTimes(2);
  });

  it("a request that failed is asked again, and the failure reaches the caller", async () => {
    const port = engine(() => Promise.reject(new Error("gone")));
    const page = rememberAnswers(port);
    await expect(page.translate(at(4, 11))).rejects.toThrow("gone");
    await expect(page.translate(at(4, 11))).rejects.toThrow("gone");
    expect(port.translate).toHaveBeenCalledTimes(2);
  });

  it("keeps the most recent answers only, a re-read one counting as recent", async () => {
    const port = engine((r) => translated(`#${r.selection!.start}`));
    const page = rememberAnswers(port, 2);
    await page.translate(at(1, 2));
    await page.translate(at(2, 3));
    await page.translate(at(1, 2)); // re-read: now the most recent
    await page.translate(at(3, 4)); // evicts 2–3, the least recent
    port.translate.mockClear();
    await page.translate(at(1, 2));
    await page.translate(at(3, 4));
    expect(port.translate).not.toHaveBeenCalled();
    await page.translate(at(2, 3));
    expect(port.translate).toHaveBeenCalledOnce();
    expect(ANSWER_MEMORY_SIZE).toBe(32);
  });

  it("belongs to one page: two pages keep their own", async () => {
    const port = engine(() => translated("Elle a abandonné…"));
    await rememberAnswers(port).translate(at(4, 11));
    await rememberAnswers(port).translate(at(4, 11));
    expect(port.translate).toHaveBeenCalledTimes(2);
  });

  it("passes a warm through", () => {
    const port = engine(() => translated("x"));
    rememberAnswers(port).warm?.();
    expect(port.warm).toHaveBeenCalledOnce();
  });
});
