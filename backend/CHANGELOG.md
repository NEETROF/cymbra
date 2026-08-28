# Changelog

## [0.22.1](https://github.com/NEETROF/cymbra/compare/backend-v0.22.0...backend-v0.22.1) (2026-08-28)


### Bug Fixes

* **plans:** reactivating an entitlement clears its withdrawal stamp ([#293](https://github.com/NEETROF/cymbra/issues/293)) ([b5ac03a](https://github.com/NEETROF/cymbra/commit/b5ac03afe1d50af37c92f1c52a41273093e69c7d))

## [0.22.0](https://github.com/NEETROF/cymbra/compare/backend-v0.21.1...backend-v0.22.0) (2026-08-28)


### Features

* **plans:** the server builds the console's aggregator deep link ([#291](https://github.com/NEETROF/cymbra/issues/291)) ([5d2f3e7](https://github.com/NEETROF/cymbra/commit/5d2f3e7fa5198dbcb72098a7b46e3904287daf9d))

## [0.21.1](https://github.com/NEETROF/cymbra/compare/backend-v0.21.0...backend-v0.21.1) (2026-08-25)


### Bug Fixes

* **deploy:** ship every maintenance bin in the backend image ([#283](https://github.com/NEETROF/cymbra/issues/283)) ([bb51834](https://github.com/NEETROF/cymbra/commit/bb51834ba5d38b29b50b0183edab256399366cbb))

## [0.21.0](https://github.com/NEETROF/cymbra/compare/backend-v0.20.0...backend-v0.21.0) (2026-08-24)


### Features

* **music:** read, play and score drum parts, gated to the beta audience ([#279](https://github.com/NEETROF/cymbra/issues/279)) ([504701b](https://github.com/NEETROF/cymbra/commit/504701b5d5a251ddfa6a0c0865095747b97a1770))
* **plans:** a closed beta campaign can be reopened ([#280](https://github.com/NEETROF/cymbra/issues/280)) ([ac5a2a3](https://github.com/NEETROF/cymbra/commit/ac5a2a3ec2b8943da5f1a8b87aa767797ed37e7a))


### Bug Fixes

* **music:** stop the streak recovery offer from coming back ([#277](https://github.com/NEETROF/cymbra/issues/277)) ([c280d89](https://github.com/NEETROF/cymbra/commit/c280d89c3f150024f03fe39bbed29f7b46821585))

## [0.20.0](https://github.com/NEETROF/cymbra/compare/backend-v0.19.2...backend-v0.20.0) (2026-08-23)


### Features

* **music:** unroll repeats across the engine, app and back office ([#274](https://github.com/NEETROF/cymbra/issues/274)) ([b06c960](https://github.com/NEETROF/cymbra/commit/b06c960aad0798ab13d014df6e2b18b0179807b8))


### Bug Fixes

* **plans:** key store entitlements on the stable user:product ref ([#276](https://github.com/NEETROF/cymbra/issues/276)) ([14f1947](https://github.com/NEETROF/cymbra/commit/14f19470a57eb40bf0fa3131d9c42400b7b4ceef))

## [0.19.2](https://github.com/NEETROF/cymbra/compare/backend-v0.19.1...backend-v0.19.2) (2026-08-22)


### Bug Fixes

* **plans:** key Google entitlements on the base order id ([#266](https://github.com/NEETROF/cymbra/issues/266)) ([3499f36](https://github.com/NEETROF/cymbra/commit/3499f36dac1c609b936950f45255d6837a6391c0))

## [0.19.1](https://github.com/NEETROF/cymbra/compare/backend-v0.19.0...backend-v0.19.1) (2026-08-22)


### Bug Fixes

* **music:** ship the RevenueCat public SDK keys in the prod build config ([#264](https://github.com/NEETROF/cymbra/issues/264)) ([3829169](https://github.com/NEETROF/cymbra/commit/3829169b8006f0a69e68f7c772f8308d3470cd70))

## [0.19.0](https://github.com/NEETROF/cymbra/compare/backend-v0.18.1...backend-v0.19.0) (2026-08-22)


### Features

* **plans:** route store billing through RevenueCat ([#261](https://github.com/NEETROF/cymbra/issues/261)) ([a42ab2a](https://github.com/NEETROF/cymbra/commit/a42ab2a5eed6b9da9adb0c8b7a209aae866d7ef6))

## [0.18.1](https://github.com/NEETROF/cymbra/compare/backend-v0.18.0...backend-v0.18.1) (2026-08-18)


### Bug Fixes

* **corpus:** count user uploads as referenced when reconciling ([#255](https://github.com/NEETROF/cymbra/issues/255)) ([2ef19b3](https://github.com/NEETROF/cymbra/commit/2ef19b35be63bfd4c993b2bb9a2330cb4617d043))

## [0.18.0](https://github.com/NEETROF/cymbra/compare/backend-v0.17.1...backend-v0.18.0) (2026-08-17)


### Features

* **site:** show who is signed in on /account; fix(music): plan screen side safe area ([#249](https://github.com/NEETROF/cymbra/issues/249)) ([ea55d45](https://github.com/NEETROF/cymbra/commit/ea55d4548ec0974979cb25d091ab24edb7f7b4dd))


### Bug Fixes

* **music:** macOS Google sign-in in sandboxed builds needs network.server ([#252](https://github.com/NEETROF/cymbra/issues/252)) ([846bef2](https://github.com/NEETROF/cymbra/commit/846bef211ea5ad69ba48ce44c42dd29369740574))
* **plans:** the paid row, not the governing row, drives managed-on and the cross-channel purchase refusal ([#250](https://github.com/NEETROF/cymbra/issues/250)) ([7f67ffe](https://github.com/NEETROF/cymbra/commit/7f67ffe9163cdbcb99602038f93d18c8fffb6145))

## [0.17.1](https://github.com/NEETROF/cymbra/compare/backend-v0.17.0...backend-v0.17.1) (2026-08-16)


### Bug Fixes

* **deploy:** route every Axum prefix to the HTTP port in Caddy ([#245](https://github.com/NEETROF/cymbra/issues/245)) ([3e06212](https://github.com/NEETROF/cymbra/commit/3e0621287b78b008505acc2cd535021c2f2eb5b9))

## [0.17.0](https://github.com/NEETROF/cymbra/compare/backend-v0.16.1...backend-v0.17.0) (2026-08-16)


### Features

* **plans:** premium subscription, beta access codes, and cymbra.app account pages ([#242](https://github.com/NEETROF/cymbra/issues/242)) ([6406a66](https://github.com/NEETROF/cymbra/commit/6406a665616463968cc5097b83999d19e07c3fe4))

## [0.16.1](https://github.com/NEETROF/cymbra/compare/backend-v0.16.0...backend-v0.16.1) (2026-08-16)


### Bug Fixes

* **corpus:** keep the crawler's working files out of the served corpus ([#240](https://github.com/NEETROF/cymbra/issues/240)) ([feb786f](https://github.com/NEETROF/cymbra/commit/feb786fe8a2596e4c647e6969efdd2ee44adeae1))

## [0.16.0](https://github.com/NEETROF/cymbra/compare/backend-v0.15.0...backend-v0.16.0) (2026-08-16)


### Features

* **backend:** point the branded emails at the hosted Cymbra ID logo ([#233](https://github.com/NEETROF/cymbra/issues/233)) ([89534b6](https://github.com/NEETROF/cymbra/commit/89534b6b2b9fabb42cbbcc11b3bc7ae990d1f2a6))
* **badges:** cross-domain achievement registry ([#220](https://github.com/NEETROF/cymbra/issues/220)) ([bda89b3](https://github.com/NEETROF/cymbra/commit/bda89b310d9184aad350c086ea8716ba55e9ac51))
* **music:** award points for playing and practising ([#222](https://github.com/NEETROF/cymbra/issues/222)) ([344c8b7](https://github.com/NEETROF/cymbra/commit/344c8b71c29a2d4008fac78acd63309fbbf22810))
* **music:** finish the opt-in score catalog proposal — public credit, named refusals, moderator motive ([#228](https://github.com/NEETROF/cymbra/issues/228)) ([6b6ea3c](https://github.com/NEETROF/cymbra/commit/6b6ea3c91a5a2242740ed8170736959d37213fa5))
* **music:** freemium daily access on catalog opens + score audio teaser ([#229](https://github.com/NEETROF/cymbra/issues/229)) ([6c006e2](https://github.com/NEETROF/cymbra/commit/6c006e2e0fbad1f5b8974bb6633637a1deb3a92b))
* **music:** make catalog access limits runtime-tunable ([#236](https://github.com/NEETROF/cymbra/issues/236)) ([da35358](https://github.com/NEETROF/cymbra/commit/da353587e3b256b61458012d53d3d9d280d1dd04))
* **notifications:** per-category foreground presentation with in-app banner ([#217](https://github.com/NEETROF/cymbra/issues/217)) ([cbb9fd7](https://github.com/NEETROF/cymbra/commit/cbb9fd794f8ac5ea80850381adf3048adfcab8ae))
* **notifications:** server-driven push platform (FCM iOS/Android/macOS) ([#187](https://github.com/NEETROF/cymbra/issues/187)) ([8ffb743](https://github.com/NEETROF/cymbra/commit/8ffb743432a28bd8d9366f5ff2ffceeb50b2c6a4))
* **soundfonts:** admin-set reward pricing, gated shop and coming-soon lock ([#225](https://github.com/NEETROF/cymbra/issues/225)) ([969749b](https://github.com/NEETROF/cymbra/commit/969749b76d361723d7a3fc40e4ca1fe8c864c1d5))
* **streak:** server-tracked practice streak with a confirmed points freeze ([#224](https://github.com/NEETROF/cymbra/issues/224)) ([e872377](https://github.com/NEETROF/cymbra/commit/e872377cba34e1a282547c80438a92180e5e1435))

## [0.15.0](https://github.com/NEETROF/cymbra/compare/backend-v0.14.0...backend-v0.15.0) (2026-08-10)


### Features

* **courses:** interactive solfège curriculum — 42 lessons, exercise engine v2, learning path ([#207](https://github.com/NEETROF/cymbra/issues/207)) ([129d216](https://github.com/NEETROF/cymbra/commit/129d21647cbb2f8d7ee9653487ae4d9981cf71d9))
* **music:** measure-range practice as unscored selective runs ([#189](https://github.com/NEETROF/cymbra/issues/189)) ([d4b6930](https://github.com/NEETROF/cymbra/commit/d4b6930461285a5b154f9d8a5bb2df2bdf875db3))
* **music:** offline encrypted cache of favorited scores ([#194](https://github.com/NEETROF/cymbra/issues/194)) ([00aa30e](https://github.com/NEETROF/cymbra/commit/00aa30e4bdcd9793bc83cf318de5eb4e59ece4b5))


### Performance Improvements

* **back-office:** cache soundfont bytes and reuse the parsed font ([#203](https://github.com/NEETROF/cymbra/issues/203)) ([1b171fe](https://github.com/NEETROF/cymbra/commit/1b171fea0f8079dc8404cb9936a21035fb07a61c))

## [0.14.0](https://github.com/NEETROF/cymbra/compare/backend-v0.13.0...backend-v0.14.0) (2026-08-09)


### Features

* **leaderboard:** difficulty-weighted seasonal global leaderboard ([#191](https://github.com/NEETROF/cymbra/issues/191)) ([b19a483](https://github.com/NEETROF/cymbra/commit/b19a483ca0de17c965c917e1fa4b0a651a19b1c1))
* **music:** offer to rate a score after playing it ([#199](https://github.com/NEETROF/cymbra/issues/199)) ([d57ae6b](https://github.com/NEETROF/cymbra/commit/d57ae6b40abf67e1dbbe4d4b849500ffa77009d1))


### Performance Improvements

* **backend:** switch runtime image to distroless/cc + strip binaries ([#196](https://github.com/NEETROF/cymbra/issues/196)) ([5afb222](https://github.com/NEETROF/cymbra/commit/5afb222093801e274487dd0736ab892106d24882))

## [0.13.0](https://github.com/NEETROF/cymbra/compare/backend-v0.12.0...backend-v0.13.0) (2026-08-07)


### Features

* **curation-rewards:** points economy, rewards profile, reward shop ([#176](https://github.com/NEETROF/cymbra/issues/176)) ([08dfac4](https://github.com/NEETROF/cymbra/commit/08dfac4417a3aa5886b5539eca793dc52cc37809))
* **soundfont:** entitlement-gated downloads + server-rendered preview clips ([#179](https://github.com/NEETROF/cymbra/issues/179)) ([7e8b87d](https://github.com/NEETROF/cymbra/commit/7e8b87d80c4eee7201a3ae3943a4dbe66d6d9b11))
* **soundfont:** uploader attribution, rejection reason + motivated re-proposal ([#182](https://github.com/NEETROF/cymbra/issues/182)) ([44eea47](https://github.com/NEETROF/cymbra/commit/44eea473378e715a620c0557a6079b8c9aca1c4b))

## [0.12.0](https://github.com/NEETROF/cymbra/compare/backend-v0.11.0...backend-v0.12.0) (2026-08-05)


### Features

* **analytics:** first-party feature-usage telemetry pipeline ([#174](https://github.com/NEETROF/cymbra/issues/174)) ([21319ae](https://github.com/NEETROF/cymbra/commit/21319aec67484c86a3b29787b152a44941c063e7))
* **leaderboards:** per-piece tempo/reaction leaderboards ([#6](https://github.com/NEETROF/cymbra/issues/6)) ([#173](https://github.com/NEETROF/cymbra/issues/173)) ([87c3cbe](https://github.com/NEETROF/cymbra/commit/87c3cbe16fadd39d222081e4dfeb0c92d7349bb9))
* **music:** opt-in propose of user scores to the public catalog ([#170](https://github.com/NEETROF/cymbra/issues/170)) ([43080df](https://github.com/NEETROF/cymbra/commit/43080df5d7d64c9c154a5b95a83495265ba5b9ff))
* **soundfont:** moderation, private libraries and instrument-sound hub ([#168](https://github.com/NEETROF/cymbra/issues/168)) ([153b5a3](https://github.com/NEETROF/cymbra/commit/153b5a39abd6864ee70395a72911e05dcd6216bc))

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
