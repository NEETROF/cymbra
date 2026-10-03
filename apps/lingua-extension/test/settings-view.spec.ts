import { beforeEach, describe, expect, it, vi } from "vitest";
import { mountSettings, type SyncControls } from "@/reading/settings-view.ts";
import type { LinguaPort } from "@/analyzer/port.ts";
import { ANDROID_VOICES_KEY, type AsyncStorageArea, REMOTE_VOICES_KEY, VOICE_KEY } from "@/state/storage.ts";
import { createSpeaker, type SpeechSettings, type VoiceInfo } from "@/reading/speech.ts";
import type { SyncReply } from "@/sync/messages.ts";
import { makeFakePort, makeFakeSpeech, voiceFixture, type FakePort } from "./helpers.ts";

function fakeArea(): AsyncStorageArea & { store: Record<string, unknown> } {
  const store: Record<string, unknown> = {};
  return {
    store,
    async get(keys) {
      const list = keys == null ? Object.keys(store) : Array.isArray(keys) ? keys : [keys];
      const out: Record<string, unknown> = {};
      for (const k of list) if (k in store) out[k] = store[k];
      return out;
    },
    async set(items) {
      Object.assign(store, items);
    },
  };
}

const NOW = Date.UTC(2026, 8, 17, 12, 0, 0);

const samantha: VoiceInfo = {
  name: "Samantha",
  lang: "en-US",
  localService: true,
  default: false,
  voiceURI: "Samantha",
};

const opened: string[] = [];

function mount(overrides: Partial<SyncControls> = {}, port?: LinguaPort, store: AsyncStorageArea = fakeArea()) {
  const container = document.createElement("div");
  document.body.replaceChildren(container);
  const watchers: (() => void)[] = [];
  const syncNow = vi.fn(async (): Promise<SyncReply> => ({ ok: true }));
  const sync: SyncControls = {
    available: async () => true,
    syncNow,
    lastSync: async () => NOW - 3 * 60_000,
    now: () => NOW,
    watch: (onChange) => void watchers.push(onChange),
    ...overrides,
  };
  const view = mountSettings(container, port ?? makeFakePort().port, fakeArea(), {
    persist: async () => {},
    store,
    sync,
    openPage: (url) => opened.push(url),
  });
  const block = [...container.querySelectorAll<HTMLElement>(".set-block")].find((b) =>
    b.textContent?.startsWith("Synchronisation"),
  );
  if (!block) throw new Error("no Synchronisation block");
  const button = [...block.querySelectorAll("button")].find((b) => b.textContent === "Synchroniser maintenant");
  if (!button) throw new Error("no sync button");
  return { view, container, block, button, syncNow, notify: () => watchers.forEach((w) => w()) };
}

/** Let the view's floating promises settle. */
const settle = (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 0));

