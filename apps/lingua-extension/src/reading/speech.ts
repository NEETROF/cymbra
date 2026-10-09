import { settings as frSettings } from "../i18n/fr/settings.ts";
import { DEFAULT_INTERFACE_LANGUAGE, type InterfaceLanguage, regionName } from "../i18n/index.ts";
import type { SettingsModule } from "./settings-copy.ts";

// Read-aloud (add-lingua-read-aloud): which voice may speak, which one does, and one speaker
// that owns a single utterance at a time. No DOM here, and the browser's synthesiser sits behind
// the `SpeechEngine` seam, so every decision is tested in jsdom — which has no synthesiser.
//
// The rule that matters most is eligibility. Chrome's desktop voice list mixes the operating
// system's voices with Google's, which synthesise on Google's servers and say so with
// `localService: false`. Page text must never reach one of those, so a voice speaks only when the
// browser reports it on the device AND it is named explicitly on the utterance: a bare `lang`
// lets the browser choose, and Chrome may choose a remote voice.
//
// The ranking was checked against voice lists captured on real browsers
// (`test/fixtures/voices/`), which is where its surprises came from: Safari marks every voice
// as the default; macOS lists its novelty voices (`Albert`, `Bubbles`…) before `Samantha`; an
// iPhone in French names them in French (`Bulles`, `Murmure`), so only their identifier tells;
// it lists the same voice twice, in two qualities (`Daniel`, compact and super-compact); and
// Firefox for Android writes languages in three letters (`eng-GBR-default`) and reports every
// voice of Android's engine as not local — it cannot tell where that engine synthesises. Those
// voices speak only once the reader has allowed them in Réglages, knowing why. Chrome's remote
// voices are the same bargain in the open: on a Windows set to French, the only English voices
// are Google's, so they may stand in — only where no English voice is on the device, and only
// once the reader has allowed them, told that the text then leaves it.

/** What this module reads of a voice. A `SpeechSynthesisVoice` has it, and so does a captured list. */
export interface VoiceInfo {
  readonly name: string;
  readonly lang: string;
  readonly localService: boolean;
  readonly default: boolean;
  readonly voiceURI: string;
}

/** The browser's synthesiser, as the speaker drives it. */
export interface SpeechEngine<V extends VoiceInfo = VoiceInfo> {
  /** The voices listed right now (Chrome lists none until `voiceschanged`). */
  voices(): V[];
  /** Call `listener` when the voices change; returns how to stop, where the platform allows it. */
  onVoicesChanged(listener: () => void): (() => void) | void;
  /** Speak `text` with `voice`; `done` runs once, with null or the error code. */
  speak(text: string, voice: V, done: (error: string | null) => void): void;
  /** Stop everything queued in this frame. */
  cancel(): void;
}

/** The reader's read-aloud settings, kept on the device. */
export interface SpeechSettings {
  /** The voice chosen for each studied language, by `voiceURI`; absent: the automatic choice. */
  readonly voices: Readonly<Record<string, string>>;
  /** Whether Android's own voices may speak, though the browser cannot say they stay on the device. */
  readonly androidVoices: boolean;
  /** Whether remote voices may stand in where no voice of the studied language is on the device. */
  readonly remoteVoices: boolean;
}

export const DEFAULT_SPEECH_SETTINGS: SpeechSettings = { voices: {}, androidVoices: false, remoteVoices: false };

/** Where those settings are kept, and how a change made in another context reaches this one. */
export interface VoicePreference {
  load(): Promise<SpeechSettings>;
  /** Call `onChange` when the settings change in any context; returns how to stop. */
  watch(onChange: (settings: SpeechSettings) => void): (() => void) | void;
}

/** What is being spoken: the caller's key (`selection`, `sentence`, `preview`) and the text. */
export interface Speaking {
  readonly key: string;
  readonly text: string;
}

