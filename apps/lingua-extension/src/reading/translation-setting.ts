import { keepEngineWarm } from "../translate/keepalive.ts";
import { isElement } from "./blocks.ts";
import { askModel, type ModelCommand, type ModelStatus } from "../translate/model-messages.ts";
import {
  loadTranslationSetting,
  MODEL_STATE_KEY,
  type ModelFailure,
  type ModelState,
  TRANSLATION_HOST_KEY,
  type TranslationSetting,
} from "../translate/setting.ts";

// « Traduction étendue » in the Réglages view (add-lingua-translation-delivery D2, D4, D8): one
// checkbox, its cost stated before it is ticked, and — once ticked — where the model stands, with
// the one action each state offers. Built as plain DOM into the block the settings view gives it,
// so the side panel and the in-page drawer show the same thing.
//
// The background does everything; this asks it (model-messages.ts) and follows the two storage
// keys it writes, so progress reaches every open copy of the view. Failures are explained in the
// reader's words, never with the error, which the background logs.

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

export const COPY = {
  toggle: "Traduction étendue",
  cost:
    "Traduit tes phrases sur cet appareil, sans rien envoyer. Télécharge 25,8 Mo une fois, puis utilise " +
    "environ 200 Mo de mémoire pendant la traduction. Réglage propre à cet appareil.",
  attribution: "Modèle de traduction : Firefox Translations (Mozilla), licence MPL 2.0.",
  ready: "Prête : tes sélections sont traduites sur cet appareil.",
  interrupted: "Téléchargement interrompu.",
  removed: "Le navigateur a supprimé le modèle de cet appareil. Il faut le télécharger à nouveau.",
  cancel: "Annuler",
  retry: "Réessayer",
  resume: "Reprendre",
  again: "Télécharger à nouveau",
} as const;

const FAILURE: Record<ModelFailure, string> = {
  network: "Le téléchargement a échoué : pas de connexion. Réessaie une fois en ligne.",
  unavailable: "Le téléchargement a échoué : le serveur ne répond pas. Réessaie plus tard.",
  "not-the-model": "Le téléchargement a échoué : le fichier reçu n'est pas le bon modèle. Réessaie plus tard.",
  storage: "Pas assez de place sur cet appareil pour le modèle (37 Mo).",
  unknown: "Le téléchargement a échoué. Réessaie plus tard.",
};

/** Bytes as the reader reads sizes: decimal megabytes, one decimal, French style. */
export function megabytes(bytes: number): string {
  return `${(bytes / 1_000_000).toLocaleString("fr-FR", { minimumFractionDigits: 1, maximumFractionDigits: 1 })} Mo`;
}

/** "12,3 Mo sur 25,8 Mo" — or nothing to add when the total is unknown. */
function progressText(received: number, total: number): string {
  return total > 0 ? ` ${megabytes(received)} sur ${megabytes(total)}` : "";
}

/** The line under the checkbox, for a state. */
export function stateText(state: ModelState): string {
  switch (state.phase) {
    case "downloading":
      return `Téléchargement du modèle…${progressText(state.received, state.total)}`;
    case "ready":
      return COPY.ready;
    case "failed":
      return FAILURE[state.reason];
    case "interrupted":
      return `${COPY.interrupted}${progressText(state.received, state.total)}`;
    case "removed":
      return COPY.removed;
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

/** Mount the row into `block`, the settings view's block for translation. */
export function mountTranslationSetting(block: HTMLElement, controls: TranslationControls): TranslationSettingView {
  block.hidden = true; // until the background says the setting is offered here
  // The document the view is mounted in: a panel's, the popup's or a drawer's shadow.
  const doc = block.ownerDocument;
  const row = el(doc, "label", "set-toggle");
  const box = el(doc, "input");
  box.type = "checkbox";
  row.append(box, el(doc, "span", undefined, COPY.toggle));
  const cost = el(doc, "div", "set-note", COPY.cost);
  const attribution = el(doc, "div", "set-note", COPY.attribution);
  const line = el(doc, "div", "set-note");
  line.setAttribute("role", "status");
  const bar = el(doc, "progress", "set-progress");
  const action = el(doc, "button", "set-reset");
  action.type = "button";
  block.append(row, cost, attribution, bar, line, action);

  let status: ModelStatus | null = null;
  let stopPinging: (() => void) | null = null;
  let busy = false;

  /** What the action button does in each state, and what it says. */
  const ACTIONS: Partial<Record<ModelState["phase"], { label: string; op: "disable" | "resume" }>> = {
    downloading: { label: COPY.cancel, op: "disable" },
    failed: { label: COPY.retry, op: "resume" },
    interrupted: { label: COPY.resume, op: "resume" },
    removed: { label: COPY.again, op: "resume" },
  };

  function render(): void {
    if (!status?.offered) {
      block.hidden = true;
      awake(false);
      return;
    }
    block.hidden = false;
    const { host, state } = status;
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
    line.textContent = host === "local" ? stateText(state) : "";
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