describe("Réglages — Synchronisation", () => {
  beforeEach(() => {
    document.body.replaceChildren();
    opened.length = 0;
  });

  it("shows when this device last synced", async () => {
    const s = mount();
    await settle();

    expect(s.block.hidden).toBe(false);
    expect(s.block.textContent).toContain("Synchronisé il y a 3 min.");
  });

  it("offers to start again from the server, instead of a local wipe the sync undoes", async () => {
    // Signed in, emptying this device erases nothing: the exchange pulls it all back. The
    // action is offered for what it does, and a real erasure is pointed at the account.
    const syncNow = vi.fn(async (): Promise<SyncReply> => ({ ok: true }));
    const s = mount({ syncNow });
    await settle();

    const restart = [...s.block.parentElement!.querySelectorAll("button")].find(
      (b) => b.textContent === "Repartir du serveur",
    );
    const wipe = [...s.block.parentElement!.querySelectorAll("button")].find((b) => b.textContent === "Réinitialiser…");
    expect(restart?.closest("div")?.hidden).toBe(false);
    expect(wipe?.closest("div")?.hidden).toBe(true); // the local wipe is not offered

    restart?.click();
    await settle();

    expect(syncNow).toHaveBeenCalledTimes(1); // emptied, then pulled back
    expect(s.block.parentElement?.textContent).toContain("Repris depuis le serveur.");
  });

  it("keeps the local reset for a reader with no account", async () => {
    const s = mount({ available: async () => false });
    await settle();

    const buttons = [...s.block.parentElement!.querySelectorAll("button")].map((b) => b.textContent);
    expect(buttons).toContain("Réinitialiser…");
    const restart = [...s.block.parentElement!.querySelectorAll("button")].find(
      (b) => b.textContent === "Repartir du serveur",
    );
    expect(restart?.closest("div")?.hidden).toBe(true);
  });

  it("opens the account page through the background, which the drawer cannot do itself", async () => {
    // In the drawer this code runs in the visited page, where `chrome.tabs` does not exist:
    // the link did nothing at all (dogfooding, build 100/101).
    const s = mount();
    await settle();

    const link = [...s.block.parentElement!.querySelectorAll("button")].find(
      (b) => b.textContent === "Gérer mes données",
    );
    link?.click();

    expect(opened).toEqual(["account.html#data"]);
  });

  it("stays hidden while signed out", async () => {
    const s = mount({ available: async () => false });
    await settle();

    expect(s.block.hidden).toBe(true);
  });

  it("syncs on demand and refreshes the label", async () => {
    let last = NOW - 3 * 60_000;
    const syncNow = vi.fn(async (): Promise<SyncReply> => {
      last = NOW;
      return { ok: true };
    });
    const s = mount({ lastSync: async () => last, syncNow });
    await settle();

    s.button.click();
    await settle();

    expect(syncNow).toHaveBeenCalledTimes(1);
    expect(s.block.textContent).toContain("Synchronisé à l'instant.");
  });

  it("explains a failure by category", async () => {
    const s = mount({ syncNow: async () => ({ ok: false, error: "unavailable" }) });
    await settle();

    s.button.click();
    await settle();

    expect(s.block.textContent).toContain("Serveur injoignable");
    expect(s.button.disabled).toBe(false);
  });

  it("follows a sync that finished in the background", async () => {
    let last = NOW - 3 * 60_000;
    const s = mount({ lastSync: async () => last });
    await settle();

    last = NOW;
    s.notify();
    await settle();

    expect(s.block.textContent).toContain("Synchronisé à l'instant.");
  });
});

