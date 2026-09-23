import { afterEach, describe, expect, it, vi } from "vitest";
import {
  browserSpeechEngine,
  createSpeaker,
  isDeprioritised,
  isEligible,
  pickVoice,
  rankVoices,
  sameSpokenText,
  type VoiceInfo,
  voiceGroups,
  voiceLabel,
} from "@/reading/speech.ts";
import { makeFakeSpeech, voiceFixture } from "./helpers.ts";

const voice = (over: Partial<VoiceInfo> & { name: string }): VoiceInfo => ({
  lang: "en-US",
  localService: true,
  default: false,
  voiceURI: over.name,
  ...over,
});

const samantha = voice({ name: "Samantha" });
const daniel = voice({ name: "Daniel", lang: "en-GB" });
const googleUs = voice({ name: "Google US English", localService: false });

/** Let the speaker's preference load settle. */
const settle = (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 0));

describe("which voice may speak", () => {
  it("refuses a voice that synthesises remotely, even as the browser's default", () => {
    expect(isEligible({ ...googleUs, default: true }, "en")).toBe(false);
    expect(pickVoice([{ ...googleUs, default: true }, samantha], "en", null)).toBe(samantha);
  });

  it("offers nothing when every voice of the studied language is remote", () => {
    expect(pickVoice([googleUs, voice({ name: "Thomas", lang: "fr-FR" })], "en", null)).toBeNull();
  });

  it("reads the language by its primary subtag, whatever its case or separator", () => {
    expect(isEligible(voice({ name: "English United States", lang: "en_US" }), "en")).toBe(true);
    expect(isEligible(voice({ name: "Moira", lang: "EN-ie" }), "en")).toBe(true);
    expect(isEligible(voice({ name: "Thomas", lang: "fr-FR" }), "en")).toBe(false);
  });

  it("refuses every Google voice of the Chrome macOS capture", () => {
    const remote = voiceFixture("chrome-macos").filter((v) => v.name.startsWith("Google"));
    expect(remote.length).toBeGreaterThan(0);
    expect(remote.some((v) => isEligible(v, "en"))).toBe(false);
  });
});

describe("the automatic choice, on the captured lists", () => {
  it("Chrome macOS: the one default voice, Daniel", () => {
    expect(pickVoice(voiceFixture("chrome-macos"), "en", null)?.name).toBe("Daniel");
  });

  it("Chrome macOS without an English default: Samantha, not Albert or Bubbles listed before her", () => {
    const voices = voiceFixture("chrome-macos").map((v) => ({ ...v, default: false }));
    expect(pickVoice(voices, "en", null)?.name).toBe("Samantha");
  });

  it("Safari marks every voice default, which says nothing: Samantha, not the first voice listed", () => {
    const voices = voiceFixture("safari-macos");
    expect(voices.every((v) => v.default)).toBe(true);
    expect(pickVoice(voices, "en", null)?.name).toBe("Samantha");
    expect(pickVoice(voiceFixture("safari-ios-simulator"), "en", null)?.name).toBe("Samantha");
  });

  it("Firefox macOS: the one default voice, Daniel", () => {
    expect(pickVoice(voiceFixture("firefox-macos"), "en", null)?.name).toBe("Daniel");
  });

  it.each(["chrome-macos", "safari-macos", "firefox-macos", "safari-ios-simulator"])(
    "%s: never a novelty, Eloquence or legacy voice while an ordinary one exists",
    (target) => {
      const voices = voiceFixture(target).map((v) => ({ ...v, default: false }));
      const picked = pickVoice(voices, "en", null);
      expect(picked).not.toBeNull();
      expect(isDeprioritised(picked!)).toBe(false);
    },
  );

  it("follows a single default voice even when it is one the ranking would put last", () => {
    const zarvox = voice({ name: "Zarvox", default: true });
    expect(pickVoice([samantha, zarvox], "en", null)).toBe(zarvox);
  });

  it("still speaks with a deprioritised voice when it is the only one", () => {
    const fred = voice({ name: "Fred" });
    expect(pickVoice([fred], "en", null)).toBe(fred);
  });

  it("prefers a voice downloaded for its quality, named by Chrome or identified by Safari", () => {
    const chromeAva = voice({ name: "Ava (Premium)" });
    expect(pickVoice([samantha, chromeAva], "en", null)).toBe(chromeAva);
    const safariAva = voice({ name: "Ava", voiceURI: "com.apple.voice.enhanced.en-US.Ava" });
    expect(pickVoice([samantha, safariAva], "en", null)).toBe(safariAva);
  });

  it("tries en-US, then en-GB, then the other regions, then the browser's order", () => {
    const karen = voice({ name: "Karen", lang: "en-AU" });
    expect(rankVoices([karen, daniel, samantha], "en")).toEqual([samantha, daniel, karen]);
    expect(rankVoices([karen, daniel], "en")).toEqual([daniel, karen]);
    const tessa = voice({ name: "Tessa", lang: "en-ZA" });
    expect(rankVoices([tessa, karen], "en")).toEqual([tessa, karen]);
    expect(rankVoices([voice({ name: "Anna", lang: "de-DE" })], "de")).toHaveLength(1);
  });
});

