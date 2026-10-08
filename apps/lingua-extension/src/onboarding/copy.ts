import { onboarding as enOnboarding } from "../i18n/en/onboarding.ts";
import { onboarding as esOnboarding } from "../i18n/es/onboarding.ts";
import { onboarding as frOnboarding } from "../i18n/fr/onboarding.ts";
import { studiedLanguages as enStudiedLanguages } from "../i18n/en/studied-languages.ts";
import { studiedLanguages as esStudiedLanguages } from "../i18n/es/studied-languages.ts";
import { studiedLanguages as frStudiedLanguages } from "../i18n/fr/studied-languages.ts";
import { DEFAULT_INTERFACE_LANGUAGE, type InterfaceLanguage } from "../i18n/index.ts";
import type { StudiedLanguagesCopy } from "../reading/settings-copy.ts";

// The onboarding's copy (localise-lingua-account-onboarding D1): the catalogue's `onboarding` module
// in the interface language, for the page's static text and the level rows.

/** The onboarding's copy: the catalogue's `onboarding` module, in the interface language. */
export type OnboardingCopy = typeof frOnboarding;

const ONBOARDING_COPY: Record<InterfaceLanguage, OnboardingCopy> = {
  fr: frOnboarding,
  en: enOnboarding,
  es: esOnboarding,
};

/** The onboarding's module for the interface language (French when none is given). */
export function onboardingCopy(language: InterfaceLanguage = DEFAULT_INTERFACE_LANGUAGE): OnboardingCopy {
  return ONBOARDING_COPY[language];
}

const STUDIED_LANGUAGES_COPY: Record<InterfaceLanguage, StudiedLanguagesCopy> = {
  fr: frStudiedLanguages,
  en: enStudiedLanguages,
  es: esStudiedLanguages,
};

/**
 * The languages step's copy, Réglages' studied-languages block (localise-lingua-settings) in the
 * interface language — its three modules alone, not Réglages' seven.
 */
export function studiedLanguagesCopy(language: InterfaceLanguage = DEFAULT_INTERFACE_LANGUAGE): StudiedLanguagesCopy {
  return STUDIED_LANGUAGES_COPY[language];
}