export interface Speaker {
  /** The studied language it speaks (a primary subtag). */
  readonly lang: string;
  /** Whether a voice may speak now. Without one, nothing offers to listen. */
  available(): boolean;
  /** The voices that may speak — on the device, or the allowed stand-ins — in the browser's order. */
  eligible(): VoiceInfo[];
  /** Whether the browser lists any voice at all, eligible or not (Chrome lists none at first). */
  listsVoices(): boolean;
  /** The voice the automatic choice lands on, ignoring the reader's preference. */
  automatic(): VoiceInfo | null;
  /** The reader's preference as stored — possibly a voice no longer listed. */
  preferred(): string | null;
  /** Whether the reader allowed Android's own voices. */
  androidVoices(): boolean;
  /** Whether this browser offers Android's own voices in the studied language — allowed or not. */
  offersAndroidVoices(): boolean;
  /** Whether the reader allowed remote voices to stand in. */
  remoteVoices(): boolean;
  /** Whether remote voices could stand in: some in the studied language, none on the device. */
  offersRemoteVoices(): boolean;
  speaking(): Speaking | null;
  /**
   * Stop whatever is speaking and speak `text`, with `voice` or the chosen one. Synchronous all
   * the way to the synthesiser: the click that calls it is the user activation Chrome and
   * Safari on iOS require, and anything awaited first would spend it.
   */
  speak(key: string, text: string, voice?: VoiceInfo): void;
  stop(): void;
  /** Called whenever `available`, the voices, the preference or `speaking` change. */
  subscribe(listener: () => void): () => void;
}

/**
 * Apple's novelty voices, its Eloquence voices and its legacy ones: real voices that no reader
 * should land on by default. Chrome names them (`Eddy (English (United States))`), Safari and
 * Firefox also carry an identifier whose family says it — and whose last part does not always
 * repeat the name (`Wobble` is `…voice.Deranged`). The names are English on a Mac, but an
 * iPhone translates them into its own language (`Bubbles` is `Bulles` in French): there, the
 * family is the only thing to go by.
 */
const DEPRIORITISED_NAMES = new Set([
  // novelty
  "Albert",
  "Bad News",
  "Bahh",
  "Bells",
  "Boing",
  "Bubbles",
  "Cellos",
  "Good News",
  "Jester",
  "Organ",
  "Superstar",
  "Trinoids",
  "Whisper",
  "Wobble",
  "Zarvox",
  // Eloquence
  "Eddy",
  "Flo",
  "Grandma",
  "Grandpa",
  "Reed",
  "Rocko",
  "Sandy",
  "Shelley",
  // France's Eloquence voice where every other language has Reed: Apple's
  // `com.apple.eloquence.fr-FR.Jacques`, which Chrome lists under its bare name
  // (add-lingua-french-read-aloud D2) — neither the family nor a suffixed name catches it.
  "Jacques",
  // legacy
  "Fred",
  "Junior",
  "Kathy",
  "Ralph",
]);
const DEPRIORITISED_FAMILY = /com\.apple\.(speech\.synthesis\.voice|eloquence)\./;
/** A voice the reader downloaded for its quality: Apple's identifiers, or Chrome's name suffix. */
const ENHANCED = /com\.apple\.voice\.(premium|enhanced)\.|\((premium|enhanced)\)\s*$/i;
/** Apple's qualities, worst first, as its identifiers name them. */
const QUALITIES = ["super-compact", "compact", "enhanced", "premium"];
const QUALITY = /com\.apple\.voice\.(super-compact|compact|enhanced|premium)\./;
/**
 * Within a tier, the regions tried first, per studied language: for Spanish, a voice of Spain
 * (the programme's decision D5, add-lingua-spanish-read-aloud); for French, a voice of France
 * (add-lingua-french-read-aloud D1) — macOS lists `Amélie` (fr-CA) before `Thomas` (fr-FR). The
 * other accents come after it in the browser's order.
 */
const PREFERRED_REGIONS: Record<string, readonly string[]> = { en: ["us", "gb"], es: ["es"], fr: ["fr"] };
/** Android's engine as Firefox for Android exposes it: one voice per locale, place unknown. */
const ANDROID_VOICE = /^moz-tts:android:/;
/** Firefox for Android writes ISO 639-2 languages and ISO 3166 alpha-3 regions (`eng-GBR`). */
const THREE_LETTER_LANGUAGES: Record<string, string> = {
  eng: "en",
  fra: "fr",
  fre: "fr",
  deu: "de",
  ger: "de",
  spa: "es",
  ita: "it",
  por: "pt",
  rus: "ru",
  nld: "nl",
  dut: "nl",
  jpn: "ja",
  kor: "ko",
  zho: "zh",
  chi: "zh",
};
const THREE_LETTER_REGIONS: Record<string, string> = {
  usa: "us",
  gbr: "gb",
  aus: "au",
  irl: "ie",
  ind: "in",
  zaf: "za",
  can: "ca",
  nzl: "nz",
  esp: "es",
  mex: "mx",
  arg: "ar",
  col: "co",
  chl: "cl",
  per: "pe",
  // The French-speaking regions (add-lingua-french-read-aloud D3), Canada's `can` above.
  fra: "fr",
  bel: "be",
  che: "ch",
};

