// « Traduction en ligne » (add-lingua-online-translation): the one place page text leaves the
// device — the sentence holding the reader's selection, sent only when the reader presses the
// card's button, after ticking the setting on this device, for an account that has access.
//
// The page never calls the network (D8): it sends a runtime message, and the background — which
// owns the session and the gRPC-web clients — calls `OnlineTranslationService.Translate` and maps
// its answer. Both halves live here so the protocol has one definition; neither imports the
// transport, so the content script that sends the message pulls no client in, and the background
// hands its client in.
//
// The service marks the selection in its own answer (D3): the card receives the `MarkedTranslation`
// it already renders, and never parses a model's output.

import type { Client } from "@connectrpc/connect";
import type { OnlineTranslationService } from "../gen/online_translation_pb.ts";
import { onFlagsChanged, readFlag } from "../flags/messages.ts";
import { FLAGS_KEY } from "../flags/store.ts";
import { ONLINE_TRANSLATION_KEY } from "../state/storage.ts";
import { type RuntimeSend, syncAvailable } from "../sync/messages.ts";
import { type MarkedTranslation, type Span, withoutFootnotes } from "./markup.ts";

/**
 * The account's access, read from its effective server flags (D6): true only for an account an
 * operator granted it. It is at once the grant and the kill switch; the server reads the same flag
 * before asking any provider, so a stale snapshot can show the button, never grant a translation.
 */
export const ONLINE_TRANSLATION_FLAG = "lingua.online_translation.enabled";

/**
 * The longest sentence sent, in characters (Unicode code points, as the server counts them): the
 * bound the server refuses past (D6). A longer one is never sent — the card says the online
 * translation is unavailable, as it would for any other refusal.
 */
export const MAX_SENTENCE_CHARS = 1000;

/** The privacy policy's annex, in the interface language: the setting's note links to it. */
export const PRIVACY_POLICY_URLS = {
  fr: "https://cymbra.app/confidentialite/",
  en: "https://cymbra.app/en/privacy/",
  es: "https://cymbra.app/es/privacidad/",
} as const;

/** Its own message type: an online translation never queues behind the engine's, nor the reverse. */
export const ONLINE_TRANSLATE_TYPE = "lingua-online-translate";

export interface OnlineTranslateMessage {
  type: typeof ONLINE_TRANSLATE_TYPE;
  /** The sentence the selection sits in, as the page shows it. */
  sentence: string;
  /** The selection's [start, end) in `sentence`, in UTF-16 code units, as the page counts them. */
  selection: Span;
  /**
   * The document's language. The reader's native language — the other half of the pair — is read
   * by the background from the stored profile, as for the engine (relay.ts): a page never names it.
   */
  source: string;
}

/**
 * The service's answer, as the card acts on it: a translation with the selection marked; nothing
 * (no provider answered, access withdrawn, a refused or failed request — the card does the same in
 * each case); or the daily limit reached, with when it resets, in epoch milliseconds.
 */
export type OnlineTranslationResult =
  | { kind: "translated"; translation: MarkedTranslation }
  | { kind: "unavailable" }
  | { kind: "limit"; resetsAt: number };

export const ONLINE_UNAVAILABLE: OnlineTranslationResult = { kind: "unavailable" };

/** What the page asks: a sentence, the selection in it, and the document's language. */
export interface OnlineTranslationRequest {
  sentence: string;
  selection: Span;
  source: string;
}

const isOffset = (n: unknown): n is number => typeof n === "number" && Number.isInteger(n) && n >= 0;

/** A span that covers something, inside a text of `length` code units. */
function spanWithin(span: Partial<Span> | null | undefined, length: number): span is Span {
  return !!span && isOffset(span.start) && isOffset(span.end) && span.start < span.end && span.end <= length;
}

/** The sentence's length as the server bounds it: in code points, not UTF-16 code units. */
function characters(text: string): number {
  return Array.from(text).length;
}

