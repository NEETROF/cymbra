import { acceptedLanguages } from "../analyzer/pairs.ts";
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
import { browserSpeechEngine, createSpeaker, type SpeechEngine } from "../reading/speech.ts";
import { type OpenPage, openPageViaBackground } from "../state/open-page.ts";
import { STORE_REVIEW_PAGE } from "../state/store-links.ts";
import { dailyRecorder } from "../state/dailystats.ts";
import {
  type AsyncStorageArea,
  DEFAULT_NEW_WORDS_PER_DAY,
  loadNewWordsPerDay,
  loadReviewLanguage,
  saveBackup,
  saveReviewLanguage,
  storedVoicePreference,
} from "../state/storage.ts";
import { watchBackup } from "../state/store.ts";
import type { TranslatorSource } from "../translate/create-port.ts";
import type { OnlineTranslationRequest, OnlineTranslationResult } from "../translate/online.ts";
import {
  afterSession,
  answer as answerRequest,
  loadRating,
  localDay,
  markShown,
  type RatingChoice,
  ratingDue,
  ratingOffered,
  saveRating,
} from "./rating.ts";
import { localDayStart, ReviewController } from "./session.ts";
import {
  cardTranslationRequest,
  onlineReviewRequest,
  type ReviewTranslation,
  type ReviewTranslationRequest,
  translateBounded,
} from "./translation.ts";
import {
  type ReviewActions,
  type ReviewOnlineOffer,
  type ReviewTranslationHint,
  renderReview,
  reviewCopy,
  sessionCompleted,
} from "./view.ts";

// The Révision page — the language filter, the due count and the FSRS review card — built as
// plain DOM into a container so ONE implementation serves both hosts: the native side panel and
// the in-page drawer (same pattern as mountSettings / mountStats). It owns a ReviewController on
// the host's port and persists via saveBackup, so every other surface reacts through
// storage.onChanged. Backup, restore and the pack's sources live in Réglages › Données
// (refine-lingua-review-session D9).

/**
 * « Traduction en ligne » as a review host has it (add-lingua-review-translation-fallback D2): whether
 * it is offered right now — the setting ticked on this device and the account's access — and what
 * sends a request. In the extension, the runtime-message client the selection card uses.
 */
export interface ReviewOnlineSource {
  offered(): boolean;
  translate(request: OnlineTranslationRequest): Promise<OnlineTranslationResult>;
}

/**
 * The extension's hint (add-lingua-review-translation-fallback D12): « Traduction étendue » is in the
 * host's own Réglages — off, or its models not on this device.
 */
export const inSettings = (): ReviewTranslationHint => ({ kind: "settings" });

export interface ReviewPageOptions {
  /** Epoch-seconds clock (Date.now()/1000 in production). */
  now: () => number;
  /** Preferences (chrome.storage.local): the last language chosen in the review
   *  (refine-lingua-review-language D3), the daily allowance of new words, and the voices. */
  prefs: AsyncStorageArea;
  /** The language of the page or book the review is shown beside, or null away from one (D2). */
  pageLanguage?: () => Promise<string | null>;
  /** What reads the card aloud: the browser's synthesiser unless given (a test's, or none). */
  speech?: SpeechEngine | null;
  /** The interface language its host read, whose catalogue module the page speaks
   *  (localise-lingua-review-stats D1); French when not given. */
  interfaceLanguage?: InterfaceLanguage;
  /**
   * The translator its host has, asked per language: a card the pack cannot answer is translated
   * where a model is ready (add-lingua-review-translation D4). Without one, the review is today's.
   */
  translator?: TranslatorSource;
  /**
   * « Traduction en ligne » in review (add-lingua-review-translation-fallback D2). Without one, no
   * card offers it.
   */
  online?: ReviewOnlineSource;
  /**
   * How the reader gets a translation for a card in `language` that nothing can translate — no
   * gloss, no model ready, no online button (add-lingua-review-translation-fallback D12) — or null
   * when nothing they could do would help. Without one, the no-translation note stands alone.
   */
  translationHint?: (language: string) => ReviewTranslationHint | null;
  /**
   * Beside a book, how many words looked up in it are still to sort (add-lingua-chapter-recap D6);
   * none, or 0, away from one: no count is shown.
   */
  toSort?: () => Promise<number>;
  /** Open the book's list of words to sort, in the reader page beside. */
  openToSort?: () => void;
  /**
   * Whether the end of a session may ask for a rating (add-lingua-comeback-hints D10): the build's —
   * Chromium and Firefox, never the Safari extension nor the Cymbra Lingua app (D11) — unless given.
   */
  askRating?: boolean;
  /** Opens the store page when the reader chooses to rate: the background does it, unless given. */
  openPage?: OpenPage;
  /**
   * A session's end said it was complete (« Séance terminée »): the Cymbra Lingua app counts it for
   * StoreKit's own request (add-lingua-comeback-hints D11).
   */
  onSessionFinished?: () => void;
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
  /**
   * Stop following the store: its host is taken down — a reading session built anew for another
   * native language (add-lingua-native-language-choice D3). A host that reloads never calls it.
   */
  destroy: () => void;
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
  const interfaceLanguage = opts.interfaceLanguage ?? DEFAULT_INTERFACE_LANGUAGE;
  const copy = reviewCopy(interfaceLanguage);
  // Every grade and mark-known is counted under the engine's native language (add-lingua-native-language-sync-client D3).
  const recorder = dailyRecorder(area, port);
  const controller = (): ReviewController =>
    new ReviewController(port, opts.now, recorder, () => loadNewWordsPerDay(opts.prefs));
  let session = controller();
  let lastBackup: string | null = null;