function subtags(lang: string): string[] {
  return lang.toLowerCase().replace(/_/g, "-").split("-");
}

/** The voice's language as a two-letter code (`eng-GBR-default` → `en`). */
function language(lang: string): string {
  const primary = subtags(lang)[0];
  return THREE_LETTER_LANGUAGES[primary] ?? primary;
}

/** The voice's region as a two-letter code (`eng-GBR-default` → `gb`), or none. */
function region(lang: string): string | undefined {
  const second = subtags(lang)[1];
  return second ? (THREE_LETTER_REGIONS[second] ?? second) : undefined;
}

/** A voice of Android's own engine, which Firefox for Android cannot place. */
export function isAndroidVoice(voice: VoiceInfo): boolean {
  return ANDROID_VOICE.test(voice.voiceURI);
}

/**
 * A voice may speak when it speaks the studied language and the browser reports it on the
 * device — or, only once the reader allowed them, when it is Android's own (Firefox for Android
 * reports all of those as not local, for want of knowing).
 */
export function isEligible(voice: VoiceInfo, lang: string, androidVoices = false): boolean {
  if (language(voice.lang) !== lang.toLowerCase()) return false;
  return voice.localService === true || (androidVoices && isAndroidVoice(voice));
}

/** A voice of the studied language that synthesises off the device: Chrome's `Google …` voices. */
export function isRemoteVoice(voice: VoiceInfo, lang: string): boolean {
  return language(voice.lang) === lang.toLowerCase() && voice.localService !== true && !isAndroidVoice(voice);
}

/**
 * The voices that may speak: the eligible ones whenever there is one; else, once the reader
 * allowed them, the remote ones — a stand-in, never a choice next to a voice on the device.
 */
export function usableVoices<V extends VoiceInfo>(
  voices: readonly V[],
  lang: string,
  androidVoices = false,
  remoteVoices = false,
): V[] {
  const onDevice = voices.filter((voice) => isEligible(voice, lang, androidVoices));
  if (onDevice.length > 0 || !remoteVoices) return onDevice;
  return voices.filter((voice) => isRemoteVoice(voice, lang));
}

/** The name without Chrome's parenthesised suffix: `Eddy (English (United States))` → `Eddy`. */
function baseName(name: string): string {
  const at = name.indexOf("(");
  return (at < 0 ? name : name.slice(0, at)).trim();
}

/** Whether the voice is one of Apple's novelty, Eloquence or legacy voices. */
export function isDeprioritised(voice: VoiceInfo): boolean {
  return DEPRIORITISED_NAMES.has(baseName(voice.name)) || DEPRIORITISED_FAMILY.test(voice.voiceURI);
}

function tier(voice: VoiceInfo): number {
  if (isDeprioritised(voice)) return 2;
  return ENHANCED.test(voice.voiceURI) || ENHANCED.test(voice.name) ? 0 : 1;
}

function regionRank(voice: VoiceInfo, lang: string): number {
  const preferred = PREFERRED_REGIONS[lang.toLowerCase()] ?? [];
  const at = preferred.indexOf(region(voice.lang) ?? "");
  return at < 0 ? preferred.length : at;
}

/** Apple's quality of a voice, or -1 where the identifier does not say (two such voices tie). */
function quality(voice: VoiceInfo): number {
  const named = QUALITY.exec(voice.voiceURI)?.[1];
  return named ? QUALITIES.indexOf(named) : -1;
}

/**
 * One entry per voice as the reader hears it: an iPhone lists `Daniel` twice, compact and
 * super-compact, which a picker would show as two identical lines. Same name and same language
 * are one voice — the best quality is kept, in the place the browser listed the first.
 */
