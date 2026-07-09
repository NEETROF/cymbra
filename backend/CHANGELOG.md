# Changelog

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