describe("Réglages — Réinitialisation", () => {
  beforeEach(() => {
    document.body.replaceChildren();
    opened.length = 0;
  });

  /** The sync cursor keys, spelled out: sync.ts keeps them private, and what a reset has
   *  to clear is the stored KEY, not a symbol. */
  const STATUS_CURSOR_KEY = "cymbra-lingua-status-cursor";
  const CARD_CURSOR_KEY = "cymbra-lingua-card-cursor";

  /** Signed out — the local-only reset flow. Signed in, the block offers "Repartir du
   *  serveur" instead (covered above), because a local wipe erases nothing. */
  const localOnly = { available: async () => false };

  /** A port whose destructive calls are observable. */
  function spyPort(over: Partial<FakePort> = {}) {
    const base = makeFakePort().port;
    return {
      ...base,
      reset: vi.fn(base.reset),
      resetStatuses: vi.fn(base.resetStatuses),
      setCalibration: vi.fn(base.setCalibration),
      setDeclaredLevelAt: vi.fn(base.setDeclaredLevelAt),
      ...over,
    };
  }

  /** The button carrying exactly `label`, anywhere in the mounted settings. */
  function button(container: HTMLElement, label: string): HTMLButtonElement {
    const found = [...container.querySelectorAll("button")].find((b) => b.textContent === label);
    if (!found) throw new Error(`no button "${label}"`);
    return found;
  }

  const menuOf = (container: HTMLElement) => button(container, "Réinitialiser…").parentElement!;

  it("keeps the destructive choices behind a first click", async () => {
    const { container } = mount();
    await settle();

    expect(button(container, "Réinitialiser…").hidden).toBe(false);
    expect(button(container, "Complète — tout effacer").closest("div")!.hidden).toBe(true);
  });

  it("offers the two scopes once opened, and takes them back on cancel", async () => {
    const { container } = mount();
    await settle();
    const reset = button(container, "Réinitialiser…");

    reset.click();
    expect(reset.hidden).toBe(true);
    expect(button(container, "Complète — tout effacer").closest("div")!.hidden).toBe(false);

    button(container, "Annuler").click();
    expect(reset.hidden).toBe(false);
    expect(button(container, "Complète — tout effacer").closest("div")!.hidden).toBe(true);
  });

  it("runs a partial reset on one click: statuses and calibration, deck kept", async () => {
    const port = spyPort();
    const { container } = mount(localOnly, port);
    await settle();

    button(container, "Réinitialiser…").click();
    button(container, "Partielle — statuts + calibration (garde le deck)").click();
    await settle();

    expect(port.resetStatuses).toHaveBeenCalledOnce();
    expect(port.reset).not.toHaveBeenCalled(); // the deck survives a partial reset
    expect(menuOf(container).parentElement!.textContent).toContain("Statuts et calibration réinitialisés.");
  });

  it("asks a second time before a full wipe, and destroys nothing until then", async () => {
    const port = spyPort();
    const { container } = mount(localOnly, port);
    await settle();

    button(container, "Réinitialiser…").click();
    button(container, "Complète — tout effacer").click();
    await settle();

    expect(port.reset).not.toHaveBeenCalled();
    expect(container.textContent).toContain("Effacer statuts, deck de révision et progression ?");
    expect(button(container, "Oui, confirmer").closest("div")!.hidden).toBe(false);
  });

  it("backs out of the confirmed wipe without touching anything", async () => {
    const port = spyPort();
    const { container } = mount(localOnly, port);
    await settle();

    button(container, "Réinitialiser…").click();
    button(container, "Complète — tout effacer").click();
    const yes = button(container, "Oui, confirmer");
    [...container.querySelectorAll("button")]
      .find((b) => b.textContent === "Annuler" && !b.closest("div")!.hidden)!
      .click();
    await settle();

    expect(port.reset).not.toHaveBeenCalled();
    expect(yes.closest("div")!.hidden).toBe(true);
    expect(button(container, "Réinitialiser…").hidden).toBe(false);
  });

  it("clears the sync cursors along with the data, so a later sign-in pulls everything back", async () => {
    // Wiping locally while the cursors say "already sent" would leave the server holding
    // state this device can never see again.
    const store = fakeArea();
    await store.set({ [STATUS_CURSOR_KEY]: 42, [CARD_CURSOR_KEY]: 17 });
    const port = spyPort();
    const { container } = mount(localOnly, port, store);
    await settle();

    button(container, "Réinitialiser…").click();
    button(container, "Complète — tout effacer").click();
    button(container, "Oui, confirmer").click();
    await settle();

    expect(port.reset).toHaveBeenCalledOnce();
    expect(await store.get([STATUS_CURSOR_KEY, CARD_CURSOR_KEY])).toEqual({
      [STATUS_CURSOR_KEY]: 0,
      [CARD_CURSOR_KEY]: 0,
    });
    expect(container.textContent).toContain("Données effacées.");
  });

  it("leaves the calibration slider out of the way when the pack has levels", async () => {
    // With a level declared the reader is placed by CEFR, not by a frequency threshold:
    // 0 disables the manual cut-off. Without levels it falls back to the default.
    const withLevels = spyPort({ hasLevels: async () => true });
    const a = mount(localOnly, withLevels);
    await settle();
    button(a.container, "Réinitialiser…").click();
    button(a.container, "Partielle — statuts + calibration (garde le deck)").click();
    await settle();
    expect(withLevels.setCalibration).toHaveBeenCalledWith(0);

    const noLevels = spyPort({ hasLevels: async () => false });
    const b = mount(localOnly, noLevels);
    await settle();
    button(b.container, "Réinitialiser…").click();
    button(b.container, "Partielle — statuts + calibration (garde le deck)").click();
    await settle();
    expect(noLevels.setCalibration).toHaveBeenCalledWith(3000);
  });
});

describe("Réglages — Niveau d'anglais", () => {
  beforeEach(() => {
    document.body.replaceChildren();
    opened.length = 0;
  });

  it("hands a chosen level to the engine and drops the manual calibration with it", async () => {
    const base = makeFakePort().port;
    const port = {
      ...base,
      setDeclaredLevelAt: vi.fn(base.setDeclaredLevelAt),
      setCalibration: vi.fn(base.setCalibration),
    };
    const { container } = mount({}, port);
    await settle();

    const b1 = [...container.querySelectorAll("button")].find((b) => b.textContent === "B1");
    expect(b1).toBeDefined();
    b1!.click();
    await settle();

    expect(port.setDeclaredLevelAt).toHaveBeenCalledWith("B1", expect.any(Number));
    expect(port.setCalibration).toHaveBeenCalledWith(0);
  });
});

