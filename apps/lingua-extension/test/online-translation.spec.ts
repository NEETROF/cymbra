import { create } from "@bufbuild/protobuf";
import { Code, ConnectError } from "@connectrpc/connect";
import { afterEach, describe, expect, it, vi } from "vitest";
import { TranslateResponseSchema } from "@/gen/online_translation_pb.ts";
import { FLAGS_KEY } from "@/flags/store.ts";
import { ONLINE_TRANSLATION_KEY } from "@/state/storage.ts";
import {
  answerOnlineTranslation,
  createOnlineOffer,
  isOnlineTranslateMessage,
  MAX_SENTENCE_CHARS,
  ONLINE_TRANSLATE_TYPE,
  ONLINE_TRANSLATION_FLAG,
  type OnlineAnswerDeps,
  type OnlineTranslateMessage,
  onlineAccess,
  onlineOfferSource,
  translateOnline,
} from "@/translate/online.ts";

// « Traduction en ligne » (add-lingua-online-translation): the page's message and the background's
// answer to it, both driven through fakes — a `send` that records what would leave the page, and a
// mocked gRPC-web client whose answer the test writes as the service would.

const SENTENCE = "He kept putting off the dentist.";
/** « putting off », in UTF-16 code units. */
const SELECTION = { start: 8, end: 19 };

const flush = async (): Promise<void> => {
  for (let i = 0; i < 10; i++) await Promise.resolve();
};

describe("the page's message", () => {
  it("is recognised only whole: a sentence, a selection inside it, and the document's language", () => {
    const ok: OnlineTranslateMessage = {
      type: ONLINE_TRANSLATE_TYPE,
      sentence: SENTENCE,
      selection: SELECTION,
      source: "en",
    };
    expect(isOnlineTranslateMessage(ok)).toBe(true);
    expect(isOnlineTranslateMessage({ ...ok, type: "lingua-translate" })).toBe(false);
    expect(isOnlineTranslateMessage({ ...ok, source: "" })).toBe(false);
    expect(isOnlineTranslateMessage({ ...ok, sentence: 3 })).toBe(false);
    expect(isOnlineTranslateMessage({ ...ok, selection: { start: 4, end: 4 } })).toBe(false);
    expect(isOnlineTranslateMessage({ ...ok, selection: { start: 0, end: SENTENCE.length + 1 } })).toBe(false);
    expect(isOnlineTranslateMessage({ ...ok, selection: { start: 1.5, end: 3 } })).toBe(false);
    expect(isOnlineTranslateMessage(null)).toBe(false);
  });
});

