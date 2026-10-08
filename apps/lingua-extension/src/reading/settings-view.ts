import {
  estimatedLevelsNote,
  languageName,
  levelTitle,
  noVoiceInstalled,
  previewSentence,
  windowsVoiceLanguage,
} from "../analyzer/language-labels.ts";
import { acceptedLanguages, SHIPPED_PAIRS } from "../analyzer/pairs.ts";
import type { LinguaPort } from "../analyzer/port.ts";
import { CEFR_LEVELS, type CefrLevel, type StudiedLanguage } from "../analyzer/types.ts";
import { DEFAULT_INTERFACE_LANGUAGE, fillSlots, formatCount, type InterfaceLanguage, slot } from "../i18n/index.ts";
import { needsLevelChoice } from "../state/level-choice.ts";
import { nativeChoiceOffered } from "../state/native-language.ts";
import { type OpenPage, openPageViaBackground } from "../state/open-page.ts";
import { hasShortcutEditor } from "../state/platform.ts";
import {
  type AsyncStorageArea,
  loadHudHidden,
  loadReaderFlow,
  saveAndroidVoices,
  saveHudHidden,
  saveRemoteVoices,
  saveReaderFlow,
  saveVoice,
} from "../state/storage.ts";
import {
  LAST_SYNC_KEY,
  loadLastSync,
  syncAvailable,
  syncNow as requestSyncNow,
  type SyncReply,
} from "../sync/messages.ts";
import { lastSyncLabel, syncErrorCopy } from "../sync/status.ts";
import { clearSyncCursors } from "../sync/sync.ts";
import { type AccountControls, mountAccountSetting, runtimeAccountControls } from "./account-setting.ts";
import { mountBookDisplay } from "./book-display-view.ts";
import { mountColourSettings } from "./colour-settings-view.ts";
import { type SettingsModule, settingsCopy } from "./settings-copy.ts";
import { mountNativeLanguage, nativeLanguageCopy } from "./native-language-view.ts";
import { mountStudiedLanguages } from "./studied-languages-view.ts";
import { type Speaker, type VoiceInfo, voiceGroups, voiceLabel } from "./speech.ts";
import {
  mountTranslationSetting,
  runtimeTranslationControls,
  type TranslationControls,
  type TranslationSettingView,
} from "./translation-setting.ts";

// The Réglages view, built as plain DOM into a given container so ONE implementation
// serves every host: the native side panel, the in-page drawer and the toolbar popup (same pattern as review's
// renderReview). It drives the host's own port and persists via `persist`, so every other
// surface reacts through storage.onChanged; `onReset` lets the host refresh its review
// after a wipe. It is the ONLY Réglages: the toolbar popup mounts it too, and
// `test/lint-settings-hosts.spec.ts` refuses a host that builds its own.

export interface SettingsOptions {
  /** Persist engine state after a change (backup → storage). */
  persist: () => Promise<void>;
  /** Refresh the host's review/summary after a reset (the deck may have changed). */
  onReset?: () => Promise<void> | void;
  /** The reader's data, for the reset that clears the sync cursors. */
  store: AsyncStorageArea;
  /** The Synchronisation controls' seam; the background messages by default. */
  sync?: SyncControls;
  /** Open an extension or browser page. The background does it: the drawer cannot. */
  openPage?: OpenPage;
  /** The host's speaker: the read-aloud block lists its voices, and is absent without one. */
  speaker?: Speaker;
  /**
   * « Traduction étendue »'s seam; the background by default, in a variant that carries the
   * engine. Null: no such setting (Safari).
   */
  translation?: TranslationControls | null;
  /** The Compte controls' seam; the background messages by default. */
  account?: AccountControls;
  /** Safari: a sign-in went on in the host app (the popup closes; its next open collects it). */
  onHandedOff?: () => void;
  /** The pairs the package ships: the bundle's, unless a spec offers others. */
  pairs?: readonly string[];
  /**
   * The interface language (localise-lingua-settings D1): the one the popup and the side panel read
   * with their preferences, the one the reading session handed the drawer. French when not given.
   */
  interfaceLanguage?: InterfaceLanguage;
}

