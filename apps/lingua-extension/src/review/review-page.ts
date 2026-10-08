import { acceptedLanguages, readingLanguage } from "../analyzer/pairs.ts";
import { languageName } from "../analyzer/language-labels.ts";
import type { LinguaPort } from "../analyzer/port.ts";
import type { StudiedLanguage } from "../analyzer/types.ts";
import {
  DEFAULT_INTERFACE_LANGUAGE,
  formatCount,
  type InterfaceLanguage,
  NODE_SLOT,
  plural,
  type PluralForms,
  renderAround,
} from "../i18n/index.ts";
import { dailyRecorder } from "../state/dailystats.ts";
import { type AsyncStorageArea, loadReviewLanguage, saveBackup, saveReviewLanguage } from "../state/storage.ts";
import { watchBackup } from "../state/store.ts";
import { ReviewController } from "./session.ts";
import { type ReviewActions, renderReview, reviewCopy } from "./view.ts";

// The full Révision page — summary + the FSRS review widget + lossless backup/restore +
// the pack's Sources & confidentialité — built as plain DOM into a container so ONE
// implementation serves both hosts: the native side panel and the in-page drawer (same
// pattern as mountSettings / mountStats). It owns a ReviewController on the host's port and
// persists via saveBackup, so every other surface reacts through storage.onChanged.

export interface ReviewPageOptions {
  /** Epoch-seconds clock (Date.now()/1000 in production). */
  now: () => number;
  /** Preferences (chrome.storage.local), where the last language chosen in the review is kept
   *  (refine-lingua-review-language D3). */
  prefs: AsyncStorageArea;
  /** The language of the page or book the review is shown beside, or null away from one (D2). */
  pageLanguage?: () => Promise<string | null>;
  /** The interface language its host read, whose catalogue module the page speaks
   *  (localise-lingua-review-stats D1); French when not given. */
  interfaceLanguage?: InterfaceLanguage;
}

export interface ReviewPage {
  /** Re-sync the summary + review view from the engine (call each time it is shown). */
  refresh: () => Promise<void>;
  /** Whether a review session is under way (a restore would end it). */
  reviewing: () => boolean;
  /**
   * The review opens anew beside a page, or the page or book beside it changed: the next refresh
   * opens in that page's language, whatever was chosen beside the last one (D3).
   */
  pageChanged: () => void;
}

function el<K extends keyof HTMLElementTagNameMap>(tag: K, className?: string): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (className) node.className = className;
  return node;
}

