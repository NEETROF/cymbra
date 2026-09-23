// Which TranslatorPort a surface gets. In every shipped build the answer is none: the engine
// is not built in (__TRANSLATION_HOST__ is "none"), so the reader's surfaces behave exactly as
// they did before it existed. A development build that side-loads a model gets the messaging
// port — and only ever that one, wherever the caller runs.

import { keepEngineWarm } from "./keepalive.ts";
import { MessagingTranslatorPort } from "./messaging-port.ts";
import type { TranslatorPort } from "./port.ts";

export function createTranslatorPort(): TranslatorPort | null {
  return __TRANSLATION_HOST__ === "none" ? null : new MessagingTranslatorPort();
}

/**
 * Hold the engine's host loaded while this page is being read (`keepalive.ts`). Gated the same
 * way, and written as the same folded expression, so a shipped build carries neither the call
 * nor the module — `check:variants` fails on any trace of either.
 */
export function holdEngineWarm(): void {
  return __TRANSLATION_HOST__ === "none" ? undefined : keepEngineWarm();
}