describe("the reader's choice", () => {
  it("speaks with the chosen voice while it is listed and eligible", () => {
    const voices = voiceFixture("chrome-macos");
    expect(pickVoice(voices, "en", "Moira")?.name).toBe("Moira");
  });

  it("falls back to the automatic choice when the chosen voice is gone", () => {
    expect(pickVoice(voiceFixture("chrome-macos"), "en", "Ava (Premium)")?.name).toBe("Daniel");
  });

  it("never follows a choice onto a remote voice", () => {
    expect(pickVoice(voiceFixture("chrome-macos"), "en", "Google US English")?.name).toBe("Daniel");
  });
});

describe("the voices apart", () => {
  it("recognises Apple's families by name, and by identifier wherever it sits", () => {
    expect(isDeprioritised(voice({ name: "Eddy (English (United States))" }))).toBe(true);
    // Safari's identifiers do not always repeat the name, and Firefox wraps them in its own URN.
    expect(
      isDeprioritised(voice({ name: "Hysterical", voiceURI: "com.apple.speech.synthesis.voice.Hysterical" })),
    ).toBe(true);
    expect(isDeprioritised(voice({ name: "X", voiceURI: "urn:moz-tts:osx:com.apple.eloquence.en-US.Flo" }))).toBe(true);
    expect(isDeprioritised(samantha)).toBe(false);
    expect(isDeprioritised(voice({ name: "Samantha (Enhanced)" }))).toBe(false);
  });

  it("groups Chrome macOS: 6 ordinary voices, Samantha first, then the 35 others", () => {
    const { ordinary, others } = voiceGroups(voiceFixture("chrome-macos"), "en");
    expect(ordinary.map((v) => v.name)).toEqual(["Samantha", "Daniel", "Karen", "Moira", "Rishi", "Tessa"]);
    expect(others).toHaveLength(35);
    expect(others.every(isDeprioritised)).toBe(true);
  });

  it("groups Firefox macOS: no novelty voice, only the four legacy ones apart", () => {
    const { ordinary, others } = voiceGroups(voiceFixture("firefox-macos"), "en");
    expect(ordinary).toHaveLength(6);
    expect(others.map((v) => v.name)).toEqual(["Fred", "Junior", "Kathy", "Ralph"]);
  });
});

describe("labels and texts", () => {
  it("names a voice with its region in French", () => {
    expect(voiceLabel(samantha)).toBe("Samantha — États-Unis");
    expect(voiceLabel(voice({ name: "Daniel", lang: "en_GB" }))).toBe("Daniel — Royaume-Uni");
    expect(voiceLabel(voice({ name: "Plain", lang: "en" }))).toBe("Plain");
    // Not a region code: said as it is, never thrown.
    expect(voiceLabel(voice({ name: "Odd", lang: "en-Latn" }))).toBe("Odd — LATN");
  });

  it("hears a sentence and its selection as the same text regardless of case, blanks and final punctuation", () => {
    expect(sameSpokenText("They seldom ship on Friday.", "they  seldom ship on friday")).toBe(true);
    expect(sameSpokenText("« Go away! »", "« Go away")).toBe(true);
    expect(sameSpokenText("They seldom ship.", "seldom")).toBe(false);
  });
});

