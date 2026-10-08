import type { InterfaceLanguage } from "../i18n/index.ts";
import { card as enCard } from "../i18n/en/card.ts";
import { drawer as enDrawer } from "../i18n/en/drawer.ts";
import { hud as enHud } from "../i18n/en/hud.ts";
import { selection as enSelection } from "../i18n/en/selection.ts";
import { card as esCard } from "../i18n/es/card.ts";
import { drawer as esDrawer } from "../i18n/es/drawer.ts";
import { hud as esHud } from "../i18n/es/hud.ts";
import { selection as esSelection } from "../i18n/es/selection.ts";
import { card as frCard } from "../i18n/fr/card.ts";
import { drawer as frDrawer } from "../i18n/fr/drawer.ts";
import { hud as frHud } from "../i18n/fr/hud.ts";
import { selection as frSelection } from "../i18n/fr/selection.ts";
import type { DrawerCopy } from "./drawer.ts";
import type { HudCopy } from "./hud.ts";
import type { SelectionCopy } from "./selection-card.ts";
import type { CardCopy } from "./wordpopup.ts";

// The copy of the surfaces a reading session builds (localise-lingua-reading-surfaces D1): the HUD,
// the drawer, the word card and the selection card, picked by the interface language the content
// script or the reader page read before `new ReadingSession(…)`, and handed to the session with the
// language. Each surface holds its own French module as its default — a session built without any
// reads French — and this is the one place that holds all three languages of the four.

export interface ReadingCopy {
  hud: HudCopy;
  drawer: DrawerCopy;
  card: CardCopy;
  selection: SelectionCopy;
}

const COPY: Record<InterfaceLanguage, ReadingCopy> = {
  fr: { hud: frHud, drawer: frDrawer, card: frCard, selection: frSelection },
  en: { hud: enHud, drawer: enDrawer, card: enCard, selection: enSelection },
  es: { hud: esHud, drawer: esDrawer, card: esCard, selection: esSelection },
};

/** The reading surfaces' modules for the interface language. */
export function readingCopy(language: InterfaceLanguage): ReadingCopy {
  return COPY[language];
}
