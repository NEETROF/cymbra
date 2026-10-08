import { SHIPPED_PAIRS } from "../analyzer/pairs.ts";
import type { InterfaceLanguage } from "../i18n/index.ts";
import { mountNativeQuestion, nativeLanguageCopy } from "../reading/native-language-view.ts";
import {
  type PresetDeps,
  presetNativeLanguage,
  reloadOnNativeLanguageChange,
  storedProfile,
} from "../state/native-language.ts";
import type { AsyncStorageArea } from "../state/storage.ts";
import { watchStore } from "../state/store.ts";

// The onboarding's first question, « Je lis en… » (add-lingua-native-language-choice D4), apart from
// the page's wiring so a test mounts it alone (onboarding.ts starts the page when imported). A new
// install's native language is preset before the page paints — `presetThenStart` runs before the
// page's start, which reads the interface language the preset wrote — and the question, above the
// languages section, lets the reader confirm it or pick another in the same step (M3): either is
// their answer, and the popup asks no more. While one native language ships — today — the preset
// reads nothing and the page holds no question at all.

/** What the onboarding's step needs beyond its page: the defaults are the extension's. */
export interface NativeStepDeps {
  /** The bundle's pairs, unless a spec offers others. */
  pairs?: readonly string[];
  /** `watchStore`, unless a spec watches otherwise. */
  watch?: Parameters<typeof reloadOnNativeLanguageChange>[1];
  reload?: () => void;
}

/**
 * The question, before `languagesSection`, in `interfaceLanguage` — read from the stored profile, no
 * engine needed; and the page's reload when the native language changes (D3): it then reads the
 * interface language again and fills its copy in it, as on opening. Nothing is shown while one native
 * language ships.
 */
export function mountNativeStep(
  languagesSection: HTMLElement,
  interfaceLanguage: InterfaceLanguage,
  store: AsyncStorageArea,
  deps: NativeStepDeps = {},
): HTMLElement | null {
  reloadOnNativeLanguageChange(interfaceLanguage, deps.watch ?? watchStore, deps.reload);
  const pairs = deps.pairs ?? SHIPPED_PAIRS;
  const question = mountNativeQuestion({
    before: languagesSection,
    tag: "section",
    heading: "h2",
    id: "native-section",
    language: interfaceLanguage,
    copy: nativeLanguageCopy(interfaceLanguage),
    profile: () => storedProfile(store, pairs),
    // The preset is the language selected: confirming it is an answer too.
    confirmCurrent: true,
    pairs,
  });
  void question?.view.refresh();
  return question?.el ?? null;
}

/** A new install's preset (D4), then the page's start: the start always runs, whatever the preset did. */
export async function presetThenStart(
  start: () => Promise<void>,
  deps: PresetDeps = {
    preferences: { get: (keys) => chrome.storage.local.get(keys) },
    browserLanguage: navigator.language,
  },
): Promise<void> {
  await presetNativeLanguage(deps);
  await start();
}