  const summary = el("div", "summary");
  // The words of the book beside still to sort, opening its list (add-lingua-chapter-recap D6).
  const toSort = el("button", "review-to-sort");
  toSort.type = "button";
  toSort.hidden = true;
  toSort.addEventListener("click", () => opts.openToSort?.());
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
  // Reads the card aloud in the card's own language, with the voice chosen for it in Réglages
  // (refine-lingua-review-session D11).
  // Its lifetime is the page's: `destroy` stops it following the voices and the settings
  // (add-lingua-native-language-choice D3).
  const alive = new AbortController();
  let spoken: string = "en";
  const speaker = createSpeaker(
    opts.speech === undefined ? browserSpeechEngine() : opts.speech,
    () => spoken,
    storedVoicePreference(opts.prefs),
    alive.signal,
  );
  /**
   * The current card's translation, tied to its arrival: a card that comes up — a missed one that
   * comes back included — is a new arrival, and an answer for an earlier one is dropped (D2).
   */
  let arrival = 0;
  let assist: Assist = { mine: 0, request: null, asked: false, kept: false };
  // The rating request (add-lingua-comeback-hints D10): each complete end counted once, the request
  // shown at the end that qualifies until the reader answers or starts another session.
  const askRating = opts.askRating ?? ratingOffered(__TARGET__, __APP__);
  let endCounted = false;
  let ratingShown = false;
  const ratingActions = {
    rate: () => void answerRating("rate"),
    later: () => void answerRating("later"),
    never: () => void answerRating("never"),
  };
  const render = (view: ReturnType<ReviewController["view"]>): void => {
    spoken = view.card?.language ?? language ?? spoken;
    renderReview(review, view, actions, {
      showLanguage: languages.length > 1,
      interfaceLanguage,
      speaker,
      ...(view.card ? assisted(view.card) : {}),
      ...(ratingShown && view.phase === "done" ? { rating: ratingActions } : {}),
    });
  };

  /** A new session, or none: the next complete end is counted, and no request stays on screen. */
  function newSession(): void {
    endCounted = false;
    ratingShown = false;
  }

  /**
   * An end that says the session is complete — practice included, never « Rien à réviser » — is
   * counted once; the request shows when this end qualifies, and counts as made from then on (D10).
   * Nothing is sent anywhere: the state stays in the preferences of this device.
   */
  async function countEnd(view: ReturnType<ReviewController["view"]>): Promise<void> {
    if (endCounted || view.phase !== "done" || !sessionCompleted(view.summary)) return;
    endCounted = true;
    opts.onSessionFinished?.();
    if (!askRating) return;
    let state = afterSession(await loadRating(opts.prefs), localDay(opts.now()));
    if (ratingDue(state)) {
      state = markShown(state);
      ratingShown = true;
    }
    await saveRating(opts.prefs, state);
  }

  /** The reader's answer: « Noter » opens the store page through the background, and only then. */
  async function answerRating(choice: RatingChoice): Promise<void> {
    if (!ratingShown) return;
    ratingShown = false;
    // The app has no extension API and asks no rating of its own (D11): its bundle folds this away.
    if (choice === "rate" && !__APP__) (opts.openPage ?? openPageViaBackground)(STORE_REVIEW_PAGE);
    render(session.view());
    await saveRating(opts.prefs, answerRequest(await loadRating(opts.prefs), choice));
  }

  /** Whether `card` has a gloss — the reader's, the pack's or its own (add-lingua-review-translation-fallback D1). */
  const glossed = (card: NonNullable<ReturnType<ReviewController["view"]>["card"]>): boolean => !!card.gloss;

