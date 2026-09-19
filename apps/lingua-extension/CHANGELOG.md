# Changelog

## [1.0.1](https://github.com/NEETROF/cymbra/compare/lingua-extension-v1.0.0...lingua-extension-v1.0.1) (2026-09-19)


### Bug Fixes

* **lingua:** fit the manifest description in what Apple accepts, and gate it ([#495](https://github.com/NEETROF/cymbra/issues/495)) ([753e528](https://github.com/NEETROF/cymbra/commit/753e528aba877b6c24bf2a457f904fb6d68605fb))

## [1.0.0](https://github.com/NEETROF/cymbra/compare/lingua-extension-v1.0.0...lingua-extension-v1.0.0) (2026-09-19)


### Bug Fixes

* **lingua:** fit the manifest description in what Apple accepts, and gate it ([#495](https://github.com/NEETROF/cymbra/issues/495)) ([753e528](https://github.com/NEETROF/cymbra/commit/753e528aba877b6c24bf2a457f904fb6d68605fb))

## [1.0.0](https://github.com/NEETROF/cymbra/compare/lingua-extension-v0.1.0...lingua-extension-v1.0.0) (2026-09-19)


### Features

* **lingua:** account creation, email verification, password reset and Sign in with Apple in the extension ([#436](https://github.com/NEETROF/cymbra/issues/436)) ([6e0c6b3](https://github.com/NEETROF/cymbra/commit/6e0c6b3cf15b57bc89848a66abf95fb6dfc3c6ac))
* **lingua:** Apple and Google sign-in on Safari through the host app ([#460](https://github.com/NEETROF/cymbra/issues/460)) ([22c26ec](https://github.com/NEETROF/cymbra/commit/22c26ec652457e1d249bafe6677c1b46f527800f))
* **lingua:** erase Lingua data and keep card page addresses on the device ([#465](https://github.com/NEETROF/cymbra/issues/465)) ([d7a1a88](https://github.com/NEETROF/cymbra/commit/d7a1a88db637a7ce595eb7ce95d9e818e3770fbe))
* **lingua:** keep the reader's data in IndexedDB, owned by the background ([#479](https://github.com/NEETROF/cymbra/issues/479)) ([a9b78d6](https://github.com/NEETROF/cymbra/commit/a9b78d6a06856e441b6de01b4f794a0d0838deba))
* **lingua:** release the extension and the app from their own tags ([#490](https://github.com/NEETROF/cymbra/issues/490)) ([dc15238](https://github.com/NEETROF/cymbra/commit/dc15238fa457b523bf477ae0ccca57f033f5968b))
* **lingua:** ship the extension on Safari (macOS + iOS) ([#438](https://github.com/NEETROF/cymbra/issues/438)) ([5b1fc45](https://github.com/NEETROF/cymbra/commit/5b1fc45ce7d7cafaf0e364bfd2bd6bb35851c7dd))
* **lingua:** split the marked-words list by where each decision came from ([#432](https://github.com/NEETROF/cymbra/issues/432)) ([74d77f3](https://github.com/NEETROF/cymbra/commit/74d77f3cf48fd041c2903eb8edeb633f0a07008c))


### Bug Fixes

* **auth:** tolerate a client killed before it stored a rotated refresh token ([#477](https://github.com/NEETROF/cymbra/issues/477)) ([2303c42](https://github.com/NEETROF/cymbra/commit/2303c42b69d2b12fdcd458ba8a2a9801f534372d))
* **lingua:** bound the reading counters, and keep a doomed refresh from purging a new session ([#473](https://github.com/NEETROF/cymbra/issues/473)) ([13d2df5](https://github.com/NEETROF/cymbra/commit/13d2df5d9aa701d4a354cf9bde1e2e060fca9721))
* **lingua:** finish moving the state even when the settings area is full ([#480](https://github.com/NEETROF/cymbra/issues/480)) ([ea6168a](https://github.com/NEETROF/cymbra/commit/ea6168ac9b748f236e8530631f625b828df8eb35))
* **lingua:** initialise the WASM module once for all engines ([#468](https://github.com/NEETROF/cymbra/issues/468)) ([8c9bf02](https://github.com/NEETROF/cymbra/commit/8c9bf02388a0eeb5542c9fe7d953fbbde6c3c9bd))
* **lingua:** keep the session unless the server refuses it, and say when it is gone ([#471](https://github.com/NEETROF/cymbra/issues/471)) ([4ad17fc](https://github.com/NEETROF/cymbra/commit/4ad17fc8c5057caedf78ea1b173770331c549363))
* **lingua:** let a triggered sync finish before the background sleeps ([#470](https://github.com/NEETROF/cymbra/issues/470)) ([c32d7e6](https://github.com/NEETROF/cymbra/commit/c32d7e6d1c9a2edb091d33d3e2972efd176b2222))
* **lingua:** make a release stop at the release, and give the version one home ([#494](https://github.com/NEETROF/cymbra/issues/494)) ([74730cd](https://github.com/NEETROF/cymbra/commit/74730cdaa589ec215046597e04b998749b92dbe1))
* **lingua:** make the English CEFR counts right — cumulative ladder, AGID's bogus inflections, estimated vocabulary size ([#439](https://github.com/NEETROF/cymbra/issues/439)) ([bccac77](https://github.com/NEETROF/cymbra/commit/bccac771a6cbf91cce0b1903e7ee053040f5f44a))
* **lingua:** make the release lane build for the stores, not just for production ([#493](https://github.com/NEETROF/cymbra/issues/493)) ([275b152](https://github.com/NEETROF/cymbra/commit/275b15231d9ea90289b114607758511d5e529037))
* **lingua:** open the account page from the drawer, and guard the context family ([#482](https://github.com/NEETROF/cymbra/issues/482)) ([2d0c87f](https://github.com/NEETROF/cymbra/commit/2d0c87f5a2eb8c43d98e0a67dac111e0863010f7))
* **lingua:** render scrollbars dark on the extension's own surfaces ([#433](https://github.com/NEETROF/cymbra/issues/433)) ([ab67e9e](https://github.com/NEETROF/cymbra/commit/ab67e9e224808fd54904afa59c07e4c510d14ec4))
* **lingua:** schedule the sync from the store, not from a key it left ([#481](https://github.com/NEETROF/cymbra/issues/481)) ([6ac03b1](https://github.com/NEETROF/cymbra/commit/6ac03b1149f3f7036a22ebf06dc4ee4ff0d8e281))
* **lingua:** sync when a surface opens, and on demand from Réglages ([#469](https://github.com/NEETROF/cymbra/issues/469)) ([36f15b1](https://github.com/NEETROF/cymbra/commit/36f15b15343b86da706f4b2aac00a0d735fac354))
