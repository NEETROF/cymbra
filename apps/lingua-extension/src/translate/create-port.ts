// Which TranslatorPort a surface gets, asked at the moment it needs one, for the language of the
// sentence. None, unless the reader turned « Traduction étendue » on for this device AND a ready
// pair studies that language — every model of its route on the device (add-lingua-translation-delivery,
// generalise-lingua-translation-model-state D5, generalise-lingua-translation-routes-by-pair D3):
// off, downloading, failed, interrupted, removed, or a language whose pair's models are missing,
// every surface answers exactly as it did before the engine existed — no line saying a translation
// is on its way. The page asks in the document's language and never names a pair; the background
// forms it. A build without the engine (`__TRANSLATION_HOST__` "none") never has one.
//
// When there is one, it is the messaging port — and only ever that one, wherever the caller runs —
// wrapped so that using it keeps the engine's host loaded between selections (keepalive.ts), and
// so that the page does not ask twice for what it has already been answered (answer-memory.ts).

import { rememberAnswers } from "./answer-memory.ts";
import { keepWarm } from "./keepalive.ts";
import { MessagingTranslatorPort } from "./messaging-port.ts";
import type { TranslatorPort } from "./port.ts";
import {
  ABSENT,
  languageReady,
  loadTranslationSetting,
  MODEL_STATE_KEY,
  type SettingArea,
  TRANSLATION_HOST_KEY,
  type TranslationSetting,
} from "./setting.ts";

/**
 * The translator right now for a sentence in `language`, or null: asked per selection, since the
 * reader can change it any time.
 */
export type TranslatorSource = (language: string) => TranslatorPort | null;

export interface TranslatorSourceDeps {
  area: SettingArea;
  /** Call `onChange` whenever one of `keys` changes in storage. */
  watch(keys: readonly string[], onChange: () => void): void;
  port: () => TranslatorPort;
}

const REAL: () => TranslatorSourceDeps = () => ({
  area: chrome.storage.local,
  watch: (keys, onChange) =>
    chrome.storage.onChanged.addListener((changes, areaName) => {
      if (areaName === "local" && keys.some((k) => k in changes)) onChange();
    }),
  port: () => rememberAnswers(keepWarm(new MessagingTranslatorPort())),
});

/**
 * Follow the setting and the model from here on. Until the first read lands there is no
 * translator, which is the safe answer: a surface without one behaves as it always did.
 */
export function translatorSource(deps: TranslatorSourceDeps): TranslatorSource {
  let setting: TranslationSetting = { host: "none", state: ABSENT };
  let port: TranslatorPort | null = null;
  const read = async (): Promise<void> => {
    try {
      setting = await loadTranslationSetting(deps.area);
    } catch {
      setting = { host: "none", state: ABSENT }; // an orphaned page: its extension context is gone
    }
  };
  void read();
  deps.watch([TRANSLATION_HOST_KEY, MODEL_STATE_KEY], () => void read());
  return (language) => {
    if (!languageReady(setting.host, setting.state, language)) return null;
    port ??= deps.port();
    return port;
  };
}

export function createTranslatorPort(): TranslatorSource {
  // A ternary, not an early return: esbuild drops a folded branch's references only in this form,
  // and a build without the engine must not keep the messaging port.
  return __TRANSLATION_HOST__ === "none" ? () => null : translatorSource(REAL());
}