describe("the speaker", () => {
  it("offers nothing without a synthesiser", () => {
    const s = createSpeaker(null, "en", makeFakeSpeech().preference);
    expect(s.available()).toBe(false);
    expect(s.eligible()).toEqual([]);
    s.speak("selection", "seldom");
    s.stop();
    expect(s.speaking()).toBeNull();
  });

  it("appears when the browser announces its voices late", () => {
    const fake = makeFakeSpeech([]);
    const s = createSpeaker(fake.engine, "en", fake.preference);
    const listener = vi.fn();
    s.subscribe(listener);
    expect(s.available()).toBe(false);
    fake.list([samantha]);
    expect(listener).toHaveBeenCalled();
    expect(s.available()).toBe(true);
  });

  it("names the chosen voice on the utterance, and says what it is speaking", () => {
    const fake = makeFakeSpeech([googleUs, samantha]);
    const s = createSpeaker(fake.engine, "en", fake.preference);
    s.speak("sentence", "They seldom ship on Friday.");
    expect(fake.spoken).toHaveLength(1);
    expect(fake.spoken[0].voice).toBe(samantha);
    expect(s.speaking()).toEqual({ key: "sentence", text: "They seldom ship on Friday." });
    fake.spoken[0].done(null);
    expect(s.speaking()).toBeNull();
  });

  it("switches: the previous utterance is cancelled, and its late end does not clear the new one", () => {
    const fake = makeFakeSpeech([samantha]);
    const s = createSpeaker(fake.engine, "en", fake.preference);
    s.speak("sentence", "They seldom ship on Friday.");
    s.speak("selection", "seldom");
    expect(fake.cancels()).toBe(2);
    fake.spoken[0].done("interrupted"); // Chrome reports it after the new one has started
    expect(s.speaking()).toEqual({ key: "selection", text: "seldom" });
    fake.spoken[1].done(null);
    expect(s.speaking()).toBeNull();
  });

  it("stops what it speaks, and leaves the frame alone when it speaks nothing", () => {
    const fake = makeFakeSpeech([samantha]);
    const s = createSpeaker(fake.engine, "en", fake.preference);
    s.stop();
    expect(fake.cancels()).toBe(0); // the page's own speech is not ours to cancel
    s.speak("selection", "seldom");
    s.stop();
    expect(fake.cancels()).toBe(2);
    expect(s.speaking()).toBeNull();
  });

  it("stays silent on its own cancellations and logs a real failure", () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const fake = makeFakeSpeech([samantha]);
    const s = createSpeaker(fake.engine, "en", fake.preference);
    s.speak("selection", "seldom");
    fake.spoken[0].done("canceled");
    expect(warn).not.toHaveBeenCalled();
    s.speak("selection", "seldom");
    fake.spoken[1].done("synthesis-failed");
    expect(warn).toHaveBeenCalledWith(expect.stringContaining("synthesis-failed"));
    expect(s.speaking()).toBeNull();
    warn.mockRestore();
  });

  it("follows the stored preference, and its changes from another context", async () => {
    const fake = makeFakeSpeech([samantha, daniel], "Daniel");
    const s = createSpeaker(fake.engine, "en", fake.preference);
    await settle();
    expect(s.preferred()).toBe("Daniel");
    s.speak("selection", "seldom");
    expect(fake.spoken[0].voice).toBe(daniel);
    expect(s.automatic()).toBe(samantha); // the automatic choice ignores the preference
    const listener = vi.fn();
    s.subscribe(listener);
    fake.prefer(null);
    expect(listener).toHaveBeenCalled();
    s.speak("selection", "seldom");
    expect(fake.spoken[1].voice).toBe(samantha);
  });

  it("keeps working when the preference cannot be read", async () => {
    const fake = makeFakeSpeech([samantha]);
    const s = createSpeaker(fake.engine, "en", { load: () => Promise.reject(new Error("gone")), watch: () => {} });
    await settle();
    expect(s.preferred()).toBeNull();
    expect(s.available()).toBe(true);
  });

  it("speaks with a voice passed explicitly (the settings preview), never an ineligible one, never nothing", () => {
    const fake = makeFakeSpeech([samantha, daniel, googleUs]);
    const s = createSpeaker(fake.engine, "en", fake.preference);
    s.speak("preview", "Hello.", daniel);
    expect(fake.spoken[0].voice).toBe(daniel);
    s.speak("preview", "Hello.", googleUs);
    s.speak("preview", "   ");
    expect(fake.spoken).toHaveLength(1);
  });

  it("stops calling a listener once it unsubscribed", () => {
    const fake = makeFakeSpeech([samantha]);
    const s = createSpeaker(fake.engine, "en", fake.preference);
    const listener = vi.fn();
    const unsubscribe = s.subscribe(listener);
    unsubscribe();
    s.speak("selection", "seldom");
    expect(listener).not.toHaveBeenCalled();
    expect(s.lang).toBe("en");
  });
});