describe("translateOnline (the page's half)", () => {
  const request = { sentence: SENTENCE, selection: SELECTION, source: "en" };

  it("sends the sentence, the selection and the document's language — never the native language", async () => {
    const send = vi.fn(async () => ({ kind: "unavailable" }));
    await translateOnline(request, send);
    expect(send).toHaveBeenCalledWith({
      type: ONLINE_TRANSLATE_TYPE,
      sentence: SENTENCE,
      selection: SELECTION,
      source: "en",
    });
  });

  it("hands back the translation and its marks", async () => {
    const translation = { sentence: "Il remettait toujours le dentiste à plus tard.", marks: [{ start: 3, end: 12 }] };
    const send = vi.fn(async () => ({ kind: "translated", translation }));
    expect(await translateOnline(request, send)).toEqual({ kind: "translated", translation });
  });

  it("drops a mark outside the translation rather than misplace it", async () => {
    const send = vi.fn(async () => ({
      kind: "translated",
      translation: { sentence: "Court.", marks: [{ start: 0, end: 5 }, { start: 3, end: 40 }, "x"] },
    }));
    expect(await translateOnline(request, send)).toEqual({
      kind: "translated",
      translation: { sentence: "Court.", marks: [{ start: 0, end: 5 }] },
    });
  });

  it("A translation without marks: the sentence alone", async () => {
    const send = vi.fn(async () => ({ kind: "translated", translation: { sentence: "Bonjour." } }));
    expect(await translateOnline(request, send)).toEqual({
      kind: "translated",
      translation: { sentence: "Bonjour.", marks: [] },
    });
  });

  it("hands back the limit and when it resets", async () => {
    const send = vi.fn(async () => ({ kind: "limit", resetsAt: 1_760_000_000_000 }));
    expect(await translateOnline(request, send)).toEqual({ kind: "limit", resetsAt: 1_760_000_000_000 });
  });

  it("reads anything else — nothing, a malformed limit, a thrown send — as unavailable", async () => {
    expect(await translateOnline(request, async () => undefined)).toEqual({ kind: "unavailable" });
    expect(await translateOnline(request, async () => ({ kind: "limit", resetsAt: "soon" }))).toEqual({
      kind: "unavailable",
    });
    expect(await translateOnline(request, async () => ({ kind: "translated" }))).toEqual({ kind: "unavailable" });
    expect(
      await translateOnline(request, () => {
        throw new Error("Could not establish connection. Receiving end does not exist.");
      }),
    ).toEqual({ kind: "unavailable" });
  });

  it("A 1 001-character sentence: unavailable, and no message reaches the background", async () => {
    const send = vi.fn(async () => ({ kind: "unavailable" }));
    const long = `${"a".repeat(MAX_SENTENCE_CHARS - 10)} gave up it`; // 1 001 characters
    expect(long.length).toBe(MAX_SENTENCE_CHARS + 1);
    expect(await translateOnline({ sentence: long, selection: { start: 991, end: 998 }, source: "en" }, send)).toEqual({
      kind: "unavailable",
    });
    expect(send).not.toHaveBeenCalled();
  });

  it("A sentence of exactly the bound is sent; the bound counts characters, not UTF-16 units", async () => {
    const send = vi.fn(async () => ({ kind: "unavailable" }));
    await translateOnline(
      { sentence: "a".repeat(MAX_SENTENCE_CHARS), selection: { start: 0, end: 1 }, source: "en" },
      send,
    );
    // 1 000 characters, each two UTF-16 units: within the server's bound.
    const astral = "😀".repeat(MAX_SENTENCE_CHARS);
    await translateOnline({ sentence: astral, selection: { start: 0, end: 2 }, source: "en" }, send);
    expect(send).toHaveBeenCalledTimes(2);
  });

  it("sends nothing for an empty sentence or a selection outside it", async () => {
    const send = vi.fn(async () => ({ kind: "unavailable" }));
    expect(await translateOnline({ sentence: "   ", selection: { start: 0, end: 1 }, source: "en" }, send)).toEqual({
      kind: "unavailable",
    });
    expect(await translateOnline({ sentence: "Hi.", selection: { start: 2, end: 9 }, source: "en" }, send)).toEqual({
      kind: "unavailable",
    });
    expect(send).not.toHaveBeenCalled();
  });

  it("leaves the footnote calls out, the selection moved with its text", async () => {
    const send = vi.fn(async () => ({ kind: "unavailable" }));
    await translateOnline({ sentence: "Cats[1] gave up here.", selection: { start: 8, end: 15 }, source: "en" }, send);
    expect(send).toHaveBeenCalledWith(
      expect.objectContaining({ sentence: "Cats gave up here.", selection: { start: 5, end: 12 } }),
    );
  });
});

