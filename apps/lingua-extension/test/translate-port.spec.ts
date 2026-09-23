import { describe, expect, it, vi } from "vitest";
import { createTranslatorPort } from "@/translate/create-port.ts";
import { isEngineReply } from "@/translate/host/engine.ts";
import { MessagingTranslatorPort } from "@/translate/messaging-port.ts";
import { isTranslateMessage, TRANSLATE_TYPE } from "@/translate/wire.ts";

const request = { sentence: "She gave up.", selection: { start: 4, end: 11 } };

describe("createTranslatorPort", () => {
  it("gives no translator at all in a build that does not carry the engine", () => {
    // Every shipped build. The surfaces then answer exactly as they did before it existed.
    expect(createTranslatorPort()).toBeNull();
  });
});

describe("MessagingTranslatorPort", () => {
  it("sends the request to the background and returns its translation", async () => {
    const translation = { sentence: "Elle a abandonné.", marks: [{ start: 5, end: 16 }] };
    const send = vi.fn(async () => ({ kind: "translated", translation }));
    await expect(new MessagingTranslatorPort(send).translate(request)).resolves.toEqual({
      kind: "translated",
      translation,
    });
    expect(send).toHaveBeenCalledWith({ type: TRANSLATE_TYPE, request });
  });

  it("answers unavailable when nothing is listening", async () => {
    const send = vi.fn(async () => {
      throw new Error("Could not establish connection. Receiving end does not exist.");
    });
    await expect(new MessagingTranslatorPort(send).translate(request)).resolves.toEqual({ kind: "unavailable" });
  });

  it("answers unavailable for a reply it cannot read", async () => {
    for (const reply of [undefined, null, {}, { kind: "translated" }, { kind: "translated", translation: {} }]) {
      await expect(new MessagingTranslatorPort(async () => reply).translate(request)).resolves.toEqual({
        kind: "unavailable",
      });
    }
  });

  it("fills in the marks a reply left out", async () => {
    const send = async () => ({ kind: "translated", translation: { sentence: "Elle a abandonné." } });
    await expect(new MessagingTranslatorPort(send).translate(request)).resolves.toEqual({
      kind: "translated",
      translation: { sentence: "Elle a abandonné.", marks: [] },
    });
  });
});

describe("isTranslateMessage", () => {
  it("accepts a request with a selection or with none", () => {
    expect(isTranslateMessage({ type: TRANSLATE_TYPE, request })).toBe(true);
    expect(isTranslateMessage({ type: TRANSLATE_TYPE, request: { sentence: "x", selection: null } })).toBe(true);
  });

  it("rejects what the background must not act on", () => {
    // The background reads every runtime message; a guard that says yes to the wrong shape
    // hands the relay something it cannot mark.
    expect(isTranslateMessage({ type: "lingua-rpc", method: "analyse", args: [] })).toBe(false);
    expect(isTranslateMessage({ type: TRANSLATE_TYPE })).toBe(false);
    expect(isTranslateMessage({ type: TRANSLATE_TYPE, request: { selection: null } })).toBe(false);
    expect(isTranslateMessage({ type: TRANSLATE_TYPE, request: { sentence: "x" } })).toBe(false);
    expect(
      isTranslateMessage({ type: TRANSLATE_TYPE, request: { sentence: "x", selection: { start: 1.5, end: 2 } } }),
    ).toBe(false);
    expect(isTranslateMessage(null)).toBe(false);
  });
});

describe("isEngineReply", () => {
  it("accepts a translation or a reason, and nothing else", () => {
    expect(isEngineReply({ ok: true, html: "x" })).toBe(true);
    expect(isEngineReply({ ok: false, reason: "no model" })).toBe(true);
    expect(isEngineReply({ ok: true })).toBe(false);
    expect(isEngineReply({ ok: false })).toBe(false);
    expect(isEngineReply(undefined)).toBe(false);
  });
});
