import { describe, expect, it, vi } from "vitest";
import { createTranslatorPort } from "@/translate/create-port.ts";
import { isEngineReply } from "@/translate/host/engine.ts";
import { MessagingTranslatorPort } from "@/translate/messaging-port.ts";
import { KEEPALIVE_PING } from "@/translate/keepalive.ts";
import { isTranslateMessage, isWarmMessage, TRANSLATE_TYPE, WARM_TYPE } from "@/translate/wire.ts";

const request = { sentence: "She gave up.", selection: { start: 4, end: 11 } };

describe("createTranslatorPort", () => {
  it("gives no translator at all in a variant that does not carry the engine", () => {
    // Safari's. The surfaces then answer exactly as they did before it existed. What the others
    // give depends on the reader's setting: test/translate-setting.spec.ts.
    expect(createTranslatorPort()()).toBeNull();
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

describe("MessagingTranslatorPort.warm (add-lingua-translation-android D2)", () => {
  it("sends a warm and waits for nothing", () => {
    const send = vi.fn(async () => true);
    new MessagingTranslatorPort(send).warm();
    expect(send).toHaveBeenCalledWith({ type: WARM_TYPE });
  });

  it("never throws: no listener, or no extension context left", async () => {
    const rejecting = vi.fn(async () => Promise.reject(new Error("Could not establish connection")));
    expect(() => new MessagingTranslatorPort(rejecting).warm()).not.toThrow();
    const throwing = vi.fn(() => {
      throw new Error("Extension context invalidated");
    });
    expect(() => new MessagingTranslatorPort(throwing).warm()).not.toThrow();
    await Promise.resolve();
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

describe("isWarmMessage", () => {
  it("is its own message: not a translation, not the analyser, not the keep-warm ping", () => {
    expect(isWarmMessage({ type: WARM_TYPE })).toBe(true);
    expect(new Set([WARM_TYPE, TRANSLATE_TYPE, KEEPALIVE_PING, "lingua-rpc"]).size).toBe(4);
    expect(isWarmMessage({ type: TRANSLATE_TYPE, request })).toBe(false);
    expect(isWarmMessage({ type: KEEPALIVE_PING })).toBe(false);
    expect(isWarmMessage({ type: "lingua-rpc", method: "analyse", args: [] })).toBe(false);
    expect(isWarmMessage(null)).toBe(false);
    // …and a translation guard never takes a warm for a request.
    expect(isTranslateMessage({ type: WARM_TYPE })).toBe(false);
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