  /**
   * What the revealed card's answer offers beside its gloss (add-lingua-review-translation-fallback
   * D1–D3): the translation once asked — at once without a gloss, on the link with one —, the online
   * button under its rule, the link while one of the two could answer, the keep action under a
   * translation, until the reader kept one.
   */
  function assisted(card: NonNullable<ReturnType<ReviewController["view"]>["card"]>): {
    translation?: ReviewTranslation;
    online?: ReviewOnlineOffer;
    askTranslation?: () => void;
    keep?: (gloss: string) => void;
    hint?: ReviewTranslationHint;
  } {
    const { request } = assist;
    const asked = assist.asked || !glossed(card);
    const offered = !!request && !!opts.online?.offered();
    const engine = !!request && !!opts.translator?.(request.request.language);
    // Nothing can translate this card, though something could (D12): the note says what.
    const hint =
      request && !glossed(card) && !engine && !offered ? opts.translationHint?.(request.request.language) : null;
    return {
      ...(hint ? { hint } : {}),
      ...(asked && assist.translation ? { translation: assist.translation } : {}),
      ...(asked && offered ? { online: onlineOffer() } : {}),
      ...(!asked && (offered || engine) ? { askTranslation: () => ask() } : {}),
      ...(assist.kept ? {} : { keep: (gloss: string) => keep(gloss) }),
    };
  }

  /** The engine asked for the current card's translation; its answer repaints only a revealed card. */
  function translate(mine: number, asked: ReviewTranslationRequest): void {
    const translator = opts.translator?.(asked.request.language);
    if (!translator) return;
    assist.translation = { state: "pending" };
    void translateBounded(translator, asked.request).then((answer) => {
      // Its card was answered: shown nowhere. An online answer already shown stays.
      if (mine !== arrival || (assist.translation?.state === "translated" && assist.translation.online)) return;
      assist.translation = answer
        ? { state: "translated", translation: answer, alone: asked.alone }
        : { state: "none" };
      const view = session.view();
      if (view.card?.revealed) render(view);
    });
  }

  /**
   * A card came up, or none did: a card without a gloss has its translation asked at once, before
   * the reveal, so it is there when the reader looks (D2). Its answer repaints only a revealed
   * card: before the reveal, nothing of it shows. A card with a gloss waits for the reader's tap.
   */
  function arrive(card: ReturnType<ReviewController["view"]>["card"]): void {
    const mine = ++arrival;
    const request = card ? cardTranslationRequest(card) : null;
    assist = { mine, request, asked: false, kept: false };
    if (card && request && !glossed(card)) translate(mine, request);
  }

  /** « Traduire la phrase »: the engine asked where a model is ready, the online button shown (D1). */
  function ask(): void {
    if (assist.asked || !assist.request) return;
    assist.asked = true;
    translate(assist.mine, assist.request);
    render(session.view());
  }

  /** The online button's state for the current card: one press per card per review (D2). */
  function onlineOffer(): ReviewOnlineOffer {
    const { online } = assist;
    return { state: online?.state ?? "offered", ...(online?.resetsAt ? { resetsAt: online.resetsAt } : {}), press };
  }

  function press(): void {
    const { mine, request } = assist;
    if (assist.online || !request || !opts.online) return;
    assist.online = { state: "asking" };
    render(session.view());
    const send = opts.online.translate;
    void new Promise<OnlineTranslationResult>((resolve) => resolve(send(onlineReviewRequest(request))))
      .catch((): OnlineTranslationResult => ({ kind: "unavailable" }))
      .then((result) => {
        if (mine !== arrival) return; // its card was answered: shown nowhere
        if (result.kind === "translated") {
          assist.translation = {
            state: "translated",
            translation: result.translation,
            alone: request.alone,
            online: true,
          };
          assist.online = { state: "answered" };
        } else {
          assist.online =
            result.kind === "limit" ? { state: "limit", resetsAt: result.resetsAt } : { state: "unavailable" };
        }
        render(session.view());
      });
  }

  /** Keep `gloss` as the card's gloss (D3): written, the backup saved, the answer showing it, the action gone. */
  function keep(gloss: string): void {
    if (assist.kept) return;
    assist.kept = true;
    assist.asked = true; // the translation stays under the gloss it gave
    void run(() => session.keep(gloss), true, false);
  }

  /** A session starts: the route of its language warmed, so its first card does not wait on a cold engine. */
  function warm(): void {
    if (language) opts.translator?.(language)?.warm?.(language);
  }
  /** A count of the summary — « 12 carte(s) » — its figure in bold where its message puts it. */
  const counted = (n: number, forms: PluralForms): Node[] => {
    const holder = el("span");
    renderAround(holder, plural(interfaceLanguage, n, forms, NODE_SLOT), bold(formatCount(interfaceLanguage, n)));
    return [...holder.childNodes];
  };