/** Mount the Révision page into `container`. */
export function mountReview(
  container: HTMLElement,
  port: LinguaPort,
  area: AsyncStorageArea,
  opts: ReviewPageOptions,
): ReviewPage {
  container.replaceChildren();
  const interfaceLanguage = opts.interfaceLanguage ?? DEFAULT_INTERFACE_LANGUAGE;
  const copy = reviewCopy(interfaceLanguage);
  // Every grade and mark-known is counted under the engine's native language (add-lingua-native-language-sync-client D3).
  const recorder = dailyRecorder(area, port);
  let controller = new ReviewController(port, opts.now, recorder);
  let lastBackup: string | null = null;

  const summary = el("div", "summary");
  const review = el("div", "review");
  // The language filter, when the reader studies several (add-lingua-language-stats-review D2):
  // one language at a time, never several mixed (refine-lingua-review-language D1).
  const filterRow = el("div", "review-languages");
  filterRow.setAttribute("role", "group");
  filterRow.setAttribute("aria-label", copy.language);
  filterRow.hidden = true;
  let languages: StudiedLanguage[] = [];
  /** The language the review is in: one of `languages` once they are read. */
  let language: StudiedLanguage | null = null;
  /** The page language the review last followed; undefined until it is first asked (D2). */
  let followed: StudiedLanguage | null | undefined;
  const only = (): StudiedLanguage[] | undefined => (language ? [language] : undefined);
  const accepted = (candidate: string | null | undefined): StudiedLanguage | null =>
    languages.find((l) => l === candidate) ?? null;
  const render = (view: ReturnType<ReviewController["view"]>): void =>
    renderReview(review, view, actions, { showLanguage: languages.length > 1, interfaceLanguage });
  /** A count of the summary — « 12 carte(s) » — its figure in bold where its message puts it. */
  const counted = (n: number, forms: PluralForms): Node[] => {
    const holder = el("span");
    renderAround(holder, plural(interfaceLanguage, n, forms, NODE_SLOT), bold(formatCount(interfaceLanguage, n)));
    return [...holder.childNodes];
  };

  const tools = el("div", "tools");
  const backupBtn = el("button");
  backupBtn.type = "button";
  backupBtn.textContent = copy.backup;
  const restoreBtn = el("button");
  restoreBtn.type = "button";
  restoreBtn.textContent = copy.restore;
  const fileInput = el("input");
  fileInput.type = "file";
  fileInput.accept = "application/json";
  fileInput.hidden = true;
  tools.append(backupBtn, restoreBtn, fileInput);
  const msg = el("div", "msg");

  const details = el("details");
  const detailsSummary = el("summary");
  detailsSummary.textContent = copy.sourcesAndPrivacy;
  const privacy = el("div", "privacy");
  privacy.textContent = copy.privacy;
  const licences = el("div");
  const notice = el("pre", "notice");
  details.append(detailsSummary, privacy, licences, notice);

  container.append(filterRow, summary, review, tools, msg, details);

  /** The filter's segments: one per accepted language, the review's own active. */
  function drawFilter(): void {
    filterRow.hidden = languages.length < 2;
    filterRow.replaceChildren(
      ...languages.map((choice) => {
        const b = el("button");
        b.type = "button";
        b.textContent = languageName(choice);
        b.dataset.language = choice;
        if (choice === language) b.className = "active";
        b.addEventListener("click", () => void choose(choice));
        return b;
      }),
    );
  }

  /** Where the review opens: the page's language, else the last chosen here, else the first (D2). */
  async function opening(page: StudiedLanguage | null): Promise<StudiedLanguage> {
    return page ?? accepted(await loadReviewLanguage(opts.prefs)) ?? languages[0];
  }

  /** Put the review in `next`: a session over in another language gives way to a new start. */
  function switchTo(next: StudiedLanguage): void {
    if (next !== language && controller.view().phase === "done") {
      controller = new ReviewController(port, opts.now, recorder);
    }
    language = next;
  }

  /** The reader's languages; the review keeps its language while they still hold it. */
  async function readLanguages(): Promise<void> {
    languages = await acceptedLanguages(port);
    if (!accepted(language)) switchTo(await opening(accepted(followed)));
    drawFilter();
  }

  /**
   * Follow the page the review is shown beside: a page it has not followed yet wins, while a
   * language chosen beside the same one stands (D2, D3). A session under way keeps its queue.
   */
  async function followPage(): Promise<void> {
    if (controller.view().phase === "reviewing") return;
    const page = accepted((await opts.pageLanguage?.()) ?? null);
    if (page === followed && accepted(language)) return;
    followed = page;
    switchTo(await opening(page));
    drawFilter();
  }

  async function choose(choice: StudiedLanguage): Promise<void> {
    if (controller.view().phase === "reviewing") return; // a session under way keeps its queue
    switchTo(choice);
    drawFilter();
    render(controller.view());
    await saveReviewLanguage(opts.prefs, choice);
    await refreshSummary();
  }

  async function persist(): Promise<void> {
    const backup = await port.backup();
    lastBackup = backup; // ignore our own storage.onChanged echo
    await saveBackup(area, backup);
  }

  async function refreshSummary(): Promise<void> {
    const [deck, due] = [await port.deckCount(only()), await port.dueCount(opts.now(), only())];
    summary.replaceChildren(...counted(deck, copy.deckCards), copy.summarySeparator, ...counted(due, copy.dueCards));
  }

  async function run(produce: () => Promise<ReturnType<ReviewController["view"]>>, changed: boolean): Promise<void> {
    const view = await produce();
    if (changed) {
      await persist();
      await refreshSummary();
    }
    render(view);
  }

  const actions: ReviewActions = {
    start: () => void run(() => controller.start(only()), false),
    reveal: () => void run(() => controller.reveal(), false),
    grade: (rating) => void run(() => controller.grade(rating), true),
    markKnown: () => void run(() => controller.markKnown(), true),
  };

  function download(name: string, text: string): void {
    const url = URL.createObjectURL(new Blob([text], { type: "application/json" }));
    const a = el("a");
    a.href = url;
    a.download = name;
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }

  async function doRestore(file: File): Promise<void> {
    try {
      await port.restore(await file.text());
      await persist();
      controller = new ReviewController(port, opts.now, recorder);
      await readLanguages(); // the file's profile may hold other languages
      await refreshSummary();
      render(controller.view());
      msg.textContent = copy.restored;
    } catch {
      msg.textContent = copy.notABackup;
    }
  }

  backupBtn.addEventListener("click", async () => download("cymbra-lingua-backup.json", await port.backup()));
  restoreBtn.addEventListener("click", () => fileInput.click());
  fileInput.addEventListener("change", () => {
    const file = fileInput.files?.[0];
    if (file) void doRestore(file);
  });

  void loadAttributions();
  /** The sources of the reader's language's pack (add-lingua-studied-language-profile). */
  async function loadAttributions(): Promise<void> {
    const lang = port.for(await readingLanguage(port));
    const names = await lang.licences();
    licences.textContent = names.length ? copy.sources(names.join(copy.sourceSeparator)) : "";
    notice.textContent = await lang.notice();
  }

  // Keep in sync with changes made elsewhere (a reading gesture, a reset in Réglages, or
  // another surface), unless mid-review or it is our own echo. The controller caches its
  // state, so it is rebuilt to reflect the restored engine.
  watchBackup(area, (backup) => {
    if (backup === lastBackup) return;
    if (controller.view().phase === "reviewing") return;
    void port.restore(backup).then(async () => {
      controller = new ReviewController(port, opts.now, recorder);
      void loadAttributions();
      // The profile came with the state: a language may have joined or left the filter.
      await readLanguages();
      await refreshSummary();
      render(controller.view());
    });
  });

  return {
    refresh: async () => {
      await readLanguages();
      await followPage();
      await refreshSummary();
      render(controller.view());
    },
    reviewing: () => controller.view().phase === "reviewing",
    pageChanged: () => {
      followed = undefined;
    },
  };
}

function bold(text: string): HTMLElement {
  const b = document.createElement("b");
  b.textContent = text;
  return b;
}
