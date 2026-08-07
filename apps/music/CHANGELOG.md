# Changelog

## [1.19.0](https://github.com/NEETROF/cymbra/compare/music-v1.18.0...music-v1.19.0) (2026-08-07)


### Features

* **curation-rewards:** points economy, rewards profile, reward shop ([#176](https://github.com/NEETROF/cymbra/issues/176)) ([08dfac4](https://github.com/NEETROF/cymbra/commit/08dfac4417a3aa5886b5539eca793dc52cc37809))
* **soundfont:** entitlement-gated downloads + server-rendered preview clips ([#179](https://github.com/NEETROF/cymbra/issues/179)) ([7e8b87d](https://github.com/NEETROF/cymbra/commit/7e8b87d80c4eee7201a3ae3943a4dbe66d6d9b11))
* **soundfont:** uploader attribution, rejection reason + motivated re-proposal ([#182](https://github.com/NEETROF/cymbra/issues/182)) ([44eea47](https://github.com/NEETROF/cymbra/commit/44eea473378e715a620c0557a6079b8c9aca1c4b))


### Bug Fixes

* **music:** stop the Android MIDI freeze and stuck virtual-piano keys ([#184](https://github.com/NEETROF/cymbra/issues/184)) ([0274485](https://github.com/NEETROF/cymbra/commit/027448501c617620ec2d37020c3426b4dac6c20a))

## [1.18.0](https://github.com/NEETROF/cymbra/compare/music-v1.17.0...music-v1.18.0) (2026-08-05)


### Features

* **analytics:** first-party feature-usage telemetry pipeline ([#174](https://github.com/NEETROF/cymbra/issues/174)) ([21319ae](https://github.com/NEETROF/cymbra/commit/21319aec67484c86a3b29787b152a44941c063e7))
* **leaderboards:** per-piece tempo/reaction leaderboards ([#6](https://github.com/NEETROF/cymbra/issues/6)) ([#173](https://github.com/NEETROF/cymbra/issues/173)) ([87c3cbe](https://github.com/NEETROF/cymbra/commit/87c3cbe16fadd39d222081e4dfeb0c92d7349bb9))
* **music:** opt-in propose of user scores to the public catalog ([#170](https://github.com/NEETROF/cymbra/issues/170)) ([43080df](https://github.com/NEETROF/cymbra/commit/43080df5d7d64c9c154a5b95a83495265ba5b9ff))
* **soundfont:** moderation, private libraries and instrument-sound hub ([#168](https://github.com/NEETROF/cymbra/issues/168)) ([153b5a3](https://github.com/NEETROF/cymbra/commit/153b5a39abd6864ee70395a72911e05dcd6216bc))


### Bug Fixes

* **music:** use Cymbra launcher icon on macOS and Windows ([#166](https://github.com/NEETROF/cymbra/issues/166)) ([7f291c9](https://github.com/NEETROF/cymbra/commit/7f291c9fd7df04e12ed1c898c831805ecc6aad60))
* **music:** wrap connected accounts screen body in SafeArea ([#165](https://github.com/NEETROF/cymbra/issues/165)) ([12b608c](https://github.com/NEETROF/cymbra/commit/12b608c2cfabc72d582ae45f94d42659d4cb29a1))

## [1.17.0](https://github.com/NEETROF/cymbra/compare/music-v1.16.0...music-v1.17.0) (2026-08-02)


### Features

* selectable instrument sounds — catalog, back-office management, and in-app picker ([#164](https://github.com/NEETROF/cymbra/issues/164)) ([548b252](https://github.com/NEETROF/cymbra/commit/548b252577a7b48587d6d81ab32571faa9763a40))


### Bug Fixes

* **music:** render the correct key signature per measure for modulating scores ([#160](https://github.com/NEETROF/cymbra/issues/160)) ([9b3bcbe](https://github.com/NEETROF/cymbra/commit/9b3bcbe8a260682a2829a1a36b0b49b5b5437559))
* stop UI freezes when playing a score (back-office worker + Flutter viewport cull) ([#163](https://github.com/NEETROF/cymbra/issues/163)) ([bbd7759](https://github.com/NEETROF/cymbra/commit/bbd77595d878ddc81e7e95e1b79b360e9768102e))

## [1.16.0](https://github.com/NEETROF/cymbra/compare/music-v1.15.0...music-v1.16.0) (2026-08-01)


### Features

* **account:** link sign-in identities across providers (Connected accounts) ([#147](https://github.com/NEETROF/cymbra/issues/147)) ([abe4913](https://github.com/NEETROF/cymbra/commit/abe4913df1c30528079474f764cdab286529083c))
* **auth:** verify email before binding a set-password credential ([#150](https://github.com/NEETROF/cymbra/issues/150)) ([452d93a](https://github.com/NEETROF/cymbra/commit/452d93a746a80f9eccc5b5c2bf0a34fcd0231933))
* **email:** brand and localize transactional emails with the Cymbra ID design system ([#149](https://github.com/NEETROF/cymbra/issues/149)) ([84211ba](https://github.com/NEETROF/cymbra/commit/84211ba1d46ef9b93d0aacd3f3ad7ed69f7e9394))
* **feature-flags:** add shared runtime feature-flag & config platform ([#152](https://github.com/NEETROF/cymbra/issues/152)) ([a8e487b](https://github.com/NEETROF/cymbra/commit/a8e487bc02ab378a016ce74d29f93be1edc86c29))
* **music:** per-user catalog access limits to prevent token scraping ([#156](https://github.com/NEETROF/cymbra/issues/156)) ([654bfc0](https://github.com/NEETROF/cymbra/commit/654bfc0abf359ef05493e4ef48f677e9ec7d99b2))
* persist and sync the account language preference ([#153](https://github.com/NEETROF/cymbra/issues/153)) ([30ff982](https://github.com/NEETROF/cymbra/commit/30ff9820c47707b02910c62115b7f74dccd32ab4))


### Bug Fixes

* **music:** coordinate token refresh to stop random silent sign-out ([#146](https://github.com/NEETROF/cymbra/issues/146)) ([b73a949](https://github.com/NEETROF/cymbra/commit/b73a94938c52a082dbc96dcb66f82bf925dca6d0))

## [1.15.0](https://github.com/NEETROF/cymbra/compare/music-v1.14.0...music-v1.15.0) (2026-07-30)


### Features

* **music:** play-activity heatmap, reliable stats sync, and public profiles ([#129](https://github.com/NEETROF/cymbra/issues/129)) ([d124d0d](https://github.com/NEETROF/cymbra/commit/d124d0d2604c73c0a2c867389f5ec3719a4d6256))
* **player:** stop the game at the last note, trimming trailing silence ([#142](https://github.com/NEETROF/cymbra/issues/142)) ([ab56c80](https://github.com/NEETROF/cymbra/commit/ab56c80834c9efe7fc5682439b2dfb0768ce6c23))
* **rating:** let the community rate pending scores in the deck ([#133](https://github.com/NEETROF/cymbra/issues/133)) ([885bf51](https://github.com/NEETROF/cymbra/commit/885bf5155ea159005c57753e677b90fac78f1530))
* swipe + star score rating deck, backend, and in-card preview ([#128](https://github.com/NEETROF/cymbra/issues/128)) ([e4ced64](https://github.com/NEETROF/cymbra/commit/e4ced64df93ee207fc4d2650b601352d8bc5401c))


### Bug Fixes

* **player:** rail the transport controls on the right on phone & tablet ([#134](https://github.com/NEETROF/cymbra/issues/134)) ([e9214db](https://github.com/NEETROF/cymbra/commit/e9214db87970c2ad471813ac75cab19d5e606e37))

## [1.14.0](https://github.com/NEETROF/cymbra/compare/music-v1.13.1...music-v1.14.0) (2026-07-28)


### Features

* **auth:** sign out from all devices (mobile) ([#119](https://github.com/NEETROF/cymbra/issues/119)) ([476116f](https://github.com/NEETROF/cymbra/commit/476116f89ffc528ec7a0ff87a551e01029212d76))
* **player:** start at the first note, trimming leading silence ([#104](https://github.com/NEETROF/cymbra/issues/104)) ([3fffc56](https://github.com/NEETROF/cymbra/commit/3fffc56a9edde439db28f13cf293619a4edb8eda))


### Bug Fixes

* **catalog:** serve crawled scores immediately + hub load/count feedback ([#102](https://github.com/NEETROF/cymbra/issues/102)) ([32fd77f](https://github.com/NEETROF/cymbra/commit/32fd77f4cd979305f04dbdbb4ad64e7485ebfbfe))

## [1.13.1](https://github.com/NEETROF/cymbra/compare/music-v1.13.0...music-v1.13.1) (2026-07-17)


### Bug Fixes

* **player:** game-mode scoring + Portée notation fidelity ([#98](https://github.com/NEETROF/cymbra/issues/98)) ([9ebd3b9](https://github.com/NEETROF/cymbra/commit/9ebd3b9e599ba0b5e703d4cedee0094678fb88c6))

## [1.13.0](https://github.com/NEETROF/cymbra/compare/music-v1.12.0...music-v1.13.0) (2026-07-17)


### Features

* Score Hub — search, save, facets, generated covers, favorites home ([#96](https://github.com/NEETROF/cymbra/issues/96)) ([97bd840](https://github.com/NEETROF/cymbra/commit/97bd84060d466d3627f883dc0ec2b4cf556e0725))

## [1.12.0](https://github.com/NEETROF/cymbra/compare/music-v1.11.2...music-v1.12.0) (2026-07-16)


### Features

* **music:** prepare Cymbra Music for App Store & Play Store distribution ([#93](https://github.com/NEETROF/cymbra/issues/93)) ([41715c4](https://github.com/NEETROF/cymbra/commit/41715c49f69fd002c7fc5f537b6b9c79a509f4c7))

## [1.11.2](https://github.com/NEETROF/cymbra/compare/music-v1.11.1...music-v1.11.2) (2026-07-15)


### Bug Fixes

* **ios:** add location purpose string to silence App Store warning 90683 ([#91](https://github.com/NEETROF/cymbra/issues/91)) ([8815d0d](https://github.com/NEETROF/cymbra/commit/8815d0d61e79d523a778bc91c7c8e0dbd1d6cd77))

## [1.11.1](https://github.com/NEETROF/cymbra/compare/music-v1.11.0...music-v1.11.1) (2026-07-15)


### Bug Fixes

* **ios:** add camera and photo library purpose strings to Info.plist ([#88](https://github.com/NEETROF/cymbra/issues/88)) ([2ef5c75](https://github.com/NEETROF/cymbra/commit/2ef5c759045d2b3cda87cec0f0e9220e2b74ec82))

## [1.11.0](https://github.com/NEETROF/cymbra/compare/music-v1.10.0...music-v1.11.0) (2026-07-15)


### Features

* **music:** in-app CGU and privacy links ([#84](https://github.com/NEETROF/cymbra/issues/84)) ([f6c3d67](https://github.com/NEETROF/cymbra/commit/f6c3d67de543f1b8a5f58f389e9b4ce7da251299))
* user score upload (contribution wizard + music module) ([#86](https://github.com/NEETROF/cymbra/issues/86)) ([d50f34e](https://github.com/NEETROF/cymbra/commit/d50f34ea7e5b962e6ecbac2fed43009f35b728e7))

## [1.10.0](https://github.com/NEETROF/cymbra/compare/music-v1.9.0...music-v1.10.0) (2026-07-13)


### Features

* **engine:** extract shared cymbra-musicxml-core crate ([#79](https://github.com/NEETROF/cymbra/issues/79)) ([8c3d69e](https://github.com/NEETROF/cymbra/commit/8c3d69ed2301b7bc0e09ef9a4095531bf2532b6d))
* **music:** gamify piano practice — scoring, feedback, summary & replay ([#76](https://github.com/NEETROF/cymbra/issues/76)) ([0c5f746](https://github.com/NEETROF/cymbra/commit/0c5f746af3d26aa43e00f2746c655aab0eb48ccc))

## [1.9.0](https://github.com/NEETROF/cymbra/compare/music-v1.8.2...music-v1.9.0) (2026-07-11)


### Features

* **music:** adapt player & library to smartphone landscape ([#74](https://github.com/NEETROF/cymbra/issues/74)) ([66061b7](https://github.com/NEETROF/cymbra/commit/66061b71d32d5a7e00e975a6edb4a891f8255b3c))
* **music:** add in-app localization (en/fr/it/es) ([#70](https://github.com/NEETROF/cymbra/issues/70)) ([33fdc13](https://github.com/NEETROF/cymbra/commit/33fdc132b6e6c37895ed33f91d6905064d6db5d1))
* **music:** tolerate sustained notes in Wait Mode ([#72](https://github.com/NEETROF/cymbra/issues/72)) ([e9a9aa6](https://github.com/NEETROF/cymbra/commit/e9a9aa6eb8db8518b4ec9ec9d044d9d4619854fe))

## [1.8.2](https://github.com/NEETROF/cymbra/compare/music-v1.8.1...music-v1.8.2) (2026-07-10)


### Bug Fixes

* **music:** unique iOS CFBundleName (fixes ITMS-90129) ([#68](https://github.com/NEETROF/cymbra/issues/68)) ([c260c46](https://github.com/NEETROF/cymbra/commit/c260c46a936401a3cfb1be5d19cb3f419be618e3))

## [1.8.1](https://github.com/NEETROF/cymbra/compare/music-v1.8.0...music-v1.8.1) (2026-07-10)


### Bug Fixes

* **music:** require full screen on iPad (landscape-only app) ([#66](https://github.com/NEETROF/cymbra/issues/66)) ([f47df8d](https://github.com/NEETROF/cymbra/commit/f47df8d49dbe79696fc05bea21a9ff9ec65d18bf))

## [1.8.0](https://github.com/NEETROF/cymbra/compare/music-v1.7.0...music-v1.8.0) (2026-07-10)


### Documentation

* **music:** summarize the signed release pipeline in the README ([#64](https://github.com/NEETROF/cymbra/issues/64)) ([1c72533](https://github.com/NEETROF/cymbra/commit/1c7253308a2020ae78960cfe0a55955e392bab91))

## [1.7.0](https://github.com/NEETROF/cymbra/compare/music-v1.6.0...music-v1.7.0) (2026-07-09)


### Features

* desktop Google sign-in via browser loopback OAuth (Windows/Linux) ([#59](https://github.com/NEETROF/cymbra/issues/59)) ([3203778](https://github.com/NEETROF/cymbra/commit/3203778beb9cfd071ef534a4b24d002668d1be2a))

## [1.6.0](https://github.com/NEETROF/cymbra/compare/music-v1.5.0...music-v1.6.0) (2026-07-06)


### Features

* **music:** TLS gRPC to prod via CYMBRA_GRPC_SECURE dart-define ([#50](https://github.com/NEETROF/cymbra/issues/50)) ([d7a0123](https://github.com/NEETROF/cymbra/commit/d7a012347cc116d4f5766df037030994d29e51fa))

## [1.5.0](https://github.com/NEETROF/cymbra/compare/music-v1.4.0...music-v1.5.0) (2026-07-05)


### Features

* **music:** account access — Sign in with Apple config + cross-platform gating + verified smoke test ([#36](https://github.com/NEETROF/cymbra/issues/36)) ([61ec756](https://github.com/NEETROF/cymbra/commit/61ec75659d3ec16520b3b4a636474d738286cad1))

## [1.4.0](https://github.com/NEETROF/cymbra/compare/music-v1.3.0...music-v1.4.0) (2026-07-02)


### Features

* **account:** Cymbra ID account access (Google + Apple sign-in, handle onboarding) ([#30](https://github.com/NEETROF/cymbra/issues/30)) ([c71e8a1](https://github.com/NEETROF/cymbra/commit/c71e8a133d6697d6d58de6456ae18cc8dd0bd2fc))

## [1.3.0](https://github.com/NEETROF/cymbra/compare/music-v1.2.0...music-v1.3.0) (2026-06-28)


### Features

* **player:** playback metronome (+ staff measure-bar fix) ([#28](https://github.com/NEETROF/cymbra/issues/28)) ([18bb8ea](https://github.com/NEETROF/cymbra/commit/18bb8ea2b635dbe39090a50b62757883d128a597))

## [1.2.0](https://github.com/NEETROF/cymbra/compare/music-v1.1.1...music-v1.2.0) (2026-06-26)


### Features

* **audio:** bundled SoundFont piano + MIDI hot-plug fix ([#26](https://github.com/NEETROF/cymbra/issues/26)) ([ec6e41d](https://github.com/NEETROF/cymbra/commit/ec6e41dc63f41faade3c86232e51c8da3dbce621))
* **music:** dynamic on-screen keyboard with range chooser and feedback ([#17](https://github.com/NEETROF/cymbra/issues/17)) ([6538377](https://github.com/NEETROF/cymbra/commit/65383775be4e56db0965d81f9452fd528031315c))
* **music:** hands-separate practice (Left/Right/Both) + settings drawer ([#24](https://github.com/NEETROF/cymbra/issues/24)) ([fb5afd6](https://github.com/NEETROF/cymbra/commit/fb5afd62eea4b75eabf3b2991240a6a70e6e4a5e))
* **music:** MusicXML parsing, engraving geometry & a score library ([#21](https://github.com/NEETROF/cymbra/issues/21)) ([4f95e59](https://github.com/NEETROF/cymbra/commit/4f95e597044286fd05a225cb20bf3a7b12c5a6cc))
* **music:** playability — playable keyboard, assist keys, onset Wait Mode, Partition cursor ([#22](https://github.com/NEETROF/cymbra/issues/22)) ([ab78785](https://github.com/NEETROF/cymbra/commit/ab787856baff9c7acea2238056646fefce476c5d))
* OpenSpec + 80% coverage gate + Riverpod 2/Freezed state management ([#15](https://github.com/NEETROF/cymbra/issues/15)) ([2a7ce56](https://github.com/NEETROF/cymbra/commit/2a7ce5655bcee30a4f5723a91111cbb34eacbcb9))

## [1.1.1](https://github.com/NEETROF/cymbra/compare/music-v1.1.0...music-v1.1.1) (2026-06-22)


### Bug Fixes

* **android:** unsafe(no_mangle) for edition 2024 + Android CI check ([#7](https://github.com/NEETROF/cymbra/issues/7)) ([b3a10a0](https://github.com/NEETROF/cymbra/commit/b3a10a0b64f50e2f0b235838d3de11f48d250cc0))

## [1.1.0](https://github.com/NEETROF/cymbra/compare/music-v1.0.0...music-v1.1.0) (2026-06-21)


### Features

* bootstrap Cymbra monorepo with the music app (Flutter + Rust POC) ([8fb70e2](https://github.com/NEETROF/cymbra/commit/8fb70e2d92f92d04c641ef99ba8f1bea7d6046dd))