describe("answerOnlineTranslation (the background's half)", () => {
  const message: OnlineTranslateMessage = {
    type: ONLINE_TRANSLATE_TYPE,
    sentence: SENTENCE,
    selection: SELECTION,
    source: "en",
  };

  function deps(answer: () => Promise<unknown>, over: Partial<OnlineAnswerDeps> = {}) {
    const translate = vi.fn<(request: unknown) => Promise<unknown>>(answer);
    const log = vi.fn();
    const d: OnlineAnswerDeps = {
      client: () => ({ translate }) as unknown as ReturnType<OnlineAnswerDeps["client"]>,
      native: async () => "fr",
      signedIn: () => true,
      log,
      ...over,
    };
    return { d, translate, log };
  }

  it("asks the service from the document's language into the reader's native language, the selection by offset", async () => {
    const { d, translate } = deps(async () => create(TranslateResponseSchema, {}));
    await answerOnlineTranslation(d, message);
    expect(translate).toHaveBeenCalledWith({
      sentence: SENTENCE,
      selectionStart: 8,
      selectionEnd: 19,
      sourceLanguage: "en",
      targetLanguage: "fr",
    });
  });

  it("A French reader of a Spanish book: es into fr, directly", async () => {
    const { d, translate } = deps(async () => create(TranslateResponseSchema, {}));
    await answerOnlineTranslation(d, { ...message, source: "es" });
    expect(translate.mock.calls[0]![0]).toMatchObject({ sourceLanguage: "es", targetLanguage: "fr" });
  });

  it("maps a translation to the MarkedTranslation the card renders, its marks as UTF-16 offsets", async () => {
    const translation = "Il remettait toujours le dentiste à plus tard.";
    const { d } = deps(async () =>
      create(TranslateResponseSchema, {
        outcome: {
          case: "translated",
          value: {
            translation,
            marks: [
              { start: 3, end: 12 },
              { start: 34, end: 45 },
            ],
          },
        },
      }),
    );
    expect(await answerOnlineTranslation(d, message)).toEqual({
      kind: "translated",
      translation: {
        sentence: translation,
        marks: [
          { start: 3, end: 12 },
          { start: 34, end: 45 },
        ],
      },
    });
  });

  it("drops a mark the translation cannot hold", async () => {
    const { d } = deps(async () =>
      create(TranslateResponseSchema, {
        outcome: { case: "translated", value: { translation: "Oui.", marks: [{ start: 2, end: 9 }] } },
      }),
    );
    expect(await answerOnlineTranslation(d, message)).toEqual({
      kind: "translated",
      translation: { sentence: "Oui.", marks: [] },
    });
  });

  it("maps the limit, its reset from unix seconds to milliseconds", async () => {
    const { d } = deps(async () =>
      create(TranslateResponseSchema, { outcome: { case: "limitReached", value: { resetsAt: 1_760_000_000n } } }),
    );
    expect(await answerOnlineTranslation(d, message)).toEqual({ kind: "limit", resetsAt: 1_760_000_000_000 });
  });

  it("maps unavailable, and an answer with no outcome, as unavailable", async () => {
    const unavailable = deps(async () =>
      create(TranslateResponseSchema, { outcome: { case: "unavailable", value: {} } }),
    );
    expect(await answerOnlineTranslation(unavailable.d, message)).toEqual({ kind: "unavailable" });
    const empty = deps(async () => create(TranslateResponseSchema, {}));
    expect(await answerOnlineTranslation(empty.d, message)).toEqual({ kind: "unavailable" });
  });

  it("A transport error: unavailable, and the log holds its code — never the sentence", async () => {
    const { d, log } = deps(async () => {
      throw new ConnectError(`echo: ${SENTENCE}`, Code.Unavailable);
    });
    expect(await answerOnlineTranslation(d, message)).toEqual({ kind: "unavailable" });
    expect(log).toHaveBeenCalledTimes(1);
    expect(JSON.stringify(log.mock.calls)).not.toContain("dentist");
  });

  it("An error with no code, and a profile that cannot be read: unavailable", async () => {
    const thrown = deps(async () => {
      throw "boom";
    });
    expect(await answerOnlineTranslation(thrown.d, message)).toEqual({ kind: "unavailable" });
    const noProfile = deps(async () => create(TranslateResponseSchema, {}), {
      native: async () => {
        throw new Error("no store");
      },
    });
    expect(await answerOnlineTranslation(noProfile.d, message)).toEqual({ kind: "unavailable" });
    expect(noProfile.translate).not.toHaveBeenCalled();
  });

  it("logs to the console when no log is given", async () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const { d } = deps(async () => {
      throw new ConnectError("down", Code.Unavailable);
    });
    await answerOnlineTranslation({ ...d, log: undefined }, message);
    expect(warn).toHaveBeenCalled();
    warn.mockRestore();
  });

  it("Signed out, a sentence over the bound, or a document in the reader's own language: nothing is asked", async () => {
    const signedOut = deps(async () => create(TranslateResponseSchema, {}), { signedIn: () => false });
    expect(await answerOnlineTranslation(signedOut.d, message)).toEqual({ kind: "unavailable" });
    expect(signedOut.translate).not.toHaveBeenCalled();

    const long = deps(async () => create(TranslateResponseSchema, {}));
    const sentence = "a".repeat(MAX_SENTENCE_CHARS + 1);
    expect(await answerOnlineTranslation(long.d, { ...message, sentence })).toEqual({ kind: "unavailable" });
    expect(long.translate).not.toHaveBeenCalled();

    const own = deps(async () => create(TranslateResponseSchema, {}));
    expect(await answerOnlineTranslation(own.d, { ...message, source: "fr" })).toEqual({ kind: "unavailable" });
    expect(own.translate).not.toHaveBeenCalled();
  });
});

