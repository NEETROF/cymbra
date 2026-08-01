# Changelog

## [0.3.0](https://github.com/NEETROF/cymbra/compare/back-office-v0.2.1...back-office-v0.3.0) (2026-08-01)


### Features

* **back-office:** download a catalog score's MusicXML from the table ([#155](https://github.com/NEETROF/cymbra/issues/155)) ([db3b85d](https://github.com/NEETROF/cymbra/commit/db3b85d4db984c3ef2ae29b6d185ad02667aa8c1))
* **feature-flags:** add shared runtime feature-flag & config platform ([#152](https://github.com/NEETROF/cymbra/issues/152)) ([a8e487b](https://github.com/NEETROF/cymbra/commit/a8e487bc02ab378a016ce74d29f93be1edc86c29))
* persist and sync the account language preference ([#153](https://github.com/NEETROF/cymbra/issues/153)) ([30ff982](https://github.com/NEETROF/cymbra/commit/30ff9820c47707b02910c62115b7f74dccd32ab4))
* **roles:** scope-matched role administration across global/music/live ([#154](https://github.com/NEETROF/cymbra/issues/154)) ([d03fa29](https://github.com/NEETROF/cymbra/commit/d03fa29fa0eaa477ea56c9475bbd0c5786a1efec))

## [0.2.1](https://github.com/NEETROF/cymbra/compare/back-office-v0.2.0...back-office-v0.2.1) (2026-07-30)


### Bug Fixes

* **deploy:** mount SoundFont warm-cache on server + harden bo cache ([#143](https://github.com/NEETROF/cymbra/issues/143)) ([004caec](https://github.com/NEETROF/cymbra/commit/004caec3cfe1913323ff03f950f27d9748b55af8))

## [0.2.0](https://github.com/NEETROF/cymbra/compare/back-office-v0.1.0...back-office-v0.2.0) (2026-07-30)


### Features

* **auth:** browser HttpOnly cookie sessions for the back office ([#114](https://github.com/NEETROF/cymbra/issues/114)) ([ec7b723](https://github.com/NEETROF/cymbra/commit/ec7b723ea5806260b06bf110b4621fa342e614b2))
* **auth:** session revocation — admin cut-off (BO) + sign-out-everywhere API ([#116](https://github.com/NEETROF/cymbra/issues/116)) ([e26eb23](https://github.com/NEETROF/cymbra/commit/e26eb23903c23f59ce6ff499f8dd2384ef61a758))
* **back-office:** Cloudflare Pages deploy config + moderator onboarding docs ([#122](https://github.com/NEETROF/cymbra/issues/122)) ([02b2ef4](https://github.com/NEETROF/cymbra/commit/02b2ef41046ab80231414b7f5bb42da2632183c8))
* **back-office:** decoded gRPC-web console trace with prod opt-in flag ([#125](https://github.com/NEETROF/cymbra/issues/125)) ([9e5c88a](https://github.com/NEETROF/cymbra/commit/9e5c88a7752f042af1acc98981a27b567079b51c))
* **back-office:** version via release-please + Google & Apple sign-in ([#132](https://github.com/NEETROF/cymbra/issues/132)) ([d349f6f](https://github.com/NEETROF/cymbra/commit/d349f6f8e7a5f3b309f81e0ff3dc8f2f07e409f5))
* **back-office:** Vue 3 moderation console + account directory ([#108](https://github.com/NEETROF/cymbra/issues/108)) ([aa158fd](https://github.com/NEETROF/cymbra/commit/aa158fd6615a6c27e383e66f1cc2b08a548b6f56))
* **back-office:** wasm notation preview, audio playback & review mode ([#138](https://github.com/NEETROF/cymbra/issues/138)) ([f37198a](https://github.com/NEETROF/cymbra/commit/f37198a52443be697f49b3d035633e6a3b2ded57))
* **moderation:** wire community re-review flag into the back-office queue ([#131](https://github.com/NEETROF/cymbra/issues/131)) ([29f5b1b](https://github.com/NEETROF/cymbra/commit/29f5b1b1d2bc163261cfd500661a5226f4ecacc2))
* **music:** moderator/admin editing of catalog curatorial metadata ([#141](https://github.com/NEETROF/cymbra/issues/141)) ([b822ccf](https://github.com/NEETROF/cymbra/commit/b822ccf88ba6a4d5d01883e06cad7233c4eea491))


### Bug Fixes

* **back-office:** auto-reload once on stale-deploy chunk load error ([#126](https://github.com/NEETROF/cymbra/issues/126)) ([92f40bf](https://github.com/NEETROF/cymbra/commit/92f40bf2059bbc0b60fe0508f98dfae572ea8116))
* **back-office:** paginate the catalog & queue tables ([#124](https://github.com/NEETROF/cymbra/issues/124)) ([443abf4](https://github.com/NEETROF/cymbra/commit/443abf4e3067387f0c82fb3560c090c4e535314e))
* **back-office:** revert stale-deploy auto-reload — caused a reload loop ([#127](https://github.com/NEETROF/cymbra/issues/127)) ([bd34872](https://github.com/NEETROF/cymbra/commit/bd3487250ecb92c106bf06f7a9ccd4f81c259784))
