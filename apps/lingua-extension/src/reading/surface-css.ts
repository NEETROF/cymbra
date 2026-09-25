import drawerCss from "../styles/drawer.css";
import hudCss from "../styles/hud.css";
import popupCss from "../styles/wordpopup.css";
import reviewCss from "../styles/review.css";
import statsCss from "../stats/stats.css";
import settingsCss from "../styles/settings.css";
import tokensCss from "../styles/tokens.css";

// The style sheets of the reading surfaces, as the two hosts of the reading module hand them
// to `ReadingSession`: the content script on a web page, and the reader page on a book. One
// list, so the popup, the drawer and the HUD look the same in both.

export interface SurfaceCss {
  /** The token sheet — the palette and the two `::highlight()` rules — for the read document. */
  tokens: string;
  /** The word popup and the selection card. */
  popup: string;
  /** The drawer: review, statistics and settings. */
  drawer: string;
  /** The in-page HUD pill. */
  hud: string;
}

export const SURFACE_CSS: SurfaceCss = {
  tokens: tokensCss,
  popup: `${tokensCss}\n${popupCss}`,
  drawer: `${tokensCss}\n${reviewCss}\n${statsCss}\n${settingsCss}\n${drawerCss}`,
  hud: `${tokensCss}\n${hudCss}`,
};