describe("whether a card offers the button", () => {
  function source(store: Record<string, unknown>, access: () => Promise<boolean>) {
    const watchers = new Map<string, () => void>();
    const offered = onlineOfferSource({
      area: { get: async (key) => ({ [key as string]: store[key as string] }) },
      watch: (keys, onChange) => keys.forEach((k) => watchers.set(k, onChange)),
      access,
    });
    return { offered, change: (key: string) => watchers.get(key)?.() };
  }

  it("Setting ticked and access: offered", async () => {
    const { offered } = source({ [ONLINE_TRANSLATION_KEY]: true }, async () => true);
    expect(offered()).toBe(false); // until the first reads land: off, the safe answer
    await flush();
    expect(offered()).toBe(true);
  });

  it("Setting not ticked, or no access: not offered", async () => {
    const unticked = source({}, async () => true);
    const noAccess = source({ [ONLINE_TRANSLATION_KEY]: true }, async () => false);
    await flush();
    expect(unticked.offered()).toBe(false);
    expect(noAccess.offered()).toBe(false);
  });

  it("follows the setting and the flags: ticked on the page, then access withdrawn", async () => {
    const store: Record<string, unknown> = {};
    let access = true;
    const s = source(store, async () => access);
    await flush();
    expect(s.offered()).toBe(false);
    store[ONLINE_TRANSLATION_KEY] = true;
    s.change(ONLINE_TRANSLATION_KEY);
    await flush();
    expect(s.offered()).toBe(true);
    access = false;
    s.change(FLAGS_KEY);
    await flush();
    expect(s.offered()).toBe(false);
  });

  it("An area or a background that fails: not offered", async () => {
    const offered = onlineOfferSource({
      area: {
        get: async () => {
          throw new Error("Extension context invalidated.");
        },
      },
      watch: () => {},
      access: async () => {
        throw new Error("no background");
      },
    });
    await flush();
    expect(offered()).toBe(false);
  });

  it("access is the account's flag, signed in", async () => {
    const send = (flag: boolean, signedIn: boolean) => async (m: unknown) =>
      (m as { type: string }).type === "flags:get"
        ? { values: { [ONLINE_TRANSLATION_FLAG]: flag } }
        : { ok: true, state: { signedIn } };
    expect(await onlineAccess(send(true, true))).toBe(true);
    expect(await onlineAccess(send(true, false))).toBe(false);
    expect(await onlineAccess(send(false, true))).toBe(false);
  });
});

describe("createOnlineOffer (the browser's)", () => {
  afterEach(() => vi.unstubAllGlobals());

  it("reads the setting and the access from the extension, follows both, and stops with its surface", async () => {
    const listeners = new Set<(changes: Record<string, unknown>, area: string) => void>();
    let ticked = false;
    vi.stubGlobal("chrome", {
      storage: {
        local: { get: async () => ({ [ONLINE_TRANSLATION_KEY]: ticked }) },
        onChanged: {
          addListener: (fn: (changes: Record<string, unknown>, area: string) => void) => listeners.add(fn),
          removeListener: (fn: (changes: Record<string, unknown>, area: string) => void) => listeners.delete(fn),
        },
      },
      runtime: {
        sendMessage: async (m: { type: string }) =>
          m.type === "flags:get" ? { values: { [ONLINE_TRANSLATION_FLAG]: true } } : { state: { signedIn: true } },
      },
    });
    const life = new AbortController();
    const offered = createOnlineOffer(life.signal);
    await flush();
    expect(offered()).toBe(false);
    ticked = true;
    for (const l of listeners) l({ [ONLINE_TRANSLATION_KEY]: { newValue: true } }, "local");
    for (const l of listeners) l({ other: {} }, "sync");
    await flush();
    expect(offered()).toBe(true);
    expect(listeners.size).toBe(2);
    life.abort();
    expect(listeners.size).toBe(0);
  });
});
