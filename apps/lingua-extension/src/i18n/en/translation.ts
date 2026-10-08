import type { translation as fr } from "../fr/translation.ts";

// The translation setting's copy in English — a draft after the French (src/i18n/README.md).
// A size comes formatted ("25.8") and takes its unit here.

export const translation: typeof fr = {
  toggle: "Extended translation",
  attribution: "Translation model: Firefox Translations (Mozilla), MPL 2.0 license.",
  ready: "Ready: your selections are translated on this device.",
  removed: "The browser removed the model from this device. It has to be downloaded again.",
  missing: "A model is missing for one of your languages.",
  cancel: "Cancel",
  download: "Download",
  retry: "Try again",
  resume: "Resume",
  again: "Download again",
  megabytes: (amount) => `${amount} MB`,
  theModel: "the model",
  memoryPivot: "about 340 MB",
  memorySingle: "about 200 MB",
  cost: (download, memory) =>
    `Translates your sentences on this device, sending nothing. Downloads ${download} once, then uses ` +
    `${memory} of memory while translating. This setting applies to this device only.`,
  failedNetwork: "The download failed: no connection. Try again once online.",
  failedUnavailable: "The download failed: the server isn't answering. Try again later.",
  failedNotTheModel: "The download failed: the file received isn't the right model. Try again later.",
  failedStorageSized: (size) => `Not enough room on this device for the model (${size}).`,
  failedStorage: "Not enough room on this device for the model.",
  failedUnknown: "The download failed. Try again later.",
  progress: (received, total) => ` ${received} of ${total}`,
  downloading: (progress) => `Downloading the model…${progress}`,
  interruptedAt: (progress) => `Download interrupted.${progress}`,
  missingSized: (size) => `A model is missing for one of your languages. ${size} to download.`,
};