  container.append(filterRow, summary, toSort, review);

  /** The filter's segments: one per accepted language, the review's own active. */
  function drawFilter(): void {
    filterRow.hidden = languages.length < 2;
    filterRow.replaceChildren(
      ...languages.map((choice) => {
        const b = el("button");
        b.type = "button";
        b.textContent = languageName(interfaceLanguage, choice);
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

  // What a review would hold today, new words beyond the day's allowance left out: the figure the
  // popup and the toolbar icon show too (add-lingua-comeback-hints D1).
  async function refreshSummary(): Promise<void> {
    const now = opts.now();
    // Preferences that cannot be read (a page torn down) count with the default allowance.
    const newPerDay = await loadNewWordsPerDay(opts.prefs).catch(() => DEFAULT_NEW_WORDS_PER_DAY);
    const allowance = { newPerDay, dayStart: localDayStart(now) };
    const [deck, due] = [await port.deckCount(only()), await port.reviewableCount(now, only(), allowance)];
    summary.replaceChildren(...counted(deck, copy.deckCards), copy.summarySeparator, ...counted(due, copy.dueCards));
  }

  /** The count of words to sort beside a book; hidden away from one, or at nothing to sort. */
  async function refreshToSort(): Promise<void> {
    const n = await (opts.toSort?.() ?? Promise.resolve(0)).catch(() => 0);
    toSort.hidden = n <= 0;
    toSort.textContent = n > 0 ? plural(interfaceLanguage, n, copy.toSort) : "";
  }

  async function run(
    produce: () => Promise<ReturnType<ReviewController["view"]>>,
    changed: boolean,
    arrives = true,
  ): Promise<void> {
    const view = await produce();
    if (arrives) arrive(view.card);
    // Preferences that cannot be read never hold the end back: no request then.
    await countEnd(view).catch(() => {});
    if (changed) {
      await persist();
      await refreshSummary();
    }
    render(view);
  }

  const actions: ReviewActions = {
    // A session starts with the focus in the card, so the keys answer it at once.
    start: () => {
      warm();
      newSession();
      void run(() => session.start(only()), false).then(() => review.focus({ preventScroll: true }));
    },
    // Nothing due: practise the cards already reviewed, no schedule moved (add-lingua-review-practice).
    practice: () => {
      warm();
      newSession();
      void run(() => session.start(only(), true), false).then(() => review.focus({ preventScroll: true }));
    },
    reveal: () => void run(() => session.reveal(), false, false),
    grade: (rating) => void run(() => session.grade(rating), true),
    markKnown: () => void run(() => session.markKnown(), true),
    ignore: () => void run(() => session.ignore(), true),
    // Ending the session (add-lingua-review-stop-session D4): the end every session has — the backup
    // saved and the counts read with no session under way, what a host's end-of-session save expects.
    end: () => void run(() => session.end(), true),
  };

  /** A fresh controller over the engine as it now is: the idle card and the counts. */
  async function restart(): Promise<void> {
    session = controller();
    newSession();
    arrive(null);
    // The profile came with the state: a language may have joined or left the filter.
    await readLanguages();
    await refreshSummary();
    render(session.view());
  }

  // Keep in sync with changes made elsewhere (a reading gesture, a reset or a restore in
  // Réglages, or another surface), unless it is our own echo. Mid-review, another surface's
  // change waits — unless it already ended this engine's session: a restore made in this very
  // page (Réglages › Données beside the review) drops the session it was walking.
  // The Cymbra Lingua app has no other surface writing its store (add-lingua-apple-review D1).
  const watch = __APP__ ? () => () => {} : watchBackup;
  const unwatch = watch(area, (backup) => {
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
      await refreshToSort();
      render(session.view());
    },
    reviewing: () => session.view().phase === "reviewing",
    pageChanged: () => {
      followed = undefined;
    },
    destroy: () => {
      unwatch();
      speaker.stop();
      alive.abort();
    },
  };
}

/** What the current card's answer holds beyond its gloss, for one arrival. */
interface Assist {
  /** The arrival it belongs to. */
  mine: number;
  /** What the card's translation asks — engine and online alike —, or null when it asks nothing. */
  request: ReviewTranslationRequest | null;
  /** The reader tapped « Traduire la phrase » (a card without a gloss needs no tap). */
  asked: boolean;
  /** The reader kept a translation: the action is gone. */
  kept: boolean;
  translation?: ReviewTranslation;
  /** The online request, once pressed: never sent twice for this arrival. */
  online?: { state: "asking" | "answered" | "unavailable" | "limit"; resetsAt?: number };
}

function bold(text: string): HTMLElement {
  const b = document.createElement("b");
  b.textContent = text;
  return b;
}
