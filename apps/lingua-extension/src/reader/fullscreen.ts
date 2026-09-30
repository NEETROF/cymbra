// The reader page in fullscreen (add-lingua-reader-fullscreen): the browser's bars hidden while
// a book is read. The page's root goes fullscreen, not the book — the word popup and the drawer
// hang from `documentElement`, and only the fullscreen element's subtree is painted (design D1).
// iPadOS before 16.4 only knows the `webkit` names; Safari on iPhone knows neither for a page.

/** What the reader page needs of the browser's fullscreen. */
export interface FullscreenHost {
  /** Whether this browser can put the page in fullscreen at all. */
  readonly available: boolean;
  /** Whether the page is in fullscreen now. */
  active(): boolean;
  enter(): Promise<void>;
  exit(): Promise<void>;
  /** Call back whenever the page enters or leaves fullscreen, whoever asked. */
  onChange(listener: () => void): void;
}

/** The part of a document the fullscreen API lives on, standard or `webkit`. */
export interface FullscreenDocument {
  documentElement: {
    requestFullscreen?: () => Promise<void>;
    webkitRequestFullscreen?: () => void;
  };
  fullscreenEnabled?: boolean;
  webkitFullscreenEnabled?: boolean;
  fullscreenElement?: Element | null;
  webkitFullscreenElement?: Element | null;
  exitFullscreen?: () => Promise<void>;
  webkitExitFullscreen?: () => void;
  addEventListener(type: string, listener: () => void): void;
}

/** The fullscreen of `doc`'s page, over whichever API the browser has. */
export function documentFullscreen(doc: FullscreenDocument): FullscreenHost {
  const root = doc.documentElement;
  const standard = typeof root.requestFullscreen === "function";
  const webkit = !standard && typeof root.webkitRequestFullscreen === "function";
  const available = standard ? doc.fullscreenEnabled === true : webkit && doc.webkitFullscreenEnabled === true;
  return {
    available,
    active: () => (doc.fullscreenElement ?? doc.webkitFullscreenElement ?? null) !== null,
    enter: async () => {
      if (standard) await root.requestFullscreen!();
      else if (webkit) root.webkitRequestFullscreen!();
    },
    exit: async () => {
      if (typeof doc.exitFullscreen === "function") await doc.exitFullscreen();
      else if (typeof doc.webkitExitFullscreen === "function") doc.webkitExitFullscreen();
    },
    onChange: (listener) => {
      doc.addEventListener(standard ? "fullscreenchange" : "webkitfullscreenchange", listener);
    },
  };
}
