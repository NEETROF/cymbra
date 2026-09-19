# Tasks

## 1. Versioning

- [x] 1.1 Add `lingua-extension` (`apps/lingua-extension`, release-type `node`) and `lingua-apple` (`apps/lingua-apple`, release-type `simple`) to `release-please-config.json`, with `extra-files` mirroring the version into `apps/lingua-extension/manifest.json` (`$.version`)
- [x] 1.2 Seed both components in `.release-please-manifest.json` at the version the tree carries, and cut the history at the commit that introduces them (D9)
- [x] 1.3 Add `apps/lingua-apple/version.txt`, matching the seeded version
- [x] 1.4 Guard in `lingua-extension-check`: `package.json` and `manifest.json` agree, and the version is dot-separated integers with no suffix — with its own unit test, failing on each of the two shapes it rejects

## 2. The extension's release lane

- [x] 2.1 `.github/workflows/lingua-extension-release.yml`: on a `lingua-extension-v*` tag, plus a dispatch that builds and publishes nothing (the only way to validate a source change before tagging)
- [x] 2.2 Refuse a version the stores would not take, before building (D3)
- [x] 2.3 Production build of the chromium and firefox variants: the real EN→FR pack (cached sources, size-checked as the Apple lane does) and `LINGUA_GRPC_WEB_URL=https://api.cymbra.app`; fail on a test pack or a non-production endpoint
- [x] 2.4 Zip each variant as `cymbra-lingua-<variant>-<version>.zip` and attach both to the GitHub Release
- [x] 2.5 Chrome Web Store: upload the chromium package to the existing item, then publish it
- [x] 2.6 AMO: build the source archive (extension sources, the `lingua-*` crates, and the rebuild instructions — no pack, D5) and run `web-ext sign --channel listed --upload-source-code`
- [x] 2.7 Secret guard at the top of the job, naming what is missing, as `lingua-apple-release` does for its profiles
- [x] 2.8 The run's summary states the version is in review at each store, not delivered

## 3. The Apple lane

- [x] 3.1 `lingua-apple-release.yml`: add the `lingua-apple-v*` tag trigger, checking out the tag
- [x] 3.2 Read `version.txt`, assert it matches the tag, and pass `MARKETING_VERSION` to both `xcodebuild` archives beside `CURRENT_PROJECT_VERSION`
- [x] 3.3 Deliver without asking on a tag; keep the `deliver` opt-in for a dispatch
- [x] 3.4 Extend `tool/test_ci_release_signing.py`'s suite, or add a sibling test, for the version-vs-tag check

## 4. Documentation

- [x] 4.1 `apps/lingua-extension/README.md`: a release section — the tag, what the lane builds, the secrets, and what "published" means
- [x] 4.2 `apps/lingua-apple/README.md`: the tag trigger and the version source, next to the existing signing secrets
- [x] 4.3 The workflow headers carry their own required-secrets list, as the Apple lane's does

## 5. First release (needs the store accounts)

- [ ] 5.1 Create the Chrome Web Store item and fill its listing (screenshots, description, privacy answers); note its item id
- [ ] 5.2 Create the AMO listing for `lingua@cymbra.app`
- [ ] 5.3 Create the credentials and store them as repository secrets: the Chrome Web Store OAuth client id/secret + refresh token and the item id; the AMO JWT issuer and secret
- [ ] 5.7 Once the item exists, carry its id everywhere it is needed: `https://<id>.chromiumapp.org/` as a redirect URI on the Google OAuth client, `chrome-extension://<id>` in the production `CYMBRA_ALLOWED_WEB_ORIGINS`, and the two repository variables `LINGUA_GOOGLE_CLIENT_ID` / `LINGUA_APPLE_CLIENT_ID`
- [ ] 5.4 Read the first Release PR's changelog before merging it (D9), then merge and verify the GitHub Release carries both zips; submitting to the stores is a separate dispatch with that tag and `publish` ticked (D12)
- [ ] 5.6 **Remove `release-as` from `apps/lingua-extension` in `release-please-config.json`** once `lingua-extension-v1.0.0` is tagged — it is sticky, and left in place it pins every later release to 1.0.0 (D10)
- [ ] 5.5 Tag the Apple app and verify the delivered build reports the tag's version in App Store Connect
