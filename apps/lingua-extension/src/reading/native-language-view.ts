import { languageWithArticle, nativeLanguageName } from "../analyzer/language-labels.ts";
import { SHIPPED_PAIRS } from "../analyzer/pairs.ts";
import type { NativeLanguage, StudiedLanguage } from "../analyzer/types.ts";
import { nativeLanguage as enNativeLanguage } from "../i18n/en/native-language.ts";
import { nativeLanguage as esNativeLanguage } from "../i18n/es/native-language.ts";
import { nativeLanguage as frNativeLanguage } from "../i18n/fr/native-language.ts";
import type { InterfaceLanguage } from "../i18n/index.ts";
import {
  chooseNativeLanguage,
  type NativeLanguageReply,
  nativeChoiceOffered,
  offeredNatives,
  studiedForNative,
} from "../state/native-language.ts";

/**
 * The choice's copy (D6): the catalogue's `native-language` module. Picked by this view, not with
 * Réglages' blocks (`settings-copy.ts`), so that a bundle built without the choice carries none of it.
 */
export type NativeLanguageCopy = typeof frNativeLanguage;

// The reader's choice of native language (add-lingua-native-language-choice D4): one view, mounted
// in Réglages (« Langue », above the studied languages), in the onboarding (its first question) and
// in the popup's first run. The native languages a shipped pair is glossed in, each named in its own
// language; the current one selected; picking another says, before it is confirmed, which languages
// the reader will then study; confirming sends `lingua-native-language`, and the background rewrites
// the profile, after which every page reloads and every reading session is built anew (D2, D3). While
// one native language ships — today — there is nothing to choose: the view mounts nothing at all, so
// every host's DOM is what it was before it (M22).

/** The reader's profile as the view needs it: their native language and the languages they study. */
export interface NativeProfile {
  native: NativeLanguage;
  studied: StudiedLanguage[];
}

export interface NativeLanguageOptions {
  /** The interface language: the view's copy is its module, the consequence names languages in it. */
  language: InterfaceLanguage;
  copy: NativeLanguageCopy;
  /** The reader's profile now: from the host's port, or from the stored backup (no engine). */
  profile: () => Promise<NativeProfile>;
  /** Send the choice; the background, by default. */
  choose?: (native: NativeLanguage) => Promise<NativeLanguageReply>;
  /** After the background answered a confirmed choice: the host refreshes, or drops its call to action. */
  onChosen?: (native: NativeLanguage, reply: Extract<NativeLanguageReply, { ok: true }>) => Promise<void> | void;
  /**
   * A question of its own — the onboarding's, the popup's first run: the preset is already applied,
   * and confirming it is the reader's answer, which marks the choice as made (D4) — the button shows
   * for the current language too, until it is answered.
   */
  confirmCurrent?: boolean;
  /** The pairs the package ships: the bundle's, unless a spec offers others. */
  pairs?: readonly string[];
}

export interface NativeLanguageView {
  /** Show the stored choice (it may have changed in another context). */
  refresh(): Promise<void>;
}

const COPY: Record<InterfaceLanguage, NativeLanguageCopy> = {
  fr: frNativeLanguage,
  en: enNativeLanguage,
  es: esNativeLanguage,
};

/** The choice's copy in the interface language: the catalogue's `native-language` module (D6). */
export function nativeLanguageCopy(language: InterfaceLanguage): NativeLanguageCopy {
  return COPY[language];
}

/** Tells apart the radio groups of two views mounted in one document. */
let mounted = 0;

/**
 * Mount the choice into `container`, or nothing when fewer than two native languages ship — then
 * null, and `container` is left untouched.
 */