describe("Réglages — liens sortants", () => {
  beforeEach(() => {
    document.body.replaceChildren();
    opened.length = 0;
  });

  it("sends the reader to the browser's own shortcut editor", async () => {
    // A content script cannot open chrome:// itself — the background does it (open-page.ts).
    const { container } = mount();
    await settle();
    const config = [...container.querySelectorAll("button")].find(
      (b) => b.textContent === "Configurer les raccourcis du navigateur",
    );
    expect(config).toBeDefined();
    config!.click();

    expect(opened).toEqual(["chrome://extensions/shortcuts"]);
  });

  it("points a real erasure at the account, not at the local wipe", async () => {
    const { container } = mount();
    await settle();
    [...container.querySelectorAll("button")].find((b) => b.textContent === "Gérer mes données")!.click();

    expect(opened).toEqual(["account.html#data"]);
  });
});

describe("Réglages — Lecture à voix haute", () => {
  function mountVoices(voices: VoiceInfo[], initial: Partial<SpeechSettings> = {}) {
    const fake = makeFakeSpeech(voices, initial);
    const speaker = createSpeaker(fake.engine, "en", fake.preference);
    const container = document.createElement("div");
    document.body.replaceChildren(container);
    const area = fakeArea();
    mountSettings(container, makeFakePort().port, area, {
      persist: async () => {},
      store: fakeArea(),
      sync: {
        available: async () => false,
        syncNow: async () => ({ ok: true }),
        lastSync: async () => null,
        now: () => NOW,
        watch: () => {},
      },
      openPage: () => {},
      speaker,
    });
    const block = [...container.querySelectorAll<HTMLElement>(".set-block")].find((b) =>
      b.textContent?.startsWith("Lecture à voix haute"),
    );
    if (!block) throw new Error("no read-aloud block");
    const select = block.querySelector("select") as HTMLSelectElement;
    const preview = [...block.querySelectorAll("button")][0] as HTMLButtonElement;
    return { fake, speaker, area, block, select, preview };
  }

  const ordinaryLabels = (select: HTMLSelectElement): string[] =>
    [...select.children].filter((c) => c.tagName === "OPTION").map((o) => o.textContent ?? "");

  it("lists the automatic choice, the ordinary voices, then the others apart at the bottom", async () => {
    const s = mountVoices(voiceFixture("chrome-macos"));
    await settle();
    expect(s.block.hidden).toBe(false);
    expect(ordinaryLabels(s.select)).toEqual([
      "Automatique (Daniel)",
      "Samantha — États-Unis",
      "Daniel — Royaume-Uni",
      "Karen — Australie",
      "Moira — Irlande",
      "Rishi — Inde",
      "Tessa — Afrique du Sud",
    ]);
    const group = s.select.querySelector("optgroup")!;
    expect(group.label).toBe("Autres voix");
    expect(group.children).toHaveLength(35);
    expect(s.select.lastElementChild).toBe(group);
    expect(s.select.value).toBe("");
  });

  it("has no Autres voix group when every eligible voice is ordinary", async () => {
    const s = mountVoices([samantha]);
    await settle();
    expect(s.select.querySelector("optgroup")).toBeNull();
  });

  const noVoiceNote = (block: HTMLElement) =>
    [...block.querySelectorAll<HTMLElement>(".set-note")].find((n) => n.textContent?.startsWith("Aucune voix"))!;

  it("is absent while the browser lists no voice at all, and appears when one is announced", async () => {
    const s = mountVoices([]);
    await settle();
    expect(s.block.hidden).toBe(true);
    s.fake.list([samantha]);
    expect(s.block.hidden).toBe(false);
    expect(noVoiceNote(s.block).hidden).toBe(true);
  });

  it("on a French Windows, says no English voice is installed instead of offering Google's", async () => {
    const s = mountVoices(voiceFixture("chrome-windows"));
    await settle();
    const p = androidParts(s.block);
    expect(s.block.hidden).toBe(false);
    expect(noVoiceNote(s.block).hidden).toBe(false);
    expect(p.voiceRow.hidden).toBe(true);
    expect(p.onDeviceNote.hidden).toBe(true);
    expect(p.row.hidden).toBe(true);
    s.fake.list([...voiceFixture("chrome-windows"), samantha]);
    expect(noVoiceNote(s.block).hidden).toBe(true);
    expect(p.voiceRow.hidden).toBe(false);
  });

  const remoteParts = (block: HTMLElement) => {
    const toggle = [...block.querySelectorAll<HTMLInputElement>('input[type="checkbox"]')].find((t) =>
      t.closest("label")?.textContent?.includes("voix en ligne"),
    )!;
    return {
      toggle,
      row: toggle.closest("label")!,
      note: [...block.querySelectorAll<HTMLElement>(".set-note")].find((n) => n.textContent?.startsWith("En secours"))!,
      info: block.querySelector<HTMLElement>(".set-info")!,
    };
  };

  it("on a French Windows, offers the remote voices as a stand-in, off, with how to install one", async () => {
    const s = mountVoices(voiceFixture("chrome-windows"));
    await settle();
    const r = remoteParts(s.block);
    expect(r.row.hidden).toBe(false);
    expect(r.toggle.checked).toBe(false);
    expect(r.note.hidden).toBe(false);
    expect(r.info.title).toContain("Paramètres › Heure et langue › Langue et région › Ajouter une langue");
    expect(r.info.title).toContain("sans la définir comme langue d'affichage");
    expect(r.info.title).toMatch(/active les voix en ligne ci-dessous\.$/);
    expect(r.info.getAttribute("aria-label")).toBe(r.info.title);
    r.toggle.checked = true;
    r.toggle.dispatchEvent(new Event("change"));
    await settle();
    expect(s.area.store[REMOTE_VOICES_KEY]).toBe(true);
  });

  it("once remote voices are allowed, lists them — and never claims the text stays on the device", async () => {
    const s = mountVoices(voiceFixture("chrome-windows"), { remoteVoices: true });
    await settle();
    const p = androidParts(s.block);
    expect(remoteParts(s.block).toggle.checked).toBe(true);
    expect(noVoiceNote(s.block).hidden).toBe(false);
    expect(p.voiceRow.hidden).toBe(false);
    expect(p.onDeviceNote.hidden).toBe(true);
    expect(ordinaryLabels(s.select)).toEqual([
      "Automatique (Google US English)",
      "Google US English — États-Unis",
      "Google UK English Female — Royaume-Uni",
      "Google UK English Male — Royaume-Uni",
    ]);
  });

  it("points to the online voices from the tooltip only where they can stand in", async () => {
    const s = mountVoices([
      { name: "Microsoft Hortense", lang: "fr-FR", localService: true, default: true, voiceURI: "Hortense" },
    ]);
    await settle();
    const info = s.block.querySelector<HTMLElement>(".set-info")!;
    expect(noVoiceNote(s.block).hidden).toBe(false);
    expect(remoteParts(s.block).row.hidden).toBe(true);
    expect(info.title).toContain("Langue et région");
    expect(info.title).not.toContain("voix en ligne");
  });

  it("offers no remote stand-in where a voice is on the device", async () => {
    const s = mountVoices(voiceFixture("chrome-macos"), { remoteVoices: true });
    await settle();
    expect(remoteParts(s.block).row.hidden).toBe(true);
    expect(remoteParts(s.block).note.hidden).toBe(true);
    expect(androidParts(s.block).onDeviceNote.hidden).toBe(false);
  });

  it("keeps the chosen voice, and the automatic choice as no voice at all", async () => {
    const s = mountVoices(voiceFixture("chrome-macos"));
    await settle();
    s.select.value = "Moira";
    s.select.dispatchEvent(new Event("change"));
    await settle();
    expect(s.area.store[VOICE_KEY]).toEqual({ en: "Moira" }); // for the language the speaker reads
    s.select.value = "";
    s.select.dispatchEvent(new Event("change"));
    await settle();
    expect(s.area.store[VOICE_KEY]).toEqual({});
  });

  it("shows the stored choice, and the automatic one when that voice is gone", async () => {
    const kept = mountVoices(voiceFixture("chrome-macos"), { voices: { en: "Moira" } });
    await settle();
    expect(kept.select.value).toBe("Moira");
    const gone = mountVoices(voiceFixture("chrome-macos"), { voices: { en: "Ava (Premium)" } });
    await settle();
    expect(gone.select.value).toBe("");
  });

  const androidParts = (block: HTMLElement) => {
    const toggle = block.querySelector<HTMLInputElement>('input[type="checkbox"]')!;
    const notes = [...block.querySelectorAll<HTMLElement>(".set-note")];
    return {
      toggle,
      row: toggle.closest("label")!,
      voiceRow: block.querySelector<HTMLElement>(".set-voice")!,
      androidNote: notes.find((n) => n.textContent?.includes("voix d'Android"))!,
      onDeviceNote: notes.find((n) => n.textContent?.startsWith("Voix installées"))!,
    };
  };

  it("on Firefox for Android, offers Android's voices behind a switch, off, saying why", async () => {
    const s = mountVoices(voiceFixture("firefox-android"));
    await settle();
    const p = androidParts(s.block);
    expect(s.block.hidden).toBe(false);
    expect(p.row.hidden).toBe(false);
    expect(p.toggle.checked).toBe(false);
    expect(p.androidNote.hidden).toBe(false);
    expect(p.voiceRow.hidden).toBe(true); // nothing to choose from yet
    expect(p.onDeviceNote.hidden).toBe(true);
    p.toggle.checked = true;
    p.toggle.dispatchEvent(new Event("change"));
    await settle();
    expect(s.area.store[ANDROID_VOICES_KEY]).toBe(true);
  });

  it("once Android's voices are allowed, lists them — and no longer promises they stay on the device", async () => {
    const s = mountVoices(voiceFixture("firefox-android"), { androidVoices: true });
    await settle();
    const p = androidParts(s.block);
    expect(p.toggle.checked).toBe(true);
    expect(p.voiceRow.hidden).toBe(false);
    expect(p.onDeviceNote.hidden).toBe(true);
    expect(ordinaryLabels(s.select)).toEqual([
      "Automatique (anglais (USA,DEFAULT))",
      "anglais (USA,DEFAULT) — États-Unis",
      "anglais (USA,f00) — États-Unis",
      "anglais (GBR,DEFAULT) — Royaume-Uni",
      "anglais (GBR,f00) — Royaume-Uni",
    ]);
  });

  it("shows no Android switch where the browser offers no Android voice", async () => {
    const s = mountVoices(voiceFixture("chrome-macos"));
    await settle();
    const p = androidParts(s.block);
    expect(p.row.hidden).toBe(true);
    expect(p.androidNote.hidden).toBe(true);
    expect(p.onDeviceNote.hidden).toBe(false);
  });

  it("previews the selected voice, then stops", async () => {
    const s = mountVoices(voiceFixture("chrome-macos"));
    await settle();
    s.preview.click();
    expect(s.fake.spoken[0].voice.name).toBe("Daniel"); // the automatic choice
    expect(s.preview.textContent).toBe("■ Arrêter");
    s.preview.click();
    expect(s.speaker.speaking()).toBeNull();
    expect(s.preview.textContent).toBe("▶ Écouter");
    s.select.value = "Moira";
    s.preview.click();
    expect(s.fake.spoken[1].voice.name).toBe("Moira");
  });
});