export function isOnlineTranslateMessage(message: unknown): message is OnlineTranslateMessage {
  const m = message as Partial<OnlineTranslateMessage> | null;
  if (m?.type !== ONLINE_TRANSLATE_TYPE) return false;
  if (typeof m.sentence !== "string" || typeof m.source !== "string" || !m.source) return false;
  return spanWithin(m.selection, m.sentence.length);
}

/** A reply as the page reads it: anything it does not recognise is no translation. */
function asResult(reply: unknown): OnlineTranslationResult {
  const r = reply as
    { kind?: unknown; translation?: Partial<MarkedTranslation>; resetsAt?: unknown } | null | undefined;
  if (r?.kind === "translated" && typeof r.translation?.sentence === "string") {
    const sentence = r.translation.sentence;
    const marks = Array.isArray(r.translation.marks) ? r.translation.marks : [];
    return {
      kind: "translated",
      translation: { sentence, marks: marks.filter((m) => spanWithin(m, sentence.length)) },
    };
  }
  if (r?.kind === "limit" && typeof r.resetsAt === "number" && Number.isFinite(r.resetsAt)) {
    return { kind: "limit", resetsAt: r.resetsAt };
  }
  return ONLINE_UNAVAILABLE;
}

export type OnlineSend = (message: OnlineTranslateMessage) => Promise<unknown>;

const runtimeSend: OnlineSend = (message) => chrome.runtime.sendMessage(message);

/**
 * The message `request` becomes, or null for one that must not leave the page: the footnote calls
 * removed first (the selection moves with its text), then a sentence that is empty, over
 * MAX_SENTENCE_CHARS, or a selection outside it, refused. The Cymbra Lingua app answers the message
 * itself (`answerOnlineTranslation`, add-lingua-review-translation-fallback D8).
 */
export function onlineMessage(request: OnlineTranslationRequest): OnlineTranslateMessage | null {
  const { sentence, selection } = withoutFootnotes(request.sentence, request.selection);
  if (!sentence.trim() || characters(sentence) > MAX_SENTENCE_CHARS) return null;
  if (!spanWithin(selection, sentence.length)) return null;
  return { type: ONLINE_TRANSLATE_TYPE, sentence, selection, source: request.source };
}

/**
 * Ask the background for the online translation of `request`. Never throws. The footnote calls go
 * first, as for the engine (the selection moves with its text); then a sentence that is empty, over
 * MAX_SENTENCE_CHARS, or a selection outside it, answers unavailable WITHOUT a message: nothing the
 * server would refuse leaves the page.
 */
export async function translateOnline(
  request: OnlineTranslationRequest,
  send: OnlineSend = runtimeSend,
): Promise<OnlineTranslationResult> {
  const message = onlineMessage(request);
  if (!message) return ONLINE_UNAVAILABLE;
  try {
    return asResult(await send(message));
  } catch {
    // A torn-down background, a closed port: no translation right now.
    return ONLINE_UNAVAILABLE;
  }
}

// — The background's half —

export type OnlineLog = (message: string, detail?: unknown) => void;

const LOG: OnlineLog = (message, detail) => console.warn(`[Cymbra Lingua] ${message}`, detail ?? "");

/** What answering a page takes of the background: the client, the reader's profile, the session. */
export interface OnlineAnswerDeps {
  /** The gRPC-web client of the service (`api().onlineTranslation`). */
  client: () => Pick<Client<typeof OnlineTranslationService>, "translate">;
  /** The reader's native language, from their stored profile: the translation's target. */
  native: () => Promise<string>;
  /** Signed out, nothing is sent: the service would refuse it, and no request leaves unasked. */
  signedIn: () => boolean;
  log?: OnlineLog;
}

/**
 * A page's request, answered: the service's `translated` as the `MarkedTranslation` the card renders
 * (its marks are UTF-16 offsets into the translation, as the page counts them), `limit_reached` as
 * the limit with its reset in milliseconds, and everything else — `unavailable`, signed out, an
 * over-long sentence, a pair of one language, any transport error — as unavailable. Never throws, and
 * never logs the sentence or the translation: only that a request failed, and its code.
 */
