import { beforeEach, describe, expect, it, vi } from "vitest";
import { mountSettings, type SyncControls } from "@/reading/settings-view.ts";
import type { LinguaPort } from "@/analyzer/port.ts";
import { ANDROID_VOICES_KEY, type AsyncStorageArea, VOICE_KEY } from "@/state/storage.ts";
import { createSpeaker, type SpeechSettings, type VoiceInfo } from "@/reading/speech.ts";
import type { SyncReply } from "@/sync/messages.ts";
import { makeFakePort, makeFakeSpeech, voiceFixture } from "./helpers.ts";

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
  function spyPort(over: Partial<LinguaPort> = {}) {
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

  it("is absent without an eligible voice, and appears when one is announced", async () => {
    const s = mountVoices([
      { ...samantha, name: "Google US English", voiceURI: "Google US English", localService: false },
    ]);
    await settle();
    expect(s.block.hidden).toBe(true);
    s.fake.list([samantha]);
    expect(s.block.hidden).toBe(false);
  });

  it("keeps the chosen voice, and the automatic choice as no voice at all", async () => {
    const s = mountVoices(voiceFixture("chrome-macos"));
    await settle();
    s.select.value = "Moira";
    s.select.dispatchEvent(new Event("change"));
    await settle();
    expect(s.area.store[VOICE_KEY]).toBe("Moira");
    s.select.value = "";
    s.select.dispatchEvent(new Event("change"));
    await settle();
    expect(s.area.store[VOICE_KEY]).toBeNull();
  });

  it("shows the stored choice, and the automatic one when that voice is gone", async () => {
    const kept = mountVoices(voiceFixture("chrome-macos"), { voice: "Moira" });
    await settle();
    expect(kept.select.value).toBe("Moira");
    const gone = mountVoices(voiceFixture("chrome-macos"), { voice: "Ava (Premium)" });
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
