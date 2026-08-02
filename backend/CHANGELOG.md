# Changelog

## [0.11.0](https://github.com/NEETROF/cymbra/compare/backend-v0.10.1...backend-v0.11.0) (2026-08-02)


### Features

* selectable instrument sounds — catalog, back-office management, and in-app picker ([#164](https://github.com/NEETROF/cymbra/issues/164)) ([548b252](https://github.com/NEETROF/cymbra/commit/548b252577a7b48587d6d81ab32571faa9763a40))


### Bug Fixes

* **catalog:** Mutopia titles from .ly headers + back-office id display ([#162](https://github.com/NEETROF/cymbra/issues/162)) ([19d6818](https://github.com/NEETROF/cymbra/commit/19d68183313d40e0886424e33715796e0aba7194))

## [0.10.1](https://github.com/NEETROF/cymbra/compare/backend-v0.10.0...backend-v0.10.1) (2026-08-01)


### Bug Fixes

* **flags:** keep the back-office console showing every app's flags ([#158](https://github.com/NEETROF/cymbra/issues/158)) ([28a181f](https://github.com/NEETROF/cymbra/commit/28a181f673e8ff41dbf6b851f98b227d6765dde3))

## [0.10.0](https://github.com/NEETROF/cymbra/compare/backend-v0.9.1...backend-v0.10.0) (2026-08-01)


### Features

* **account:** link sign-in identities across providers (Connected accounts) ([#147](https://github.com/NEETROF/cymbra/issues/147)) ([abe4913](https://github.com/NEETROF/cymbra/commit/abe4913df1c30528079474f764cdab286529083c))
* **auth:** verify email before binding a set-password credential ([#150](https://github.com/NEETROF/cymbra/issues/150)) ([452d93a](https://github.com/NEETROF/cymbra/commit/452d93a746a80f9eccc5b5c2bf0a34fcd0231933))
* **email:** brand and localize transactional emails with the Cymbra ID design system ([#149](https://github.com/NEETROF/cymbra/issues/149)) ([84211ba](https://github.com/NEETROF/cymbra/commit/84211ba1d46ef9b93d0aacd3f3ad7ed69f7e9394))
* **feature-flags:** add shared runtime feature-flag & config platform ([#152](https://github.com/NEETROF/cymbra/issues/152)) ([a8e487b](https://github.com/NEETROF/cymbra/commit/a8e487bc02ab378a016ce74d29f93be1edc86c29))
* **music:** per-user catalog access limits to prevent token scraping ([#156](https://github.com/NEETROF/cymbra/issues/156)) ([654bfc0](https://github.com/NEETROF/cymbra/commit/654bfc0abf359ef05493e4ef48f677e9ec7d99b2))
* persist and sync the account language preference ([#153](https://github.com/NEETROF/cymbra/issues/153)) ([30ff982](https://github.com/NEETROF/cymbra/commit/30ff9820c47707b02910c62115b7f74dccd32ab4))
* **roles:** scope-matched role administration across global/music/live ([#154](https://github.com/NEETROF/cymbra/issues/154)) ([d03fa29](https://github.com/NEETROF/cymbra/commit/d03fa29fa0eaa477ea56c9475bbd0c5786a1efec))

## [0.9.1](https://github.com/NEETROF/cymbra/compare/backend-v0.9.0...backend-v0.9.1) (2026-07-30)


### Bug Fixes

* **backend:** copy backfill-titles into the runtime image ([#144](https://github.com/NEETROF/cymbra/issues/144)) ([b71021a](https://github.com/NEETROF/cymbra/commit/b71021ad73392917a4211eb58815bcfece7615c5))
* **deploy:** mount SoundFont warm-cache on server + harden bo cache ([#143](https://github.com/NEETROF/cymbra/issues/143)) ([004caec](https://github.com/NEETROF/cymbra/commit/004caec3cfe1913323ff03f950f27d9748b55af8))

## [0.9.0](https://github.com/NEETROF/cymbra/compare/backend-v0.8.0...backend-v0.9.0) (2026-07-30)


### Features

* **moderation:** wire community re-review flag into the back-office queue ([#131](https://github.com/NEETROF/cymbra/issues/131)) ([29f5b1b](https://github.com/NEETROF/cymbra/commit/29f5b1b1d2bc163261cfd500661a5226f4ecacc2))
* **music:** moderator/admin editing of catalog curatorial metadata ([#141](https://github.com/NEETROF/cymbra/issues/141)) ([b822ccf](https://github.com/NEETROF/cymbra/commit/b822ccf88ba6a4d5d01883e06cad7233c4eea491))
* **music:** play-activity heatmap, reliable stats sync, and public profiles ([#129](https://github.com/NEETROF/cymbra/issues/129)) ([d124d0d](https://github.com/NEETROF/cymbra/commit/d124d0d2604c73c0a2c867389f5ec3719a4d6256))
* **rating:** let the community rate pending scores in the deck ([#133](https://github.com/NEETROF/cymbra/issues/133)) ([885bf51](https://github.com/NEETROF/cymbra/commit/885bf5155ea159005c57753e677b90fac78f1530))
* **soundfont:** serve SoundFonts from a private bucket via an authenticated route ([#137](https://github.com/NEETROF/cymbra/issues/137)) ([6cbfdf3](https://github.com/NEETROF/cymbra/commit/6cbfdf3da58538e0406fe83d70f4a5f2463de51a))
* swipe + star score rating deck, backend, and in-card preview ([#128](https://github.com/NEETROF/cymbra/issues/128)) ([e4ced64](https://github.com/NEETROF/cymbra/commit/e4ced64df93ee207fc4d2650b601352d8bc5401c))


### Bug Fixes

* **crawler:** use embedded work-title for scores; backfill existing titles ([#139](https://github.com/NEETROF/cymbra/issues/139)) ([204af65](https://github.com/NEETROF/cymbra/commit/204af6598e8a222afb2093b6dbe0cb5ccffa7269))

## [0.8.0](https://github.com/NEETROF/cymbra/compare/backend-v0.7.0...backend-v0.8.0) (2026-07-28)


### Features

* **auth:** browser HttpOnly cookie sessions for the back office ([#114](https://github.com/NEETROF/cymbra/issues/114)) ([ec7b723](https://github.com/NEETROF/cymbra/commit/ec7b723ea5806260b06bf110b4621fa342e614b2))
* **auth:** session revocation — admin cut-off (BO) + sign-out-everywhere API ([#116](https://github.com/NEETROF/cymbra/issues/116)) ([e26eb23](https://github.com/NEETROF/cymbra/commit/e26eb23903c23f59ce6ff499f8dd2384ef61a758))
* **back-office:** Vue 3 moderation console + account directory ([#108](https://github.com/NEETROF/cymbra/issues/108)) ([aa158fd](https://github.com/NEETROF/cymbra/commit/aa158fd6615a6c27e383e66f1cc2b08a548b6f56))
* **catalog:** gate hub on moderation status; hide unvalidated scores ([#106](https://github.com/NEETROF/cymbra/issues/106)) ([99c0243](https://github.com/NEETROF/cymbra/commit/99c02436a74612bec9e4a4f9fe684e53e89a4c38))
* **moderation:** back-office backend — roles, evaluate, role admin, gRPC-web ([#107](https://github.com/NEETROF/cymbra/issues/107)) ([925b4e8](https://github.com/NEETROF/cymbra/commit/925b4e87a828fdef47be3b35b0df39f21ee24f69))


### Bug Fixes

* **catalog:** queue sort crashed — `needs_review` emitted an invalid ORDER BY ([#111](https://github.com/NEETROF/cymbra/issues/111)) ([5f3b2bf](https://github.com/NEETROF/cymbra/commit/5f3b2bf8ab09961464b0da402aa81121af41eb3a))
* **catalog:** serve crawled scores immediately + hub load/count feedback ([#102](https://github.com/NEETROF/cymbra/issues/102)) ([32fd77f](https://github.com/NEETROF/cymbra/commit/32fd77f4cd979305f04dbdbb4ad64e7485ebfbfe))
* **deploy:** add music to the migrator role's search_path in provision script ([#100](https://github.com/NEETROF/cymbra/issues/100)) ([37010a8](https://github.com/NEETROF/cymbra/commit/37010a823092a1dbcaeb7d113061c43d9e68e1c3))
* **deploy:** expose the back office (bo.cymbra.app) in prod ([#123](https://github.com/NEETROF/cymbra/issues/123)) ([74996ac](https://github.com/NEETROF/cymbra/commit/74996ac6c63b468d02b2804872e3da5ed0a6ee2e))

## [0.7.0](https://github.com/NEETROF/cymbra/compare/backend-v0.6.1...backend-v0.7.0) (2026-07-17)


### Features

* Score Hub — search, save, facets, generated covers, favorites home ([#96](https://github.com/NEETROF/cymbra/issues/96)) ([97bd840](https://github.com/NEETROF/cymbra/commit/97bd84060d466d3627f883dc0ec2b4cf556e0725))

## [0.6.1](https://github.com/NEETROF/cymbra/compare/backend-v0.6.0...backend-v0.6.1) (2026-07-15)


### Bug Fixes

* **deploy:** wire the score-upload store for prod ([#90](https://github.com/NEETROF/cymbra/issues/90)) ([3d22f0e](https://github.com/NEETROF/cymbra/commit/3d22f0eabe8364efd5753ccd355918fa1e0cb7d1))

## [0.6.0](https://github.com/NEETROF/cymbra/compare/backend-v0.5.0...backend-v0.6.0) (2026-07-15)


### Features

* user score upload (contribution wizard + music module) ([#86](https://github.com/NEETROF/cymbra/issues/86)) ([d50f34e](https://github.com/NEETROF/cymbra/commit/d50f34ea7e5b962e6ecbac2fed43009f35b728e7))

## [0.5.0](https://github.com/NEETROF/cymbra/compare/backend-v0.4.0...backend-v0.5.0) (2026-07-13)


### Features

* **score-crawler:** licence-gated score crawler + catalog ingestion ([#80](https://github.com/NEETROF/cymbra/issues/80)) ([7885049](https://github.com/NEETROF/cymbra/commit/788504910d8c189813b0c899fc46f2576e01519c))

## [0.4.0](https://github.com/NEETROF/cymbra/compare/backend-v0.3.0...backend-v0.4.0) (2026-07-09)


### Features

* desktop Google sign-in via browser loopback OAuth (Windows/Linux) ([#59](https://github.com/NEETROF/cymbra/issues/59)) ([3203778](https://github.com/NEETROF/cymbra/commit/3203778beb9cfd071ef534a4b24d002668d1be2a))

## [0.3.0](https://github.com/NEETROF/cymbra/compare/backend-v0.2.1...backend-v0.3.0) (2026-07-08)


### Features

* **backend:** complete cross-schema account deletion ([#56](https://github.com/NEETROF/cymbra/issues/56)) ([6fd8aa1](https://github.com/NEETROF/cymbra/commit/6fd8aa1fe2ba8f63edad9d8122d4e2b66bf54118))

## [0.2.1](https://github.com/NEETROF/cymbra/compare/backend-v0.2.0...backend-v0.2.1) (2026-07-06)


### Bug Fixes

* **backend:** cold-restore safety for pg_dump backups (mq_uuid_exists) ([#49](https://github.com/NEETROF/cymbra/issues/49)) ([d07e2c6](https://github.com/NEETROF/cymbra/commit/d07e2c628032fd42ee0df2b3337e220ec6cf0924))

## [0.2.0](https://github.com/NEETROF/cymbra/compare/backend-v0.1.0...backend-v0.2.0) (2026-07-05)


### Features

* **account:** Cymbra ID account access (Google + Apple sign-in, handle onboarding) ([#30](https://github.com/NEETROF/cymbra/issues/30)) ([c71e8a1](https://github.com/NEETROF/cymbra/commit/c71e8a133d6697d6d58de6456ae18cc8dd0bd2fc))
* **backend:** add Cymbra ID backend foundation ([#27](https://github.com/NEETROF/cymbra/issues/27)) ([f122e63](https://github.com/NEETROF/cymbra/commit/f122e6385b05adf2c4c5dea2ef43abc6872adac4))
* **backend:** durable refresh-token sessions in Postgres ([#39](https://github.com/NEETROF/cymbra/issues/39)) ([594eed5](https://github.com/NEETROF/cymbra/commit/594eed595dc37dd22bd367685d06a8a93e0e133a))
* **db:** ops admin_svc role + environment-driven role bootstrap ([#33](https://github.com/NEETROF/cymbra/issues/33)) ([587bc09](https://github.com/NEETROF/cymbra/commit/587bc09e5110c56aa932e06a695dfcf94497358f))
* **jobs:** durable async-job infrastructure (cymbra-jobs + cymbra-worker) ([#32](https://github.com/NEETROF/cymbra/issues/32)) ([4e89875](https://github.com/NEETROF/cymbra/commit/4e89875c3a65e7100ba3a04c1b9bc9a1615819b5))
* **music:** account access — Sign in with Apple config + cross-platform gating + verified smoke test ([#36](https://github.com/NEETROF/cymbra/issues/36)) ([61ec756](https://github.com/NEETROF/cymbra/commit/61ec75659d3ec16520b3b4a636474d738286cad1))


### Bug Fixes

* **backend:** patch all 6 Dependabot advisories (rustls-webpki, jsonwebtoken, opentelemetry) ([#35](https://github.com/NEETROF/cymbra/issues/35)) ([2d913a8](https://github.com/NEETROF/cymbra/commit/2d913a83365c67fcb714e01e375d47b7646ccc20))
