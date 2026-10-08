import { onboarding as enOnboarding } from "../i18n/en/onboarding.ts";
import { onboarding as esOnboarding } from "../i18n/es/onboarding.ts";
import { onboarding as frOnboarding } from "../i18n/fr/onboarding.ts";
import { DEFAULT_INTERFACE_LANGUAGE, type InterfaceLanguage } from "../i18n/index.ts";

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
