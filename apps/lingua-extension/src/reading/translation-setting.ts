import { translation as frTranslation } from "../i18n/fr/translation.ts";
import { DEFAULT_INTERFACE_LANGUAGE, formatNumber, type InterfaceLanguage } from "../i18n/index.ts";
import { keepEngineWarm } from "../translate/keepalive.ts";
import { isElement } from "./blocks.ts";
import { askModel, type ModelCommand, type ModelCost, type ModelStatus } from "../translate/model-messages.ts";
import {
  loadTranslationSetting,
  MODEL_STATE_KEY,
  type ModelFailure,
  type ModelState,
  TRANSLATION_HOST_KEY,
  type TranslationSetting,
} from "../translate/setting.ts";
import { settingsCopy, type TranslationCopy } from "./settings-copy.ts";

// « Traduction étendue » in the Réglages view (add-lingua-translation-delivery D2, D4, D8): one
// checkbox, its cost stated before it is ticked, and — once ticked — where the model stands, with
// the one action each state offers. Built as plain DOM into the block the settings view gives it,
// so the side panel and the in-page drawer show the same thing.
//
// The background does everything; this asks it (model-messages.ts) and follows the two storage
// keys it writes, so progress reaches every open copy of the view. Failures are explained in the
// reader's words, never with the error, which the background logs. The words are the catalogue's
// `translation` module and the sizes are written in the interface language, both handed by the
// settings view (localise-lingua-settings D1, D4); French when neither is given.

/** What the row needs from the background and the browser. A test hands in a fake. */
export interface TranslationControls {
  /** Where things stand, reconciled by the background (an interrupted download, a removed model). */
  status(): Promise<ModelStatus>;
  command(op: Exclude<ModelCommand, "status">): Promise<ModelStatus>;
  /** Call `onChange` with the stored setting whenever the background changes it. */
  watch(onChange: (setting: TranslationSetting) => void): void;
  /** Keep the engine's host awake — ping it — while `when` holds; returns how to stop. */
  keepAwake(when: () => boolean): () => void;
}

export function runtimeTranslationControls(): TranslationControls {
  return {
    status: () => askModel("status"),
    command: (op) => askModel(op),
    watch: (onChange) =>
      chrome.storage.onChanged.addListener((changes, area) => {
        if (area !== "local" || !(TRANSLATION_HOST_KEY in changes || MODEL_STATE_KEY in changes)) return;
        void loadTranslationSetting(chrome.storage.local).then(onChange, () => undefined);
      }),
    keepAwake: (when) => keepEngineWarm(undefined, when),
  };
}

/** The setting's French copy, the catalogue's module: what it shows when mounted without a language. */
export const COPY: TranslationCopy = frTranslation;

/**
 * Bytes as the reader reads sizes: decimal megabytes, one decimal, in the interface language
 * (« 25,8 Mo » in French, as before the catalogue).
 */
export function megabytes(
  bytes: number,
  language: InterfaceLanguage = DEFAULT_INTERFACE_LANGUAGE,
  copy: TranslationCopy = settingsCopy(language).translation,
): string {
  return copy.megabytes(
    formatNumber(language, bytes / 1_000_000, { minimumFractionDigits: 1, maximumFractionDigits: 1 }),
  );
}

/**
 * What the setting costs, before it is ticked: the sizes come from the package's catalogue
 * (generalise-lingua-translation-catalogue D4), and without them the sentence names none.
 */
export function costText(
  cost?: ModelCost,
  language: InterfaceLanguage = DEFAULT_INTERFACE_LANGUAGE,
  copy: TranslationCopy = settingsCopy(language).translation,
): string {
  const download = cost ? megabytes(cost.download, language, copy) : copy.theModel;
  // One model works in about 200 MB; through the pivot two do, the study's 322 MiB
  // (add-lingua-spanish-translation-pivot D4).
  const memory = cost?.pivot ? copy.memoryPivot : copy.memorySingle;
  return copy.cost(download, memory);
}

/** Why a download failed, in the reader's words; `size` names a storage failure's need when known. */
function failureText(reason: ModelFailure, copy: TranslationCopy, size: string | null): string {
  switch (reason) {
    case "network":
      return copy.failedNetwork;
    case "unavailable":
      return copy.failedUnavailable;
    case "not-the-model":
      return copy.failedNotTheModel;
    case "storage":
      return size ? copy.failedStorageSized(size) : copy.failedStorage;
    default:
      return copy.failedUnknown;
  }
}

/** The line under the checkbox, for a state; `cost` sizes a storage failure. */
export function stateText(
  state: ModelState,
  cost?: ModelCost,
  language: InterfaceLanguage = DEFAULT_INTERFACE_LANGUAGE,
  copy: TranslationCopy = settingsCopy(language).translation,
): string {
  const size = (bytes: number): string => megabytes(bytes, language, copy);
  // « 12,3 Mo sur 25,8 Mo » — or nothing to add when the total is unknown.
  const progress = (received: number, total: number): string =>
    total > 0 ? copy.progress(size(received), size(total)) : "";
  switch (state.phase) {
    case "downloading":
      return copy.downloading(progress(state.received, state.total));
    case "ready":
      return copy.ready;
    case "failed":
      return failureText(state.reason, copy, cost ? size(cost.stored) : null);
    case "interrupted":
      return `${copy.interrupted}${progress(state.received, state.total)}`;
    case "removed":
      return copy.removed;
    case "missing":
      return state.total > 0 ? copy.missingSized(size(state.total)) : copy.missing;
    default:
      return "";
  }
}

