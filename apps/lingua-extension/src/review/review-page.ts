import { acceptedLanguages } from "../analyzer/pairs.ts";
import { languageName } from "../analyzer/language-labels.ts";
import type { LinguaPort } from "../analyzer/port.ts";
import type { StudiedLanguage } from "../analyzer/types.ts";
import { dailyRecorder } from "../state/dailystats.ts";
import {
  type AsyncStorageArea,
  loadNewWordsPerDay,
  loadReviewLanguage,
  saveBackup,
  saveReviewLanguage,
} from "../state/storage.ts";
import { watchBackup } from "../state/store.ts";
import { ReviewController } from "./session.ts";
import { type ReviewActions, renderReview } from "./view.ts";

// The Révision page — the language filter, the due count and the FSRS review card — built as
// plain DOM into a container so ONE implementation serves both hosts: the native side panel and
// the in-page drawer (same pattern as mountSettings / mountStats). It owns a ReviewController on
// the host's port and persists via saveBackup, so every other surface reacts through
// storage.onChanged. Backup, restore and the pack's sources live in Réglages › Données
// (refine-lingua-review-session D9).

export interface ReviewPageOptions {
  /** Epoch-seconds clock (Date.now()/1000 in production). */
  now: () => number;
  /** Preferences (chrome.storage.local): the last language chosen in the review
   *  (refine-lingua-review-language D3) and the daily allowance of new words. */
  prefs: AsyncStorageArea;
  /** The language of the page or book the review is shown beside, or null away from one (D2). */
  pageLanguage?: () => Promise<string | null>;
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

/** Mount the Révision page into `container`; `area` holds the reader's data. */
export function mountReview(
  container: HTMLElement,
  port: LinguaPort,
  area: AsyncStorageArea,
  opts: ReviewPageOptions,
): ReviewPage {
  container.replaceChildren();
  const controller = (): ReviewController =>
    new ReviewController(port, opts.now, dailyRecorder(area), () => loadNewWordsPerDay(opts.prefs));
  let session = controller();
  let lastBackup: string | null = null;

  const summary = el("div", "summary");
  const review = el("div", "review");
  // The language filter, when the reader studies several (add-lingua-language-stats-review D2):
  // one language at a time, never several mixed (refine-lingua-review-language D1).
  const filterRow = el("div", "review-languages");
  filterRow.setAttribute("role", "group");
  filterRow.setAttribute("aria-label", "Langue");
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
    renderReview(review, view, actions, { showLanguage: languages.length > 1 });

  container.append(filterRow, summary, review);

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
    if (next !== language && session.view().phase === "done") session = controller();
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
    if (session.view().phase === "reviewing") return;
    const page = accepted((await opts.pageLanguage?.()) ?? null);
    if (page === followed && accepted(language)) return;
    followed = page;
    switchTo(await opening(page));
    drawFilter();
  }

  async function choose(choice: StudiedLanguage): Promise<void> {
    if (session.view().phase === "reviewing") return; // a session under way keeps its queue
    switchTo(choice);
    drawFilter();
    render(session.view());
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
    summary.replaceChildren(
      bold(String(deck)),
      document.createTextNode(" carte(s) · "),
      bold(String(due)),
      document.createTextNode(" à revoir"),
    );
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
    // A session starts with the focus in the card, so the keys answer it at once.
    start: () => void run(() => session.start(only()), false).then(() => review.focus({ preventScroll: true })),
    reveal: () => void run(() => session.reveal(), false),
    grade: (rating) => void run(() => session.grade(rating), true),
    markKnown: () => void run(() => session.markKnown(), true),
    ignore: () => void run(() => session.ignore(), true),
  };

  /** A fresh controller over the engine as it now is: the idle card and the counts. */
  async function restart(): Promise<void> {
    session = controller();
    // The profile came with the state: a language may have joined or left the filter.
    await readLanguages();
    await refreshSummary();
    render(session.view());
  }

  // Keep in sync with changes made elsewhere (a reading gesture, a reset or a restore in
  // Réglages, or another surface), unless it is our own echo. Mid-review, another surface's
  // change waits — unless it already ended this engine's session: a restore made in this very
  // page (Réglages › Données beside the review) drops the session it was walking.
  watchBackup(area, (backup) => {
    if (backup === lastBackup) return;
    void (async () => {
      if (session.view().phase === "reviewing") {
        if (await port.reviewCurrent()) return;
        await restart();
        return;
      }
      await port.restore(backup);
      await restart();
    })();
  });

  return {
    refresh: async () => {
      await readLanguages();
      await followPage();
      await refreshSummary();
      render(session.view());
    },
    reviewing: () => session.view().phase === "reviewing",
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