function distinctVoices<V extends VoiceInfo>(voices: readonly V[]): { voice: V; index: number }[] {
  const kept = new Map<string, { voice: V; index: number }>();
  voices.forEach((voice, index) => {
    const key = `${voice.name}|${subtags(voice.lang).join("-")}`;
    const seen = kept.get(key);
    if (!seen) kept.set(key, { voice, index });
    else if (quality(voice) > quality(seen.voice)) kept.set(key, { voice, index: seen.index });
  });
  return [...kept.values()];
}

/** Usable voices best first, each once: tier, then region, then the browser's order. */
export function rankVoices<V extends VoiceInfo>(
  voices: readonly V[],
  lang: string,
  androidVoices = false,
  remoteVoices = false,
): V[] {
  return distinctVoices(usableVoices(voices, lang, androidVoices, remoteVoices))
    .sort(
      (a, b) =>
        tier(a.voice) - tier(b.voice) || regionRank(a.voice, lang) - regionRank(b.voice, lang) || a.index - b.index,
    )
    .map(({ voice }) => voice);
}

/**
 * The voice that speaks: the reader's choice while it is still listed and eligible; else the
 * browser's default, only when it is the ONE voice so marked (Safari marks all of them, which
 * says nothing); else the best-ranked eligible voice; else none.
 */
export function pickVoice<V extends VoiceInfo>(
  voices: readonly V[],
  lang: string,
  preferred: string | null,
  androidVoices = false,
  remoteVoices = false,
): V | null {
  const usable = usableVoices(voices, lang, androidVoices, remoteVoices);
  if (preferred) {
    const chosen = usable.find((v) => v.voiceURI === preferred);
    if (chosen) return chosen;
  }
  const defaults = voices.filter((v) => v.default);
  if (defaults.length === 1 && usable.includes(defaults[0])) return defaults[0];
  return rankVoices(voices, lang, androidVoices, remoteVoices)[0] ?? null;
}

/** The eligible voices as Réglages lists them: the ordinary ones, then the others, apart. */
export function voiceGroups<V extends VoiceInfo>(
  voices: readonly V[],
  lang: string,
  androidVoices = false,
  remoteVoices = false,
): { ordinary: V[]; others: V[] } {
  const ranked = rankVoices(voices, lang, androidVoices, remoteVoices);
  return { ordinary: ranked.filter((v) => !isDeprioritised(v)), others: ranked.filter(isDeprioritised) };
}

/**
 * A voice as Réglages names it, in the interface language (localise-lingua-settings D4):
 * `Samantha — États-Unis` in French, `Samantha — United States` in English — the region through
 * `regionName`, the code where the runtime cannot name it, the name alone where the voice says none.
 * French when no language is given; `copy` is Réglages' module in that language, passed with it —
 * the French one when not given.
 */
export function voiceLabel(
  voice: VoiceInfo,
  language: InterfaceLanguage = DEFAULT_INTERFACE_LANGUAGE,
  copy: Pick<SettingsModule, "voiceLabel"> = frSettings,
): string {
  const code = region(voice.lang)?.toUpperCase();
  if (!code) return voice.name;
  return copy.voiceLabel(voice.name, regionName(language, code));
}

/**
 * Whether two texts would be heard as the same: the sentence button is left out when the
 * sentence is the selection. Blanks collapsed, final punctuation and case ignored.
 */
