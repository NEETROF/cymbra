# Tasks

## 1. The listings

- [ ] 1.1 `apps/lingua-extension/STORE-LISTING.md`, in the wording the owner settles (design D2):
  - the listing language line;
  - the summary, quoted from `manifest.json`;
  - the French and English descriptions;
  - the single purpose and the remote-code answer;
  - the test instructions, within 1,000 characters, labels quoted from the source.
- [ ] 1.2 `apps/lingua-apple/STORE-LISTING.md`, in the wording the owner settles (design D3):
  - the subtitle, the promotional text, the keywords and the description, each with its count;
  - a « What's New » paragraph for the release that ships Spanish;
  - no « bêta » and nothing about price.

## 2. Checks

- [ ] 2.1 Every count is within its store's limit:
  - summary 112, subtitle 30, promotional text 170, keywords 100;
  - App Store description 4,000, test instructions 1,000;
  - the counts written in the files match the text.
- [ ] 2.2 `openspec validate add-lingua-spanish-listings --strict` passes.

## 3. Release (owner)

- [ ] 3.1 The listings are pasted into the Chrome Web Store, addons.mozilla.org and App Store Connect dashboards with the release that ships Spanish, when change 30's pages are deployed.