export function mountNativeLanguage(container: HTMLElement, opts: NativeLanguageOptions): NativeLanguageView | null {
  const pairs = opts.pairs ?? SHIPPED_PAIRS;
  if (!nativeChoiceOffered(pairs)) return null;
  const doc = container.ownerDocument;
  const { copy, language } = opts;
  const choose = opts.choose ?? ((native: NativeLanguage) => chooseNativeLanguage(native));
  const name = `cymbra-lingua-native-${++mounted}`;

  const el = <K extends keyof HTMLElementTagNameMap>(tag: K, className?: string, text?: string) => {
    const node = doc.createElement(tag);
    if (className) node.className = className;
    if (text != null) node.textContent = text;
    return node;
  };

  const row = el("div", "set-languages");
  row.setAttribute("role", "radiogroup");
  row.setAttribute("aria-label", copy.question);
  const radios = offeredNatives(pairs).map((native) => {
    const label = el("label", "set-toggle");
    const radio = el("input");
    radio.type = "radio";
    radio.name = name;
    radio.value = native;
    radio.dataset.native = native;
    // Each language in its own name, said in its own language.
    const own = el("span", undefined, nativeLanguageName(language, native));
    own.lang = native;
    label.append(radio, own);
    radio.addEventListener("change", () => {
      if (radio.checked) pick(native);
    });
    return { native, radio };
  });
  row.append(...radios.map(({ radio }) => radio.parentElement as HTMLElement));
  const note = el("div", "set-note", copy.note);
  const consequence = el("div", "set-note");
  consequence.hidden = true;
  const confirm = el("button", "set-reset", copy.confirm);
  confirm.type = "button";
  confirm.hidden = true;
  const failed = el("div", "set-warn", copy.failed);
  failed.hidden = true;
  // Its notes and its button come and go: `.set-native` keeps `hidden` hiding them (settings.css).
  container.classList.add("set-native");
  container.append(row, note, consequence, confirm, failed);

  let current: NativeProfile | null = null;
  let picked: NativeLanguage | null = null;
  /** The language shown was confirmed: the question is answered, its button goes. */
  let answered = false;

  /** Select `native`, and say what confirming it does. */
  function pick(native: NativeLanguage): void {
    picked = native;
    for (const { native: n, radio } of radios) radio.checked = n === native;
    failed.hidden = true;
    const changes = current !== null && native !== current.native;
    const studied = current ? studiedForNative(current.studied, native, pairs) : null;
    consequence.hidden = !changes || studied === null;
    consequence.textContent =
      changes && studied ? copy.studiesAfter(studied.map((l) => languageWithArticle(language, l)).join(", ")) : "";
    confirm.hidden = !(changes || (opts.confirmCurrent && !answered));
  }

  confirm.addEventListener("click", () => void submit());

  async function submit(): Promise<void> {
    if (picked === null) return;
    const native = picked;
    confirm.disabled = true;
    const reply = await choose(native);
    confirm.disabled = false;
    if (!reply.ok) {
      failed.hidden = false;
      return;
    }
    if (!reply.changed) answered = true;
    await opts.onChosen?.(native, reply);
    await refresh();
  }

  async function refresh(): Promise<void> {
    current = await opts.profile();
    pick(current.native);
  }

  return { refresh };
}

/** Where a question of its own goes, and how it is built (`mountNativeQuestion`). */
export interface NativeQuestionOptions extends NativeLanguageOptions {
  /** The element the question goes before: the onboarding's languages section, the popup's setup. */
  before: Element;
  /** Its element: a page section (the onboarding), or a block (the popup). */
  tag: "section" | "div";
  /** The question's element: the onboarding's heading, or a line (the popup). */
  heading: "h2" | "div";
  id: string;
  className?: string;
}

/**
 * The choice as a question of its own, « Je lis en… » over the view (D4): the onboarding's first
 * question, and the popup's first-run call to action, inserted before `before`. Nothing at all when
 * fewer than two native languages ship — null, and the page is left as it was.
 */
export function mountNativeQuestion(opts: NativeQuestionOptions): { el: HTMLElement; view: NativeLanguageView } | null {
  if (!nativeChoiceOffered(opts.pairs ?? SHIPPED_PAIRS)) return null;
  const doc = opts.before.ownerDocument;
  const el = doc.createElement(opts.tag);
  el.id = opts.id;
  if (opts.className) el.className = opts.className;
  const heading = doc.createElement(opts.heading);
  heading.className = "native-question";
  heading.textContent = opts.copy.question;
  el.append(heading);
  const view = mountNativeLanguage(el, opts) as NativeLanguageView;
  opts.before.before(el);
  return { el, view };
}