/** The key the settings preview speaks under — not a card's, so no card silences it. */
const PREVIEW_KEY = "preview";
/**
 * How to install a voice of `language` on the device — a system voice, not a change of language
 * (add-lingua-language-choice: the languages named from language-labels). On Windows, through
 * « Langue et région »: « Voix › Ajouter des voix » did nothing on a French Windows 11
 * (2026-09-29), and that install can fail outright (0x800F0950), hence the pointer to the fallback
 * (`installVoiceHelpWithFallback`, where the remote voices can stand in).
 */
function installVoiceHelp(
  copy: SettingsModule,
  interfaceLanguage: InterfaceLanguage,
  language: StudiedLanguage,
): string {
  return copy.installVoiceHelp(
    windowsVoiceLanguage(interfaceLanguage, language),
    languageName(interfaceLanguage, language),
  );
}

/** What the Synchronisation block needs from the background and the store. */
export interface SyncControls {
  /** Whether a Cymbra account is signed in (the block shows only then). */
  available: () => Promise<boolean>;
  /** Run one exchange now. */
  syncNow: () => Promise<SyncReply>;
  /** This device's last successful sync (epoch millis), or null. */
  lastSync: () => Promise<number | null>;
  now: () => number;
  /** Call `onChange` whenever a sync completes elsewhere (the background); returns how to stop. */
  watch: (onChange: () => void) => (() => void) | void;
}

function runtimeSyncControls(area: AsyncStorageArea): SyncControls {
  return {
    available: () => syncAvailable(),
    syncNow: () => requestSyncNow(),
    lastSync: () => loadLastSync(area),
    now: () => Date.now(),
    watch: (onChange) => {
      const listener = (changes: Record<string, chrome.storage.StorageChange>, areaName: string): void => {
        if (areaName === "local" && changes[LAST_SYNC_KEY]) onChange();
      };
      chrome.storage.onChanged.addListener(listener);
      return () => chrome.storage.onChanged.removeListener(listener);
    },
  };
}

/** Réglages' sub-tabs, one per specialisation: the view had grown too long to scroll through. */
export type SettingsTab = "language" | "look" | "pages" | "data";

export interface SettingsView {
  /** Re-sync the controls with the engine (call each time the view is shown). */
  refresh: () => Promise<void>;
  /** Bring a sub-tab forward (an entry point that leads to one setting, the level). */
  show: (tab: SettingsTab) => void;
  /**
   * Stop watching the background and the preferences: the view's host is taken down — the drawer of
   * a reading session built anew for another native language (add-lingua-native-language-choice D3).
   * A page that reloads never calls it.
   */
  destroy: () => void;
}

/** Tells apart the tab ids of two views mounted in one document. */
let mountCount = 0;

function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  className?: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text != null) node.textContent = text;
  return node;
}

/** One level block of Réglages, for one accepted language (add-lingua-language-choice D3). */
interface LevelBlock {
  readonly language: StudiedLanguage;
  readonly el: HTMLElement;
  refresh(): Promise<void>;
}

/** A block of Réglages, under its title: the catalogue's, where the hosts' lint reads it (D2). */
function settingBlock(label: string): HTMLDivElement {
  const block = el("div", "set-block");
  block.append(el("div", "set-label", label));
  return block;
}

/** A shortcut's line: the catalogue's message, the keys rendered where the language puts them (D3). */
function shortcut(line: (keys: string) => string, ...keys: (string | HTMLElement)[]): HTMLLIElement {
  const li = el("li");
  li.append(...fillSlots(line(slot(0)), [keys]));
  return li;
}

function kbd(key: string): HTMLElement {
  return el("kbd", undefined, key);
}

