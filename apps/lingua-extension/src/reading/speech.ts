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
// voices speak only once the reader has allowed them in Réglages, knowing why.

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
  onVoicesChanged(listener: () => void): void;
  /** Speak `text` with `voice`; `done` runs once, with null or the error code. */
  speak(text: string, voice: V, done: (error: string | null) => void): void;
  /** Stop everything queued in this frame. */
  cancel(): void;
}

/** The reader's read-aloud settings, kept on the device. */
export interface SpeechSettings {
  /** The chosen voice's `voiceURI`, or null for the automatic choice. */
  readonly voice: string | null;
  /** Whether Android's own voices may speak, though the browser cannot say they stay on the device. */
  readonly androidVoices: boolean;
}

export const DEFAULT_SPEECH_SETTINGS: SpeechSettings = { voice: null, androidVoices: false };

/** Where those settings are kept, and how a change made in another context reaches this one. */
export interface VoicePreference {
  load(): Promise<SpeechSettings>;
  watch(onChange: (settings: SpeechSettings) => void): void;
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
  /** The eligible voices, in the browser's order. */
  eligible(): VoiceInfo[];
  /** The voice the automatic choice lands on, ignoring the reader's preference. */
  automatic(): VoiceInfo | null;
  /** The reader's preference as stored — possibly a voice no longer listed. */
  preferred(): string | null;
  /** Whether the reader allowed Android's own voices. */
  androidVoices(): boolean;
  /** Whether this browser offers Android's own voices in the studied language — allowed or not. */
  offersAndroidVoices(): boolean;
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
/** Within a tier, the regions tried first, per studied language. */
const PREFERRED_REGIONS: Record<string, readonly string[]> = { en: ["us", "gb"] };
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

/** Eligible voices best first, each once: tier, then region, then the browser's order. */
export function rankVoices<V extends VoiceInfo>(voices: readonly V[], lang: string, androidVoices = false): V[] {
  return distinctVoices(voices.filter((voice) => isEligible(voice, lang, androidVoices)))
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
): V | null {
  if (preferred) {
    const chosen = voices.find((v) => v.voiceURI === preferred && isEligible(v, lang, androidVoices));
    if (chosen) return chosen;
  }
  const defaults = voices.filter((v) => v.default);
  if (defaults.length === 1 && isEligible(defaults[0], lang, androidVoices)) return defaults[0];
  return rankVoices(voices, lang, androidVoices)[0] ?? null;
}

/** The eligible voices as Réglages lists them: the ordinary ones, then the others, apart. */
export function voiceGroups<V extends VoiceInfo>(
  voices: readonly V[],
  lang: string,
  androidVoices = false,
): { ordinary: V[]; others: V[] } {
  const ranked = rankVoices(voices, lang, androidVoices);
  return { ordinary: ranked.filter((v) => !isDeprioritised(v)), others: ranked.filter(isDeprioritised) };
}

/** A voice as Réglages names it: `Samantha — États-Unis`, the region in French. */
export function voiceLabel(voice: VoiceInfo): string {
  const code = region(voice.lang)?.toUpperCase();
  if (!code) return voice.name;
  let place = code;
  try {
    place = new Intl.DisplayNames(["fr"], { type: "region" }).of(code) ?? code;
  } catch {
    // Not a region this runtime can name (or no Intl.DisplayNames): the code says enough.
  }
  return `${voice.name} — ${place}`;
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

export function createSpeaker<V extends VoiceInfo>(
  engine: SpeechEngine<V> | null,
  lang: string,
  preference: VoicePreference,
): Speaker {
  let voices: V[] = engine ? engine.voices() : [];
  let settings: SpeechSettings = DEFAULT_SPEECH_SETTINGS;
  let current: (Speaking & { token: object }) | null = null;
  const listeners = new Set<() => void>();
  const notify = (): void => {
    for (const listener of [...listeners]) listener();
  };

  engine?.onVoicesChanged(() => {
    voices = engine.voices();
    notify();
  });
  const follow = (next: SpeechSettings): void => {
    settings = next;
    notify();
  };
  void preference.load().then(follow, () => {});
  preference.watch(follow);

  const chosen = (): V | null => pickVoice(voices, lang, settings.voice, settings.androidVoices);

  return {
    lang,
    available: () => chosen() !== null,
    eligible: () => voices.filter((v) => isEligible(v, lang, settings.androidVoices)),
    automatic: () => pickVoice(voices, lang, null, settings.androidVoices),
    preferred: () => settings.voice,
    androidVoices: () => settings.androidVoices,
    offersAndroidVoices: () => voices.some((v) => isAndroidVoice(v) && isEligible(v, lang, true)),
    speaking: () => (current ? { key: current.key, text: current.text } : null),
    speak(key, text, voice) {
      const v = (voice as V | undefined) ?? chosen();
      if (!engine || !v || !isEligible(v, lang, settings.androidVoices) || !text.trim()) return;
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
      if (typeof synth.addEventListener === "function") synth.addEventListener("voiceschanged", listener);
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
