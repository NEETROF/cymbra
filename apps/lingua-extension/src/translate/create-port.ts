// Which TranslatorPort a surface gets. In every shipped build the answer is none: the engine
// is not built in (__TRANSLATION_HOST__ is "none"), so the reader's surfaces behave exactly as
// they did before it existed. A development build that side-loads a model gets the messaging
// port — and only ever that one, wherever the caller runs — wrapped so that using it keeps the
// engine's host loaded between selections (keepalive.ts).

import { keepWarm } from "./keepalive.ts";
import { MessagingTranslatorPort } from "./messaging-port.ts";
import type { TranslatorPort } from "./port.ts";

export function createTranslatorPort(): TranslatorPort | null {
  return __TRANSLATION_HOST__ === "none" ? null : keepWarm(new MessagingTranslatorPort());
}