describe("Réglages and the studied language", () => {
  it("asks every language-bound question in English", async () => {
    const { port, calls } = makeFakePort();
    mount({}, port);
    await vi.waitFor(() => expect(calls.languages.length).toBeGreaterThan(0));
    expect(new Set(calls.languages)).toEqual(new Set(["en"]));
  });
});

describe("Réglages — Affichage", () => {
  it("holds the text size and the theme, for the books and the whole interface", async () => {
    const { container } = mount();
    await settle();
    const blocks = [...container.querySelectorAll<HTMLElement>(".set-block")];
    const titled = (title: string): HTMLElement | undefined => blocks.find((b) => b.textContent?.startsWith(title));
    const display = titled("Affichage");
    expect(display?.textContent).toContain("Taille du texte");
    expect(display?.textContent).toContain("Thème");
    expect(display?.textContent).toContain("toute l'interface");
    // Shown once: the Livres block keeps the library and the flow only.
    expect(titled("Livres")?.textContent).not.toContain("Taille du texte");
    // Right before the colours, which it works with.
    expect(blocks.indexOf(display!)).toBe(blocks.indexOf(titled("Couleurs")!) - 1);
  });
});

describe("Réglages — the reader's languages (add-lingua-language-choice)", () => {
  async function mountWith(
    studied: ("en" | "es")[],
    pairs: string[],
    estimated: (language: string) => boolean = () => false,
  ) {
    const { port, calls } = makeFakePort();
    await port.setStudiedLanguages(studied);
    port.hasLevels = async () => true;
    port.levelsEstimated = async function (this: { language: string }) {
      return estimated(this.language);
    };
    const container = document.createElement("div");
    document.body.replaceChildren(container);
    mountSettings(container, port, fakeArea(), {
      persist: async () => {},
      store: fakeArea(),
      pairs,
      sync: {
        available: async () => false,
        syncNow: async () => ({ ok: true }),
        lastSync: async () => null,
        now: () => NOW,
        watch: () => {},
      },
    });
    await vi.waitFor(() => expect(container.querySelectorAll(".set-levels .set-block").length).toBeGreaterThan(0));
    await settle();
    const titles = [...container.querySelectorAll<HTMLElement>(".set-block > .set-label")].map((l) => l.textContent);
    const block = (title: string) =>
      [...container.querySelectorAll<HTMLElement>(".set-block")].find(
        (b) => b.querySelector(".set-label")?.textContent === title,
      );
    return { container, titles, block, port, calls };
  }

  it("offers the languages and a level block for each, with two shipped", async () => {
    const s = await mountWith(["en", "es"], ["en-fr", "es-fr"]);
    expect(s.block("Langues étudiées")?.hidden).toBe(false);
    expect(s.titles).toContain("Niveau d'anglais");
    expect(s.titles).toContain("Niveau d'espagnol");
    // Each level, like the choice of languages, sits under the Langue tab.
    for (const title of ["Langues étudiées", "Niveau d'anglais", "Niveau d'espagnol"]) {
      expect(s.block(title)?.closest<HTMLElement>('[role="tabpanel"]')?.dataset.tab).toBe("language");
    }
    // A level chosen in the Spanish block is Spanish.
    const spanish = s.block("Niveau d'espagnol")!;
    const declared: [string, string | null][] = [];
    s.port.setDeclaredLevelAt = async function (this: { language: string }, level) {
      declared.push([this.language, level]);
    };
    spanish.querySelector<HTMLButtonElement>('button[data-lvl="A2"]')!.click();
    await settle();
    expect(declared).toEqual([["es", "A2"]]);
  });

  it("titles estimated levels as such and says why, for that language only (add-lingua-spanish-levels)", async () => {
    const s = await mountWith(["en", "es"], ["en-fr", "es-fr"], (language) => language === "es");
    const spanish = s.block("Niveau d'espagnol estimé")!;
    const note = spanish.querySelector<HTMLElement>(".set-estimate")!;
    expect(note.hidden).toBe(false);
    expect(note.textContent).toBe(
      "Niveaux estimés d'après la fréquence des mots, faute de liste CEFR libre de droits pour l'espagnol.",
    );
    const english = s.block("Niveau d'anglais")!;
    expect(english.querySelector<HTMLElement>(".set-estimate")!.hidden).toBe(true);
  });

  it("shows one block, « Niveau d'anglais », and no choice, with en-fr alone", async () => {
    const s = await mountWith(["en"], ["en-fr"]);
    expect(s.block("Langues étudiées")?.hidden).toBe(true);
    expect(s.titles.filter((t) => t?.startsWith("Niveau"))).toEqual(["Niveau d'anglais"]);
  });
});

