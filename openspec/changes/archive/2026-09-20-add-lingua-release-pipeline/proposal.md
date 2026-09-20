## Why

Every deployable in this repo is versioned by release-please from Conventional Commits and shipped from its tag — music, backend, back office, site. Lingua is the exception, and it is the one about to reach users.

The Apple app ships from a dispatch: its version is `MARKETING_VERSION = 1.0`, written by hand in eight places in the Xcode project, and its build number is the workflow's run number. Nothing ties a TestFlight build to a commit range or to a changelog; the eleven builds of the dogfooding pass are distinguishable only by their build numbers.

The extension has never shipped at all. `lingua-extension-check` builds the chromium and firefox variants on every pull request to prove they assemble, then throws them away. Publishing one today means building it on a laptop and dragging a zip into two store dashboards — which is exactly the step that, done by hand, ships the wrong pack or a `localhost` endpoint.

## What Changes

- **Two release-please components**, so the two artefacts move at their own pace: `lingua-extension` (`apps/lingua-extension`) and `lingua-apple` (`apps/lingua-apple`), tagged `lingua-extension-v*` and `lingua-apple-v*`.
- **One version source per component.** The extension's is `package.json`, mirrored into the single `manifest.json` that all three variants fold from, with a check that the two agree and that the version is a shape Chrome accepts. The Apple app's is a `version.txt` the lane passes to `xcodebuild`, the way it already passes the build number — the committed project keeps no release state.
- **A `lingua-extension-release` workflow**, on a `lingua-extension-v*` tag: the production build (the real EN→FR pack, `https://api.cymbra.app`) of the chromium and firefox variants, attached to the GitHub Release **and published** — Chrome Web Store through its API, AMO through `web-ext sign --channel listed` with the source archive Mozilla requires for generated code.
- **The Apple lane gains a tag trigger.** A `lingua-apple-v*` tag checks out that tag, stamps `MARKETING_VERSION` from `version.txt` and delivers to App Store Connect. Dispatch keeps today's behaviour, including its `deliver` opt-in.
- **The safari variant is not published on its own.** It ships inside the Apple app, built from the same commit by the Apple lane — so the extension release publishes two of the three variants, by design.
- **Published means submitted.** Both stores review; the workflow succeeds when the version is accepted for review, and says so rather than claiming users have it.

## Capabilities

### Modified Capabilities
- `lingua-browser-extension`: adds "Versioned releases" and "Store publication from a tag" — where the version comes from, what a release build contains, and what reaching a store means.

### New Capabilities
- `lingua-apple-app`: adds "Versioned App Store releases" — the shipped version comes from the tag, not from the Xcode project. (The capability itself is created by `add-lingua-apple`; this change only adds a requirement to it.)

## Impact

- `release-please-config.json`, `.release-please-manifest.json`: two components.
- `apps/lingua-extension`: `manifest.json` mirrored by release-please, a version guard in `lingua-extension-check`, a release section in the README.
- `apps/lingua-apple`: `version.txt`, and `lingua-apple-release.yml` gains the tag trigger and the version stamp.
- `.github/workflows/lingua-extension-release.yml`: new.
- **Needs credentials that do not exist yet**, and they are not ours to create: a Chrome Web Store item plus an OAuth client and refresh token for its API, and an AMO listing plus a JWT issuer/secret. Until they are stored as secrets the workflow refuses to run, the way the Apple lane refuses without its profiles.
- **Out of scope**: the store listings' own content (screenshots, descriptions, privacy answers), which is filled in each dashboard; unlisted/self-hosted Firefox distribution; publishing the safari variant anywhere but inside the Apple app.