/**
 * Whether `node` is on screen as far as the DOM says: attached, and no ancestor hidden — through
 * a shadow root to its host. Node types, not `instanceof`: the view may live in another realm.
 */
export function shown(node: Element): boolean {
  if (!node.isConnected) return false;
  for (let n: Node | null = node; n;) {
    if (isElement(n) && (n as HTMLElement).hidden) return false;
    const parent: Node | null = n.parentNode;
    n = parent?.nodeType === Node.DOCUMENT_FRAGMENT_NODE && "host" in parent ? (parent as ShadowRoot).host : parent;
  }
  return true;
}

function el<K extends keyof HTMLElementTagNameMap>(
  doc: Document,
  tag: K,
  className?: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const node = doc.createElement(tag);
  if (className) node.className = className;
  if (text != null) node.textContent = text;
  return node;
}

export interface TranslationSettingView {
  /** Re-read where things stand (call each time the settings view is shown). */
  refresh(): Promise<void>;
}

export interface TranslationSettingOptions {
  /** The interface language: the sizes are written in it; French when not given. */
  language?: InterfaceLanguage;
  /** The setting's copy; that language's module when not given. */
  copy?: TranslationCopy;
}

/** Mount the row into `block`, the settings view's block for translation. */
export function mountTranslationSetting(
  block: HTMLElement,
  controls: TranslationControls,
  opts: TranslationSettingOptions = {},
): TranslationSettingView {
  block.hidden = true; // until the background says the setting is offered here
  const language = opts.language ?? DEFAULT_INTERFACE_LANGUAGE;
  const copy = opts.copy ?? settingsCopy(language).translation;
  // The document the view is mounted in: a panel's, the popup's or a drawer's shadow.
  const doc = block.ownerDocument;
  const row = el(doc, "label", "set-toggle");
  const box = el(doc, "input");
  box.type = "checkbox";
  row.append(box, el(doc, "span", undefined, copy.toggle));
  const costNote = el(doc, "div", "set-note", costText(undefined, language, copy));
  const attribution = el(doc, "div", "set-note", copy.attribution);
  const line = el(doc, "div", "set-note");
  line.setAttribute("role", "status");
  const bar = el(doc, "progress", "set-progress");
  const action = el(doc, "button", "set-reset");
  action.type = "button";
  block.append(row, costNote, attribution, bar, line, action);

  let status: ModelStatus | null = null;
  let stopPinging: (() => void) | null = null;
  let busy = false;

  /** What the action button does in each state, and what it says. */
  const ACTIONS: Partial<Record<ModelState["phase"], { label: string; op: "disable" | "resume" }>> = {
    downloading: { label: copy.cancel, op: "disable" },
    failed: { label: copy.retry, op: "resume" },
    interrupted: { label: copy.resume, op: "resume" },
    removed: { label: copy.again, op: "resume" },
    // A language the reader added needs a model: asked for, never fetched unasked (model-state D3).
    missing: { label: copy.download, op: "resume" },
  };

  function render(): void {
    if (!status?.offered) {
      block.hidden = true;
      awake(false);
      return;
    }
    block.hidden = false;
    const { host, state } = status;
    costNote.textContent = costText(status.cost, language, copy);
    box.checked = host === "local";
    box.disabled = busy;
    const downloading = host === "local" && state.phase === "downloading";
    bar.hidden = !downloading;
    if (downloading && state.total > 0) {
      bar.max = state.total;
      bar.value = Math.min(state.received, state.total);
    } else {
      bar.removeAttribute("value"); // indeterminate
    }
    line.textContent = host === "local" ? stateText(state, status.cost, language, copy) : "";
    line.hidden = !line.textContent;
    const offer = host === "local" ? ACTIONS[state.phase] : undefined;
    action.hidden = !offer;
    action.disabled = busy;
    if (offer) {
      action.textContent = offer.label;
      action.dataset.op = offer.op;
    }
    awake(downloading);
  }

  /** While a download runs and the view is on screen, keep its host awake (D4). */
  function awake(on: boolean): void {
    if (on && !stopPinging) stopPinging = controls.keepAwake(() => shown(block));
    if (!on && stopPinging) {
      stopPinging();
      stopPinging = null;
    }
  }

  async function run(op: Exclude<ModelCommand, "status">): Promise<void> {
    busy = true;
    render();
    try {
      status = await controls.command(op);
    } finally {
      busy = false;
      render();
    }
  }

  box.addEventListener("change", () => void run(box.checked ? "enable" : "disable"));
  action.addEventListener("click", () => {
    const op = action.dataset.op;
    if (op === "disable" || op === "resume") void run(op);
  });
  controls.watch((setting) => {
    if (!status) return;
    status = { ...status, ...setting };
    render();
  });

  async function refresh(): Promise<void> {
    status = await controls.status();
    render();
  }

  return { refresh };
}