export function sameSpokenText(a: string, b: string): boolean {
  const norm = (s: string): string =>
    s
      .replace(/\s+/g, " ")
      .trim()
      .replace(/[\s.!?…;:,"'”’»«“‘)]+$/u, "")
      .toLowerCase();
  return norm(a) === norm(b);
}

/** Errors that are the speaker's own doing (it cancelled), not a failure worth logging. */
const OWN_ERRORS = new Set(["interrupted", "canceled"]);

/**
 * `language`: the studied language it speaks, or a getter read at each use, so a surface can
 * create its speaker before it knows the reader's language (add-lingua-studied-language-profile).
 * `signal`: the speaker's lifetime, when its surface can be taken down before its page goes — a
 * reading session built anew for another native language (add-lingua-native-language-choice D3):
 * once aborted, it follows neither the voices nor the settings, and nothing it listened with keeps
 * the session alive.
 */
export function createSpeaker<V extends VoiceInfo>(
  engine: SpeechEngine<V> | null,
  language: string | (() => string),
  preference: VoicePreference,
  signal?: AbortSignal,
): Speaker {
  const langOf = typeof language === "function" ? language : () => language;
  let voices: V[] = engine ? engine.voices() : [];
  let settings: SpeechSettings = DEFAULT_SPEECH_SETTINGS;
  let current: (Speaking & { token: object }) | null = null;
  const listeners = new Set<() => void>();
  const notify = (): void => {
    for (const listener of [...listeners]) listener();
  };

  const unwatchVoices = engine?.onVoicesChanged(() => {
    voices = engine.voices();
    notify();
  });
  const follow = (next: SpeechSettings): void => {
    settings = next;
    notify();
  };
  void preference.load().then(follow, () => {});
  const unwatchSettings = preference.watch(follow);
  signal?.addEventListener(
    "abort",
    () => {
      unwatchVoices?.();
      unwatchSettings?.();
      listeners.clear();
    },
    { once: true },
  );

  const usable = (): V[] => usableVoices(voices, langOf(), settings.androidVoices, settings.remoteVoices);
  const chosen = (): V | null =>
    pickVoice(voices, langOf(), settings.voices[langOf()] ?? null, settings.androidVoices, settings.remoteVoices);

  return {
    get lang() {
      return langOf();
    },
    available: () => chosen() !== null,
    eligible: usable,
    listsVoices: () => voices.length > 0,
    automatic: () => pickVoice(voices, langOf(), null, settings.androidVoices, settings.remoteVoices),
    preferred: () => settings.voices[langOf()] ?? null,
    androidVoices: () => settings.androidVoices,
    offersAndroidVoices: () => voices.some((v) => isAndroidVoice(v) && isEligible(v, langOf(), true)),
    remoteVoices: () => settings.remoteVoices,
    offersRemoteVoices: () =>
      voices.some((v) => isRemoteVoice(v, langOf())) &&
      !voices.some((v) => isEligible(v, langOf(), settings.androidVoices)),
    speaking: () => (current ? { key: current.key, text: current.text } : null),
    speak(key, text, voice) {
      const v = (voice as V | undefined) ?? chosen();
      if (!engine || !v || !usable().some((u) => u.voiceURI === v.voiceURI) || !text.trim()) return;
      engine.cancel();
      const token = {};
      current = { key, text, token };
      // `cancel` makes the previous utterance report its end later — on Chrome after this one
      // has started — so an answer only counts for the utterance still current.
      engine.speak(text, v, (error) => {
        if (current?.token !== token) return;
        current = null;
        if (error && !OWN_ERRORS.has(error)) console.warn(`[lingua] read-aloud failed: ${error}`);
        notify();
      });
      notify();
    },
    stop() {
      // Only when something of ours speaks: `cancel` empties the frame's whole queue, the
      // page's own utterances included.
      if (!current) return;
      current = null;
      engine?.cancel();
      notify();
    },
    subscribe(listener) {
      listeners.add(listener);
      return () => void listeners.delete(listener);
    },
  };
}

/** The browser's `speechSynthesis` behind the seam, or null where there is none. */
export function browserSpeechEngine(
  synth: SpeechSynthesis | undefined = globalThis.speechSynthesis,
): SpeechEngine<SpeechSynthesisVoice> | null {
  if (!synth || typeof SpeechSynthesisUtterance === "undefined") return null;
  // Chrome may collect an utterance nothing references before it ends, and then never reports
  // the end: keep the live one.
  let live: SpeechSynthesisUtterance | null = null;
  return {
    voices: () => synth.getVoices(),
    onVoicesChanged(listener) {
      // Never through `onvoiceschanged`: the page shares that object, and assigning its handler
      // would replace the page's own.
      if (typeof synth.addEventListener !== "function") return;
      synth.addEventListener("voiceschanged", listener);
      return () => synth.removeEventListener("voiceschanged", listener);
    },
    speak(text, voice, done) {
      const utterance = new SpeechSynthesisUtterance(text);
      utterance.voice = voice;
      utterance.lang = voice.lang;
      let settled = false;
      const finish = (error: string | null): void => {
        if (settled) return;
        settled = true;
        if (live === utterance) live = null;
        done(error);
      };
      utterance.onend = () => finish(null);
      utterance.onerror = (e) => finish(e.error ?? "unknown");
      live = utterance;
      synth.speak(utterance);
    },
    cancel: () => synth.cancel(),
  };
}
