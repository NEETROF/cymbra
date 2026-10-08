import { SHIPPED_PAIRS } from "../analyzer/pairs.ts";
import type { InterfaceLanguage } from "../i18n/index.ts";
import { mountNativeQuestion, nativeLanguageCopy } from "../reading/native-language-view.ts";
import type { NativeLanguage } from "../analyzer/types.ts";
import { type NativeLanguageReply, storedProfile } from "../state/native-language.ts";
import type { AsyncStorageArea } from "../state/storage.ts";

// The popup's first-run call to action for the native language (add-lingua-native-language-choice
// D4), apart from popup.ts so a test mounts it on the popup's page: Safari does not always open the
// onboarding tab, so a new install whose choice was never made is asked here — preset from the
// browser's language before the page painted (popup.ts), the question above the page's figures
// (`#setup`, `#controls`), before the level's. Confirming the preset, or another language, answers it:
// the block goes, or the page reloads in the language chosen, and the marker the answer set keeps it
// from asking again. Nothing is mounted while one native language ships — today.

export interface NativeCtaOptions {
  /** The bundle's pairs, unless a spec offers others. */
  pairs?: readonly string[];
  /** The background, unless a spec answers. */
  choose?: (native: NativeLanguage) => Promise<NativeLanguageReply>;
}

/** Mount the question in the popup's main view, before `#setup`; null when it is not offered. */
export function mountNativeCta(
  doc: Document,
  interfaceLanguage: InterfaceLanguage,
  store: AsyncStorageArea,
  opts: NativeCtaOptions = {},
): HTMLElement | null {
  const before = doc.getElementById("setup");
  if (!before) return null;
  const pairs = opts.pairs ?? SHIPPED_PAIRS;
  const question = mountNativeQuestion({
    before,
    tag: "div",
    heading: "div",
    id: "native-cta",
    className: "native-cta",
    language: interfaceLanguage,
    copy: nativeLanguageCopy(interfaceLanguage),
    profile: () => storedProfile(store, pairs),
    choose: opts.choose,
    // The preset is the language selected: confirming it is an answer too.
    confirmCurrent: true,
    // Another language: the page reloads on the change announced. The same: answered, the block goes.
    onChosen: (_native, reply) => {
      if (!reply.changed) question?.el.remove();
    },
    pairs,
  });
  void question?.view.refresh();
  return question?.el ?? null;
}