describe("the browser's synthesiser behind the seam", () => {
  class FakeUtterance {
    voice: SpeechSynthesisVoice | null = null;
    lang = "";
    onend: (() => void) | null = null;
    onerror: ((e: { error?: string }) => void) | null = null;
    constructor(readonly text: string) {}
  }

  function fakeSynth(withEvents = true) {
    const utterances: FakeUtterance[] = [];
    const listeners: (() => void)[] = [];
    const synth = {
      getVoices: vi.fn(() => [samantha]),
      speak: vi.fn((u: FakeUtterance) => void utterances.push(u)),
      cancel: vi.fn(),
      ...(withEvents ? { addEventListener: (_type: string, l: () => void) => void listeners.push(l) } : {}),
    };
    return { synth: synth as unknown as SpeechSynthesis, raw: synth, utterances, listeners };
  }

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("is absent where the page has no synthesiser", () => {
    expect(browserSpeechEngine()).toBeNull(); // jsdom has none
    expect(browserSpeechEngine(fakeSynth().synth)).toBeNull(); // nor an utterance constructor
  });

  it("names the voice and its language on the utterance, and reports its end once", () => {
    vi.stubGlobal("SpeechSynthesisUtterance", FakeUtterance);
    const f = fakeSynth();
    const engine = browserSpeechEngine(f.synth)!;
    const done = vi.fn();
    engine.speak("seldom", samantha as SpeechSynthesisVoice, done);
    const u = f.utterances[0];
    expect(u.text).toBe("seldom");
    expect(u.voice).toBe(samantha);
    expect(u.lang).toBe("en-US");
    u.onend?.();
    u.onend?.();
    expect(done).toHaveBeenCalledTimes(1);
    expect(done).toHaveBeenCalledWith(null);
  });

  it("reports an error with its code", () => {
    vi.stubGlobal("SpeechSynthesisUtterance", FakeUtterance);
    const f = fakeSynth();
    const engine = browserSpeechEngine(f.synth)!;
    const done = vi.fn();
    engine.speak("seldom", samantha as SpeechSynthesisVoice, done);
    f.utterances[0].onerror?.({ error: "interrupted" });
    engine.speak("seldom", samantha as SpeechSynthesisVoice, done);
    f.utterances[1].onerror?.({});
    expect(done.mock.calls).toEqual([["interrupted"], ["unknown"]]);
  });

  it("lists, cancels and follows the voice list through the synthesiser", () => {
    vi.stubGlobal("SpeechSynthesisUtterance", FakeUtterance);
    const f = fakeSynth();
    const engine = browserSpeechEngine(f.synth)!;
    expect(engine.voices()).toEqual([samantha]);
    engine.cancel();
    expect(f.raw.cancel).toHaveBeenCalled();
    const listener = vi.fn();
    engine.onVoicesChanged(listener);
    f.listeners.forEach((l) => l());
    expect(listener).toHaveBeenCalled();
  });

  it("does not touch the page's handler when the synthesiser has no addEventListener", () => {
    vi.stubGlobal("SpeechSynthesisUtterance", FakeUtterance);
    const f = fakeSynth(false);
    const engine = browserSpeechEngine(f.synth)!;
    expect(() => engine.onVoicesChanged(() => {})).not.toThrow();
    expect((f.raw as Record<string, unknown>).onvoiceschanged).toBeUndefined();
  });
});