describe("Réglages — sub-tabs", () => {
  const titles = (panel: Element): string[] =>
    [...panel.querySelectorAll(".set-block > .set-label")].map((l) => l.textContent ?? "");
  const tabs = (container: HTMLElement): HTMLButtonElement[] => [
    ...container.querySelectorAll<HTMLButtonElement>('[role="tab"]'),
  ];
  const shown = (container: HTMLElement): HTMLElement[] => [
    ...container.querySelectorAll<HTMLElement>('[role="tabpanel"]:not([hidden])'),
  ];

  it("groups the blocks by specialisation, each tab over its own panel", async () => {
    const { container } = mount();
    await settle();
    expect(tabs(container).map((t) => t.textContent)).toEqual(["Langue", "Apparence", "Pages & livres", "Données"]);
    const panels = [...container.querySelectorAll<HTMLElement>('[role="tabpanel"]')];
    expect(panels.map(titles)).toEqual([
      ["Langues étudiées", "Niveau d'anglais", "Traduction", "Lecture à voix haute"],
      ["Affichage", "Couleurs"],
      ["Barre sur la page", "Livres", "Raccourcis & gestes"],
      ["Synchronisation", "Réinitialisation"],
    ]);
    // The book's size and theme come with Affichage, under Apparence; its page turn joins the
    // continuous flow in Livres, under Pages & livres.
    expect(panels[1].textContent).toContain("Taille du texte");
    expect(panels[1].textContent).toContain("Thème");
    expect(panels[1].textContent).not.toContain("Tourne des pages");
    const books = [...panels[2].querySelectorAll<HTMLElement>(".set-block")].find(
      (b) => b.querySelector(".set-label")?.textContent === "Livres",
    )!;
    expect(books.textContent).toMatch(/Défilement continu.*Tourne des pages.*Directe.*Glissée/);
    // Every block lives in a tab: none is left loose above the tabs.
    expect(container.querySelectorAll(":scope > .set-block")).toHaveLength(0);
    for (const [i, tab] of tabs(container).entries()) {
      expect(tab.getAttribute("aria-controls")).toBe(panels[i].id);
      expect(panels[i].getAttribute("aria-labelledby")).toBe(tab.id);
    }
  });

  it("opens on Langue, and a click shows that tab's panel alone", async () => {
    const { container } = mount();
    await settle();
    expect(shown(container).map(titles)).toEqual([
      ["Langues étudiées", "Niveau d'anglais", "Traduction", "Lecture à voix haute"],
    ]);
    expect(tabs(container).map((t) => t.getAttribute("aria-selected"))).toEqual(["true", "false", "false", "false"]);

    tabs(container)[3].click();
    expect(shown(container).map(titles)).toEqual([["Synchronisation", "Réinitialisation"]]);
    expect(tabs(container)[3].classList.contains("active")).toBe(true);
    expect(tabs(container)[0].classList.contains("active")).toBe(false);
    // One tab stop: the selected tab; the arrows reach the others.
    expect(tabs(container).map((t) => t.tabIndex)).toEqual([-1, -1, -1, 0]);
  });

  it("keeps the chosen tab across a refresh", async () => {
    const { container, view } = mount();
    await settle();
    tabs(container)[1].click();
    await view.refresh();
    expect(shown(container).map(titles)).toEqual([["Affichage", "Couleurs"]]);
  });

  it("moves between tabs with the arrows, Home and End", async () => {
    const { container } = mount();
    await settle();
    const press = (key: string): void => {
      const event = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true });
      (document.activeElement ?? tabs(container)[0]).dispatchEvent(event);
    };
    const selected = (): string | undefined =>
      tabs(container).find((t) => t.ariaSelected === "true")?.textContent ?? undefined;
    tabs(container)[0].focus();
    press("ArrowLeft");
    expect(selected()).toBe("Données");
    expect(document.activeElement).toBe(tabs(container)[3]);
    press("ArrowRight");
    expect(selected()).toBe("Langue");
    press("End");
    expect(selected()).toBe("Données");
    press("Home");
    expect(selected()).toBe("Langue");
    press("ArrowRight");
    expect(selected()).toBe("Apparence");
    press("Enter"); // not a tab key: nothing moves
    expect(selected()).toBe("Apparence");
  });

  it("brings a tab forward on request (the popup's level links)", async () => {
    const { container, view } = mount();
    await settle();
    tabs(container)[2].click();
    view.show("language");
    expect(shown(container).map(titles)[0]).toContain("Niveau d'anglais");
    view.show("data");
    expect(shown(container).map(titles)[0]).toContain("Réinitialisation");
  });

  it("gives two views in one document distinct tab ids", async () => {
    const { container } = mount();
    const second = document.createElement("div");
    document.body.append(second);
    mountSettings(second, makeFakePort().port, fakeArea(), {
      persist: async () => {},
      store: fakeArea(),
      sync: {
        available: async () => false,
        syncNow: async () => ({ ok: true }),
        lastSync: async () => null,
        now: () => NOW,
        watch: () => {},
      },
      openPage: () => {},
    });
    await settle();
    expect(tabs(second)).toHaveLength(4);
    const ids = [...document.querySelectorAll("[id]")].map((n) => n.id);
    expect(ids.length).toBe(2 * 2 * tabs(container).length);
    expect(new Set(ids).size).toBe(ids.length);
  });
});
