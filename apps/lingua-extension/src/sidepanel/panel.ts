import type { LinguaPort } from "../analyzer/port.ts";
import { sidepanel as enSidepanel } from "../i18n/en/sidepanel.ts";
import { sidepanel as esSidepanel } from "../i18n/es/sidepanel.ts";
import { sidepanel as frSidepanel } from "../i18n/fr/sidepanel.ts";
import { fillPageInLanguage, type InterfaceLanguage } from "../i18n/index.ts";
import { mountReview, type ReviewPage } from "../review/review-page.ts";
import type { AsyncStorageArea } from "../state/storage.ts";
import { mountStats } from "../stats/view.ts";

// The side panel's start, as far as Révision and Statistiques go (localise-lingua-review-stats D1):
// it reads the interface language, fills the page's static copy in it, and mounts both views in
// it — whenever they are first asked for. A view asked for before the read has answered waits for
// it, so Révision, mounted once for the panel's lifetime, can never be mounted in French by a
// request that came first. sidepanel.ts wires the rest (the engine, the views' switch, the tabs).

/** The page's static copy by interface language (localise-lingua-reading-surfaces D2). */
const SIDEPANEL_COPY: Record<InterfaceLanguage, typeof frSidepanel> = {
  fr: frSidepanel,
  en: enSidepanel,
  es: esSidepanel,
};

/** What the panel's start is handed: the areas, the engine, and Révision's clock and page. */
export interface PanelDeps {
  /** Preferences (chrome.storage.local): the interface language, the review's last language. */
  prefs: AsyncStorageArea;
  /** The reader's data, owned by the background. */
  store: AsyncStorageArea;
  port: LinguaPort;
  /** Epoch-seconds clock. */
  now: () => number;
  /** The language of the page or book beside the panel, or null away from one. */
  pageLanguage?: () => Promise<string | null>;
}

export interface PanelViews {
  /** The interface language, once read and the page filled in it (French when the read fails). */
  readonly interfaceLanguage: Promise<InterfaceLanguage>;
  /** Révision, mounted into `container` once — in the interface language — and the same page after. */
  review(container: HTMLElement): Promise<ReviewPage>;
  /** Statistiques, mounted into `container` in the interface language. */
  stats(container: HTMLElement): Promise<void>;
}

/** Start the panel's page: the language read and the copy filled now, the views when asked for. */
export function start(document: Document, deps: PanelDeps): PanelViews {
  const interfaceLanguage = fillPageInLanguage(document, deps.prefs, (l) => SIDEPANEL_COPY[l]).then(
    ({ language }) => language,
  );
  let review: Promise<ReviewPage> | null = null;
  return {
    interfaceLanguage,
    review(container) {
      review ??= interfaceLanguage.then((language) =>
        mountReview(container, deps.port, deps.store, {
          now: deps.now,
          prefs: deps.prefs,
          pageLanguage: deps.pageLanguage,
          interfaceLanguage: language,
        }),
      );
      return review;
    },
    async stats(container) {
      await mountStats(container, deps.port, deps.store, undefined, await interfaceLanguage);
    },
  };
}