/** Mount the Réglages controls into `container`. Returns a `refresh()` to re-sync state. */
export function mountSettings(
  container: HTMLElement,
  port: LinguaPort,
  area: AsyncStorageArea,
  opts: SettingsOptions,
): SettingsView {
  container.replaceChildren();
  const pairs = opts.pairs ?? SHIPPED_PAIRS;
  // The copy, in the interface language its host hands it (D1): this view's module, and each
  // block's own, handed to it with the language where the block writes a figure or a date (D4).
  const interfaceLanguage = opts.interfaceLanguage ?? DEFAULT_INTERFACE_LANGUAGE;
  const blocksCopy = settingsCopy(interfaceLanguage);
  const copy = blocksCopy.settings;

  // — Langue maternelle — above the studied languages, only when two native languages or more ship
  // (add-lingua-native-language-choice D4): until then no block is built at all, and Réglages are
  // what they were — nor is the view bundled (`__NATIVE_CHOICE__`). A choice confirmed here needs
  // nothing of this host: on the change announced, the page reloads, or the reading session is built
  // anew, its port for the new native language (D3).
  const nativeBlock = __NATIVE_CHOICE__ && nativeChoiceOffered(pairs) ? settingBlock(copy.nativeLanguage) : null;
  const native =
    __NATIVE_CHOICE__ && nativeBlock
      ? mountNativeLanguage(nativeBlock, {
          language: interfaceLanguage,
          copy: nativeLanguageCopy(interfaceLanguage),
          profile: async () => ({ native: await port.nativeLanguage(), studied: await port.studiedLanguages() }),
          onChosen: () => refresh(),
          pairs,
        })
      : null;

  // — Langues étudiées — hidden when the package ships one language (add-lingua-language-choice D2).
  const languagesBlock = settingBlock(copy.studiedLanguages);
  const studied = mountStudiedLanguages(
    languagesBlock,
    port,
    async () => {
      await opts.persist();
      await refresh();
    },
    pairs,
    blocksCopy.studiedLanguages,
    interfaceLanguage,
  );

  // — Niveau, one block per accepted language (add-lingua-language-choice D3) —
  const levelBlocks = el("div", "set-levels");
  let levels: LevelBlock[] = [];

  /** The level block of one language: its chips, hint and calibration. */
  function levelBlockFor(language: StudiedLanguage): LevelBlock {
    const view = port.for(language);
    const block = settingBlock(levelTitle(interfaceLanguage, language));
    const title = block.querySelector<HTMLElement>(".set-label");
    const chips = el("div", "level-chips");
    const chipButtons = new Map<string, HTMLButtonElement>();
    const addChip = (value: string, label: string): void => {
      const b = el("button", value === "" ? "lvl lvl-beginner" : "lvl", label);
      b.type = "button";
      b.dataset.lvl = value;
      b.addEventListener("click", () => void setLevel((value as CefrLevel) || null));
      chips.append(b);
      chipButtons.set(value, b);
    };
    for (const lvl of CEFR_LEVELS) addChip(lvl, lvl);
    addChip("", copy.beginner);
    const hint = el("div", "set-note");
    // Levels estimated from frequency say so (add-lingua-spanish-levels).
    const estimate = el("div", "set-note set-estimate", estimatedLevelsNote(interfaceLanguage, language));
    estimate.hidden = true;
    const calibBlock = el("div", "calib");
    calibBlock.hidden = true;
    // The count as the interface language writes a count: French bare, as before (« 3000 »).
    const calibValue = el("b", undefined, formatCount(interfaceLanguage, 3000));
    const calibLabel = el("label");
    calibLabel.append(...fillSlots(copy.knowCommonest(slot(0)), [calibValue]));
    const calib = el("input");
    calib.type = "range";
    calib.min = "500";
    calib.max = "10000";
    calib.step = "100";
    calib.value = "3000";
    calibBlock.append(calibLabel, calib);
    block.append(chips, hint, estimate, calibBlock);
    calib.addEventListener("input", () => {
      calibValue.textContent = formatCount(interfaceLanguage, Number(calib.value));
    });
    calib.addEventListener("change", async () => {
      await view.setCalibration(Number(calib.value));
      await opts.persist();
    });

    async function setLevel(level: CefrLevel | null): Promise<void> {
      await view.setDeclaredLevelAt(level, Date.now());
      await view.setCalibration(0);
      await opts.persist();
      await refresh();
    }

    async function refreshBlock(): Promise<void> {
      const [hasLevels, declared, needsChoice] = [
        await view.hasLevels(),
        await view.declaredLevel(),
        await needsLevelChoice(port, language),
      ];
      const estimated = hasLevels && (await view.levelsEstimated());
      if (title) title.textContent = levelTitle(interfaceLanguage, language, estimated);
      estimate.hidden = !estimated;
      // Nothing is highlighted as chosen until a decision exists (« Débutant » is one).
      const current = needsChoice ? null : (declared ?? "");
      for (const [value, b] of chipButtons) b.classList.toggle("active", value === current);
      hint.textContent = declared
        ? copy.wordsBelowLevel(declared)
        : needsChoice
          ? copy.chooseLevelHint
          : copy.beginnerHint;
      calibBlock.hidden = hasLevels;
      if (!hasLevels) {
        const cal = await view.calibration();
        calib.value = String(cal);
        calibValue.textContent = formatCount(interfaceLanguage, cal);
      }
    }

    return { language, el: block, refresh: refreshBlock };
  }

  /** The accepted languages' blocks: rebuilt when the list changed, refreshed otherwise. */
  async function refreshLevels(): Promise<void> {
    const accepted = await acceptedLanguages(port, pairs);
    if (accepted.join() !== levels.map((l) => l.language).join()) {
      levels = accepted.map(levelBlockFor);
      levelBlocks.replaceChildren(...levels.map((l) => l.el));
    }
    await Promise.all(levels.map((l) => l.refresh()));
  }

  // — Barre sur la page —
  const barBlock = settingBlock(copy.barOnPage);
  const toggleRow = el("label", "set-toggle");
  const toggle = el("input");
  toggle.type = "checkbox";
  toggleRow.append(toggle, el("span", undefined, copy.showPercentPill));
  barBlock.append(toggleRow, el("div", "set-note", copy.pillNote));

  // — Lecture à voix haute (once the browser lists its voices) —
  const speaker = opts.speaker;
  const voiceBlock = settingBlock(copy.readAloud);
  voiceBlock.hidden = true;
  const voiceRow = el("div", "set-voice");
  const voiceSelect = el("select");
  voiceSelect.setAttribute("aria-label", copy.readingVoice);
  const previewBtn = el("button", "set-reset", copy.listen);
  previewBtn.type = "button";
  voiceRow.append(voiceSelect, previewBtn);
  const onDeviceNote = el("div", "set-note", copy.onDeviceNote);
  // Firefox for Android cannot say whether Android's engine synthesises on the device, so its
  // voices speak only once the reader allows them here, told what that means.
  const androidRow = el("label", "set-toggle");
  const androidToggle = el("input");
  androidToggle.type = "checkbox";
  androidRow.append(androidToggle, el("span", undefined, copy.useAndroidVoice));
  const androidNote = el("div", "set-note", copy.androidNote);
  // A French Windows lists only French voices of its own: Chrome's English ones are Google's,
  // remote. Said here, with how to install one (the reader need not change any language for it),
  // rather than a block that silently never shows.
  const noVoiceText = el("span");
  const noVoiceNote = el("div", "set-note");
  const installInfo = el("span", "set-info", copy.infoIcon);
  installInfo.tabIndex = 0;
  installInfo.setAttribute("role", "img");
  noVoiceNote.append(noVoiceText, installInfo);
  // Where the only voices of the studied language are remote, they may stand in — never by
  // default, and saying where the text then goes.
  const remoteRow = el("label", "set-toggle");
  const remoteToggle = el("input");
  remoteToggle.type = "checkbox";
  remoteRow.append(remoteToggle, el("span", undefined, copy.useRemoteVoices));
  const remoteNote = el("div", "set-note", copy.remoteNote);
  voiceBlock.append(androidRow, androidNote, noVoiceNote, remoteRow, remoteNote, voiceRow, onDeviceNote);

  // — Livres — the reader page, whichever host this view is rendered in (add-lingua-reader D8).
  const booksBlock = settingBlock(copy.books);
  const libraryBtn = el("button", "set-reset", copy.openLibrary);
  libraryBtn.type = "button";
  libraryBtn.addEventListener("click", () => openPage("reader.html"));
  const flowRow = el("label", "set-toggle");
  const flowToggle = el("input");
  flowToggle.type = "checkbox";
  flowRow.append(flowToggle, el("span", undefined, copy.continuousFlow));
  booksBlock.append(libraryBtn, el("div", "set-note", copy.booksNote), flowRow);

  // — Affichage — the text size and the theme: the book's text, and every surface of the extension
  // (add-lingua-colour-settings D9). The same controls as the reader's own "Aa" panel, whose page
  // turn joins Livres, after the continuous flow: both say how a book's pages go by.
  const displayBlock = settingBlock(copy.display);
  const bookDisplay = mountBookDisplay(displayBlock, area, {
    turnContainer: booksBlock,
    copy: blocksCopy.display,
    language: interfaceLanguage,
  });
  displayBlock.append(el("div", "set-note", copy.displayNote));

  // — Couleurs — how unknown and learning words are marked, everywhere (add-lingua-colour-settings).
  const coloursBlock = settingBlock(copy.colours);
  const colours = mountColourSettings(coloursBlock, area, { copy: blocksCopy.colours });

  // — Traduction (the variants that carry the engine; the background says whether it is offered) —
  const translationBlock = settingBlock(copy.translation);
  const translationControls =
    opts.translation !== undefined
      ? opts.translation
      : __TRANSLATION_HOST__ !== "none"
        ? runtimeTranslationControls()
        : null;
  const translation: TranslationSettingView | null = translationControls
    ? mountTranslationSetting(translationBlock, translationControls, {
        copy: blocksCopy.translation,
        language: interfaceLanguage,
      })
    : null;
  if (!translation) translationBlock.hidden = true;

  // — Raccourcis & gestes —
  const scBlock = settingBlock(copy.shortcuts);
  const scList = el("ul", "set-shortcuts");
  const plus = copy.keyPlus;
  scList.append(
    shortcut(copy.shortcutSidePanel, kbd(copy.keyAlt), plus, kbd(copy.keyShift), plus, kbd(copy.keyS)),
    shortcut(copy.shortcutDrawer, kbd(copy.keyAlt), plus, kbd(copy.keyShift), plus, kbd(copy.keyD)),
    shortcut(copy.shortcutCapture, kbd(copy.keyAlt), plus, kbd(copy.keyL)),
    shortcut(copy.shortcutReclassify, kbd(copy.keyAlt), copy.keyOr, kbd(copy.keyOption)),
  );
  const scConfig = el("button", "linklike", copy.configureShortcuts);
  scConfig.type = "button";
  scConfig.addEventListener("click", () => {
    openPage(__TARGET__ === "firefox" ? "about:addons" : "chrome://extensions/shortcuts");
  });
  scBlock.append(scList);
  if (hasShortcutEditor()) scBlock.append(scConfig);

  // — Compte — sign in or out, wherever Réglages are shown (the popup's main view links here).
  const accountBlock = settingBlock(copy.account);
  const account = mountAccountSetting(accountBlock, opts.account ?? runtimeAccountControls(), {
    openPage: (url) => openPage(url),
    onChange: () => refresh(),
    onHandedOff: opts.onHandedOff,
    copy: blocksCopy.accountSetting,
  });

  // — Synchronisation (signed in only): when this device last synced, and a manual run —
  const sync = opts.sync ?? runtimeSyncControls(area);
  const openPage = opts.openPage ?? openPageViaBackground;
  const syncBlock = settingBlock(copy.sync);
  syncBlock.hidden = true;
  const syncStatus = el("div", "set-note");
  const syncBtn = el("button", "set-reset", copy.syncNow);
  syncBtn.type = "button";
  const syncMsg = el("div", "set-note");
  syncBlock.append(syncStatus, syncBtn, syncMsg);

  // — Réinitialisation (scope choice; a full wipe needs an extra confirm) —
  const resetBlock = settingBlock(copy.reset);
  const localNote = el("div", "set-note", copy.resetNote);
  resetBlock.append(localNote);
  const resetBtn = el("button", "set-reset", copy.resetButton);
  resetBtn.type = "button";
  const menu = el("div");
  menu.hidden = true;
  const resetPartial = el("button", "set-reset", copy.resetPartial);
  resetPartial.type = "button";
  const resetFull = el("button", "set-danger", copy.resetFull);
  resetFull.type = "button";
  const resetCancel = el("button", "set-reset", copy.cancel);
  resetCancel.type = "button";
  menu.append(resetPartial, resetFull, resetCancel);
  const confirm = el("div");
  confirm.hidden = true;
  const warn = el("div", "set-warn");
  const confirmYes = el("button", "set-danger", copy.confirm);
  confirmYes.type = "button";
  const confirmNo = el("button", "set-reset", copy.cancel);
  confirmNo.type = "button";
  confirm.append(warn, confirmYes, confirmNo);
  const resetMsg = el("div", "set-note");

  // Signed in, erasing this device erases nothing: the next exchange pulls it all back —
  // it even clears the cursors, which guarantees the return. So the same action is offered
  // for what it actually is, and a real erasure is pointed at where it lives.
  const localOnly = el("div");
  localOnly.append(resetBtn, menu, confirm);
  const synced = el("div");
  synced.hidden = true;
  const restartNote = el("div", "set-note", copy.restartNote);
  const restartBtn = el("button", "set-reset", copy.restartFromServer);
  restartBtn.type = "button";
  const eraseNote = el("div", "set-note", copy.eraseNote);
  const eraseLink = el("button", "linklike", copy.manageData);
  eraseLink.type = "button";
  eraseLink.addEventListener("click", () => openPage("account.html#data"));
  synced.append(restartNote, restartBtn, eraseNote, eraseLink);
  resetBlock.append(localOnly, synced, resetMsg);

  const showReset = (showMenu: boolean, showConfirm: boolean): void => {
    resetBtn.hidden = showMenu || showConfirm;
    menu.hidden = !showMenu;
    confirm.hidden = !showConfirm;
  };
  resetBtn.addEventListener("click", () => showReset(true, false));
  resetCancel.addEventListener("click", () => showReset(false, false));
  confirmNo.addEventListener("click", () => showReset(false, false));
  resetPartial.addEventListener("click", () => {
    showReset(false, false);
    void doReset("partial");
  });
  resetFull.addEventListener("click", () => {
    warn.textContent = copy.fullResetWarning;
    showReset(false, true);
  });
  confirmYes.addEventListener("click", () => {
    showReset(false, false);
    void doReset("full");
  });

  // — The sub-tabs: the studied language, how things look, Lingua on the pages and books read,
  // the reader's data. The blocks keep their titles; a hidden block (no voice, signed out) leaves
  // its tab with the others, never empty: each tab has one block that always shows.
  const tabs = mountTabs(container, copy.tabs, [
    {
      id: "language",
      label: copy.tabLanguage,
      blocks: [...(nativeBlock ? [nativeBlock] : []), languagesBlock, levelBlocks, translationBlock, voiceBlock],
    },
    { id: "look", label: copy.tabLook, blocks: [displayBlock, coloursBlock] },
    { id: "pages", label: copy.tabPages, blocks: [barBlock, booksBlock, scBlock] },
    { id: "data", label: copy.tabData, blocks: [accountBlock, syncBlock, resetBlock] },
  ]);

  // — Live wiring —
  toggle.addEventListener("change", async () => {
    await saveHudHidden(area, !toggle.checked);
  });
  // Kept for the language the host's speaker reads (add-lingua-language-choice D4).
  voiceSelect.addEventListener("change", () => {
    if (speaker) void saveVoice(area, speaker.lang, voiceSelect.value || null);
  });
  androidToggle.addEventListener("change", () => void saveAndroidVoices(area, androidToggle.checked));
  remoteToggle.addEventListener("change", () => void saveRemoteVoices(area, remoteToggle.checked));
  previewBtn.addEventListener("click", () => {
    if (!speaker) return;
    if (speaker.speaking()?.key === PREVIEW_KEY) {
      speaker.stop();
      return;
    }
    // The select, not the stored preference: the change it just saved may not be back yet.
    const voice = speaker.eligible().find((v) => v.voiceURI === voiceSelect.value) ?? speaker.automatic();
    if (voice) speaker.speak(PREVIEW_KEY, previewSentence(interfaceLanguage, speaker.lang as StudiedLanguage), voice);
  });
  speaker?.subscribe(() => renderVoices());
  flowToggle.addEventListener("change", async () => {
    await saveReaderFlow(area, flowToggle.checked ? "scrolled" : "paginated");
  });
  syncBtn.addEventListener("click", () => void runSync());
  restartBtn.addEventListener("click", () => void restartFromServer());
  // A sync completes after every sign-in, here or in another surface: the account follows too.
  const unwatchSync = sync.watch(() => void Promise.all([refreshSync(), account.refresh()]));

  /**
   * The automatic choice first, naming the voice it lands on; then the ordinary voices; then
   * Apple's novelty, Eloquence and legacy voices apart, at the bottom — listed, out of the way.
   * A chosen voice no longer listed shows the automatic choice, which is what speaks.
   */
  function renderVoices(): void {
    const eligible = speaker?.eligible() ?? [];
    const offersAndroid = speaker?.offersAndroidVoices() ?? false;
    const offersRemote = speaker?.offersRemoteVoices() ?? false;
    // No voice listed at all is no synthesiser, or Chrome before it announces its voices.
    voiceBlock.hidden = eligible.length === 0 && !offersAndroid && !speaker?.listsVoices();
    if (!speaker || voiceBlock.hidden) return;
    const remote = offersRemote && speaker.remoteVoices();
    noVoiceNote.hidden = (eligible.length > 0 && !remote) || offersAndroid;
    // The language the host's speaker reads: a page's in the drawer, the reader's first elsewhere.
    const voiceLanguage = speaker.lang as StudiedLanguage;
    noVoiceText.textContent = noVoiceInstalled(interfaceLanguage, voiceLanguage);
    const installHelp = installVoiceHelp(copy, interfaceLanguage, voiceLanguage);
    const help = offersRemote ? copy.installVoiceHelpWithFallback(installHelp) : installHelp;
    installInfo.title = help;
    installInfo.setAttribute("aria-label", help);
    remoteRow.hidden = !offersRemote;
    remoteNote.hidden = !offersRemote;
    remoteToggle.checked = speaker.remoteVoices();
    androidRow.hidden = !offersAndroid;
    androidNote.hidden = !offersAndroid;
    androidToggle.checked = speaker.androidVoices();
    // Where Android's voices are allowed, the promise below would not hold: the note above says why.
    onDeviceNote.hidden = eligible.length === 0 || (offersAndroid && speaker.androidVoices()) || remote;
    voiceRow.hidden = eligible.length === 0;
    if (eligible.length === 0) return;
    const option = (value: string, label: string): HTMLOptionElement => {
      const o = el("option", undefined, label);
      o.value = value;
      return o;
    };
    const voiceOption = (v: VoiceInfo): HTMLOptionElement => option(v.voiceURI, voiceLabel(v, interfaceLanguage, copy));
    const { ordinary, others } = voiceGroups(eligible, speaker.lang, speaker.androidVoices(), speaker.remoteVoices());
    const automatic = speaker.automatic();
    voiceSelect.replaceChildren(
      option("", automatic ? copy.automaticVoice(automatic.name) : copy.automatic),
      ...ordinary.map(voiceOption),
    );
    if (others.length > 0) {
      const group = el("optgroup");
      group.label = copy.otherVoices;
      group.append(...others.map(voiceOption));
      voiceSelect.append(group);
    }
    const preferred = speaker.preferred();
    voiceSelect.value = preferred && eligible.some((v) => v.voiceURI === preferred) ? preferred : "";
    previewBtn.textContent = speaker.speaking()?.key === PREVIEW_KEY ? copy.stop : copy.listen;
  }

  async function runSync(): Promise<void> {
    syncBtn.disabled = true;
    syncMsg.textContent = copy.syncing;
    const reply = await sync.syncNow();
    syncMsg.textContent = reply.ok ? "" : syncErrorCopy(reply.error, blocksCopy.sync);
    syncBtn.disabled = false;
    await refreshSync();
  }

  async function refreshSync(): Promise<void> {
    const signedIn = await sync.available();
    syncBlock.hidden = !signedIn;
    localNote.hidden = signedIn;
    localOnly.hidden = signedIn;
    synced.hidden = !signedIn;
    syncStatus.textContent = lastSyncLabel(await sync.lastSync(), sync.now(), interfaceLanguage, blocksCopy.sync);
  }

  /** Empty this device and pull the account's state back. Nothing is lost by design. */
  async function restartFromServer(): Promise<void> {
    restartBtn.disabled = true;
    resetMsg.textContent = copy.restarting;
    await doReset("full");
    const reply = await sync.syncNow();
    resetMsg.textContent = reply.ok ? copy.restarted : syncErrorCopy(reply.error, blocksCopy.sync);
    restartBtn.disabled = false;
    await refreshSync();
  }

  async function doReset(scope: "full" | "partial"): Promise<void> {
    if (scope === "partial") {
      await port.resetStatuses();
    } else {
      await port.reset();
      await clearSyncCursors(opts.store);
    }
    // Every accepted language starts over. A full reset returns the profile to the engine's native
    // language, studying its first pack's language alone — English, for a reader of French
    // (generalise-lingua-native-language D5).
    for (const language of await acceptedLanguages(port, pairs)) {
      const view = port.for(language);
      await view.setCalibration((await view.hasLevels()) ? 0 : 3000);
    }
    await opts.persist();
    await opts.onReset?.();
    await refresh();
    if (!synced.hidden) return; // the signed-in path writes its own message
    resetMsg.textContent = scope === "partial" ? copy.partialResetDone : copy.dataErased;
  }

  async function refresh(): Promise<void> {
    await Promise.all([native?.refresh(), studied.refresh(), refreshLevels()]);
    toggle.checked = !(await loadHudHidden(area));
    renderVoices();
    flowToggle.checked = (await loadReaderFlow(area)) === "scrolled";
    await Promise.all([bookDisplay.refresh(), colours.refresh()]);
    await Promise.all([refreshSync(), account.refresh(), translation?.refresh()]);
  }

  void refresh();
  return {
    refresh,
    show: tabs.show,
    destroy: () => {
      unwatchSync?.();
      translation?.destroy();
    },
  };
}

