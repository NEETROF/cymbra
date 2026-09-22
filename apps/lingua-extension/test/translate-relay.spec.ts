import { describe, expect, it, vi } from "vitest";
import type { EngineAccess, EngineReply } from "@/translate/host/engine.ts";
import { relayTranslation } from "@/translate/host/relay.ts";

/** An engine that records the markup it is handed and answers what the test says. */
function engine(answer: (markup: string) => EngineReply | Promise<EngineReply>) {
  const seen: string[] = [];
  const access: EngineAccess = {
    translate: async (markup) => {
      seen.push(markup);
      return answer(markup);
    },
  };
  return { access, seen };
}

const sentence = "She gave up after the third attempt.";
const selection = { start: 4, end: 11 }; // "gave up"

describe("relayTranslation", () => {
  it("marks the selection in its sentence and reads the translation back", async () => {
    const { access, seen } = engine(() => ({ ok: true, html: "Elle <b>a abandonné</b> après la troisième tentative." }));
    const result = await relayTranslation(access, { sentence, selection });

    expect(seen).toEqual(["She <b>gave up</b> after the third attempt."]);
    expect(result.kind).toBe("translated");
    if (result.kind !== "translated") return;
    const { sentence: fr, marks } = result.translation;
    expect(fr).toBe("Elle a abandonné après la troisième tentative.");
    expect(marks.map((m) => fr.slice(m.start, m.end))).toEqual(["a abandonné"]);
  });

  it("escapes page text before it reaches the engine", async () => {
    const { access, seen } = engine(() => ({ ok: true, html: "x" }));
    await relayTranslation(access, { sentence: "a <b> b", selection: { start: 0, end: 1 } });
    expect(seen).toEqual(["<b>a</b> &lt;b&gt; b"]);
  });

  it("translates the sentence unmarked when the selection's place is unknown", async () => {
    const { access, seen } = engine(() => ({ ok: true, html: "Elle a abandonné." }));
    const result = await relayTranslation(access, { sentence: "She gave up.", selection: null });
    expect(seen).toEqual(["She gave up."]);
    expect(result).toEqual({ kind: "translated", translation: { sentence: "Elle a abandonné.", marks: [] } });
  });

  it("answers unavailable, and logs why, when the engine gives nothing", async () => {
    const log = vi.fn();
    const { access } = engine(() => ({ ok: false, reason: "the engine did not start" }));
    await expect(relayTranslation(access, { sentence, selection }, log)).resolves.toEqual({ kind: "unavailable" });
    expect(log).toHaveBeenCalledWith("no translation:", "the engine did not start");
  });

  it("answers unavailable when reaching the engine throws", async () => {
    const log = vi.fn();
    const access: EngineAccess = { translate: () => Promise.reject(new Error("gone")) };
    await expect(relayTranslation(access, { sentence, selection }, log)).resolves.toEqual({ kind: "unavailable" });
    expect(log).toHaveBeenCalledOnce();
  });

  it("does not wake the engine for an empty sentence", async () => {
    const { access, seen } = engine(() => ({ ok: true, html: "x" }));
    await expect(relayTranslation(access, { sentence: "   ", selection: null })).resolves.toEqual({ kind: "unavailable" });
    expect(seen).toEqual([]);
  });
});
