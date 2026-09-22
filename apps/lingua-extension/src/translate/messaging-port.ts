// The ONLY TranslatorPort a surface can obtain: it sends the request to the background, which
// owns the engine off every thread that paints. See port.ts for why there is no other.

import { type TranslationRequest, type TranslationResult, type TranslatorPort, UNAVAILABLE } from "./port.ts";
import { TRANSLATE_TYPE, type TranslateMessage } from "./wire.ts";

export type TranslateSend = (message: TranslateMessage) => Promise<unknown>;

const runtimeSend: TranslateSend = (message) => chrome.runtime.sendMessage(message);

/** A reply is either a translation or nothing; anything else is treated as nothing. */
function asResult(reply: unknown): TranslationResult {
  const r = reply as Partial<TranslationResult> | null | undefined;
  if (r?.kind === "translated" && r.translation && typeof r.translation.sentence === "string") {
    return { kind: "translated", translation: { sentence: r.translation.sentence, marks: r.translation.marks ?? [] } };
  }
  return UNAVAILABLE;
}

export class MessagingTranslatorPort implements TranslatorPort {
  constructor(private readonly send: TranslateSend = runtimeSend) {}

  async translate(request: TranslationRequest): Promise<TranslationResult> {
    try {
      return asResult(await this.send({ type: TRANSLATE_TYPE, request }));
    } catch {
      // No listener (the engine is not built in), a torn-down background, a closed port:
      // all of them mean the same thing to the reader — there is no translation right now.
      return UNAVAILABLE;
    }
  }
}