export async function answerOnlineTranslation(
  deps: OnlineAnswerDeps,
  message: OnlineTranslateMessage,
): Promise<OnlineTranslationResult> {
  if (!deps.signedIn()) return ONLINE_UNAVAILABLE;
  if (characters(message.sentence) > MAX_SENTENCE_CHARS) return ONLINE_UNAVAILABLE;
  try {
    const target = await deps.native();
    if (target === message.source) return ONLINE_UNAVAILABLE;
    const response = await deps.client().translate({
      sentence: message.sentence,
      selectionStart: message.selection.start,
      selectionEnd: message.selection.end,
      sourceLanguage: message.source,
      targetLanguage: target,
    });
    const outcome = response.outcome;
    switch (outcome.case) {
      case "translated": {
        const sentence = outcome.value.translation;
        const marks = outcome.value.marks
          .map(({ start, end }) => ({ start, end }))
          .filter((m) => spanWithin(m, sentence.length));
        return { kind: "translated", translation: { sentence, marks } };
      }
      case "limitReached":
        return { kind: "limit", resetsAt: Number(outcome.value.resetsAt) * 1000 };
      default:
        return ONLINE_UNAVAILABLE;
    }
  } catch (e: unknown) {
    // The error's code only: a transport error may echo what it was sent.
    (deps.log ?? LOG)("online translation failed:", (e as { code?: unknown } | null)?.code ?? "error");
    return ONLINE_UNAVAILABLE;
  }
}

// — Whether a card offers the button —

/** The account's access: its flag on, and signed in (the flag snapshot is the account's). */
export async function onlineAccess(send?: RuntimeSend): Promise<boolean> {
  const [flag, signedIn] = await Promise.all([readFlag(ONLINE_TRANSLATION_FLAG, send), syncAvailable(send)]);
  return flag && signedIn;
}

export interface OnlineOfferDeps {
  /** The setting's area (chrome.storage.local). */
  area: { get(keys: string | string[]): Promise<Record<string, unknown>> };
  /** Call `onChange` whenever one of `keys` changes in that area. */
  watch(keys: readonly string[], onChange: () => void): void;
  /** The account's access, asked of the background. */
  access(): Promise<boolean>;
}

/**
 * Whether a card offers « Traduction en ligne » right now: the setting ticked on this device AND the
 * account's access. Followed from here on — the setting through storage, the access through the flag
 * snapshot, which the background rewrites on every sign-in and clears on sign-out. Until the first
 * read lands it is off, the safe answer: no button, nothing sent.
 */
export function onlineOfferSource(deps: OnlineOfferDeps): () => boolean {
  let chosen = false;
  let access = false;
  const readChosen = async (): Promise<void> => {
    try {
      chosen = (await deps.area.get(ONLINE_TRANSLATION_KEY))[ONLINE_TRANSLATION_KEY] === true;
    } catch {
      chosen = false; // an orphaned page: its extension context is gone
    }
  };
  const readAccess = async (): Promise<void> => {
    try {
      access = await deps.access();
    } catch {
      access = false;
    }
  };
  void readChosen();
  void readAccess();
  deps.watch([ONLINE_TRANSLATION_KEY], () => void readChosen());
  deps.watch([FLAGS_KEY], () => void readAccess());
  return () => chosen && access;
}

/** The browser's: `signal`, when given, is the lifetime of the surface that asks, and the watch ends with it. */
export function createOnlineOffer(signal?: AbortSignal): () => boolean {
  return onlineOfferSource({
    area: chrome.storage.local,
    watch: (keys, onChange) => {
      if (keys.includes(FLAGS_KEY)) {
        const stop = onFlagsChanged(onChange);
        signal?.addEventListener("abort", stop, { once: true });
        return;
      }
      const listener = (changes: Record<string, chrome.storage.StorageChange>, areaName: string): void => {
        if (areaName === "local" && keys.some((k) => k in changes)) onChange();
      };
      chrome.storage.onChanged.addListener(listener);
      signal?.addEventListener("abort", () => chrome.storage.onChanged.removeListener(listener), { once: true });
    },
    access: () => onlineAccess(),
  });
}