interface TabSpec {
  id: SettingsTab;
  label: string;
  blocks: HTMLElement[];
}

/**
 * A tab row over one panel per tab (the WAI-ARIA tabs pattern: arrows, Home and End move between
 * tabs, and moving selects). The first tab shows until the reader picks another; the view keeps
 * that choice for as long as it is mounted.
 */
function mountTabs(container: HTMLElement, label: string, specs: TabSpec[]): { show: (tab: SettingsTab) => void } {
  const prefix = `cymbra-lingua-set-${++mountCount}`;
  const row = el("div", "set-tabs");
  row.setAttribute("role", "tablist");
  row.setAttribute("aria-label", label);
  const buttons: HTMLButtonElement[] = [];
  const panels: HTMLElement[] = [];
  for (const spec of specs) {
    const tab = el("button", "set-tab", spec.label);
    tab.type = "button";
    tab.id = `${prefix}-tab-${spec.id}`;
    tab.dataset.tab = spec.id;
    tab.setAttribute("role", "tab");
    tab.setAttribute("aria-controls", `${prefix}-${spec.id}`);
    tab.addEventListener("click", () => select(buttons.indexOf(tab)));
    const panel = el("div", "set-panel");
    panel.id = `${prefix}-${spec.id}`;
    panel.dataset.tab = spec.id;
    panel.setAttribute("role", "tabpanel");
    panel.setAttribute("aria-labelledby", tab.id);
    panel.append(...spec.blocks);
    buttons.push(tab);
    panels.push(panel);
  }
  row.append(...buttons);
  row.addEventListener("keydown", (e) => {
    const at = buttons.indexOf(e.target as HTMLButtonElement);
    if (at < 0) return;
    const last = buttons.length - 1;
    const to =
      e.key === "ArrowRight"
        ? (at + 1) % buttons.length
        : e.key === "ArrowLeft"
          ? (at + last) % buttons.length
          : e.key === "Home"
            ? 0
            : e.key === "End"
              ? last
              : -1;
    if (to < 0) return;
    e.preventDefault();
    select(to);
    buttons[to].focus();
  });
  container.append(row, ...panels);

  function select(index: number): void {
    buttons.forEach((b, i) => {
      const on = i === index;
      b.classList.toggle("active", on);
      b.setAttribute("aria-selected", String(on));
      b.tabIndex = on ? 0 : -1;
      panels[i].hidden = !on;
    });
  }

  select(0);
  return { show: (tab) => select(specs.findIndex((s) => s.id === tab)) };
}
