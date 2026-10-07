import type { onboarding as fr } from "../fr/onboarding.ts";

// The onboarding page's copy in English — a draft after the French (src/i18n/README.md).

export const onboarding: typeof fr = {
  pageTitle: "Welcome — Cymbra Lingua",
  heading: "Welcome to Cymbra Lingua",
  lead: "Highlight the words you don't know yet while you read, then review them at the right time — without leaving your page.",
  whichLanguages: "Which languages are you learning?",
  levelNote:
    "Words below your level won't be highlighted. Nothing is presumed until you choose — and you can change it at any time in the settings.",
  toStart: "To get started",
  stepPin: "Pin the Cymbra Lingua icon to the browser's toolbar.",
  stepOpen: "Open a page in the language you're learning and click the icon to analyze it.",
  stepClick: "Click a highlighted word to translate it or add it to your deck.",
  accountTitle: "Find your words everywhere (optional)",
  accountLead:
    "With a Cymbra account — the same as in Cymbra Music — your words, your deck and your statistics sync between your devices. Without an account, everything stays on this device.",
  createAccount: "Create an account",
  later: "Later",
  beginnerChip: "Beginner — starting from scratch",
  levelSaved: (level) => `Level saved: ${level}. You can close this tab and start reading.`,
  beginnerSaved: "Noted — starting from scratch. You can close this tab and start reading.",
};
