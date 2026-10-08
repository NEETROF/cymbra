import type { InterfaceLanguage } from "../i18n/index.ts";
import { accountSetting as enAccountSetting } from "../i18n/en/account-setting.ts";
import { colours as enColours } from "../i18n/en/colours.ts";
import { display as enDisplay } from "../i18n/en/display.ts";
import { settings as enSettings } from "../i18n/en/settings.ts";
import { sync as enSync } from "../i18n/en/sync.ts";
import { translation as enTranslation } from "../i18n/en/translation.ts";
import { accountSetting as esAccountSetting } from "../i18n/es/account-setting.ts";
import { colours as esColours } from "../i18n/es/colours.ts";
import { display as esDisplay } from "../i18n/es/display.ts";
import { settings as esSettings } from "../i18n/es/settings.ts";
import { sync as esSync } from "../i18n/es/sync.ts";
import { translation as esTranslation } from "../i18n/es/translation.ts";
import { accountSetting as frAccountSetting } from "../i18n/fr/account-setting.ts";
import { colours as frColours } from "../i18n/fr/colours.ts";
import { display as frDisplay } from "../i18n/fr/display.ts";
import { settings as frSettings } from "../i18n/fr/settings.ts";
import { sync as frSync } from "../i18n/fr/sync.ts";
import { translation as frTranslation } from "../i18n/fr/translation.ts";

// Réglages' copy (localise-lingua-settings D1): the six catalogue modules of the view and its blocks,
// picked by the interface language its host hands `mountSettings` — the popup and the side panel the
// one they read with their preferences, the drawer the one the reading session handed it. The view
// hands each block its own module, and the language where the block writes a figure or a date. A
// block or a function handed a language and no module reads that language's; handed neither, French,
// so a spec, or a host with no language yet (the onboarding page), mounts what it mounted before.
// This is the one place that holds all three languages of the six.

export type SettingsModule = typeof frSettings;
export type ColoursCopy = typeof frColours;
export type DisplayCopy = typeof frDisplay;
export type TranslationCopy = typeof frTranslation;
export type AccountSettingCopy = typeof frAccountSetting;
export type SyncCopy = typeof frSync;

export interface SettingsCopy {
  settings: SettingsModule;
  colours: ColoursCopy;
  display: DisplayCopy;
  translation: TranslationCopy;
  accountSetting: AccountSettingCopy;
  sync: SyncCopy;
}

const COPY: Record<InterfaceLanguage, SettingsCopy> = {
  fr: {
    settings: frSettings,
    colours: frColours,
    display: frDisplay,
    translation: frTranslation,
    accountSetting: frAccountSetting,
    sync: frSync,
  },
  en: {
    settings: enSettings,
    colours: enColours,
    display: enDisplay,
    translation: enTranslation,
    accountSetting: enAccountSetting,
    sync: enSync,
  },
  es: {
    settings: esSettings,
    colours: esColours,
    display: esDisplay,
    translation: esTranslation,
    accountSetting: esAccountSetting,
    sync: esSync,
  },
};

/** Réglages' modules for the interface language. */
export function settingsCopy(language: InterfaceLanguage): SettingsCopy {
  return COPY[language];
}
