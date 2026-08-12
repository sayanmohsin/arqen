# Changelog

## [0.8.1](https://github.com/sayanmohsin/arqen/compare/arqen-v0.8.0...arqen-v0.8.1) (2026-08-11)


### Bug Fixes

* harden deployment with Thingd 0.79.1 ([fac7272](https://github.com/sayanmohsin/arqen/commit/fac7272f3f9d8f9e7e5f976acf381fb55633dcf2))

## [0.8.0](https://github.com/sayanmohsin/arqen/compare/arqen-v0.7.0...arqen-v0.8.0) (2026-08-10)


### Features

* add native Thingd 0.78 replication ([#35](https://github.com/sayanmohsin/arqen/issues/35)) ([deee00a](https://github.com/sayanmohsin/arqen/commit/deee00ab012e2aca09442a095b3cc44e595751f2))

## [0.7.0](https://github.com/sayanmohsin/arqen/compare/arqen-v0.6.0...arqen-v0.7.0) (2026-08-09)


### Features

* integrate Thingd 0.77 storage and sync ([316b46f](https://github.com/sayanmohsin/arqen/commit/316b46f70bc8935e6dd1c88414a73a68d73f9fce))


### Bug Fixes

* align thingd HTTP adapter with public REST contract ([4a9dec5](https://github.com/sayanmohsin/arqen/commit/4a9dec5be1d290b762c532981e537b5b446471a2))

## [0.6.0](https://github.com/sayanmohsin/arqen/compare/arqen-v0.5.0...arqen-v0.6.0) (2026-08-08)


### Features

* finalize Arqen beta hardening ([64119f1](https://github.com/sayanmohsin/arqen/commit/64119f14367dbe35a9b30213431ea15f347359c1))

## [0.5.0](https://github.com/sayanmohsin/arqen/compare/arqen-v0.4.0...arqen-v0.5.0) (2026-08-06)


### Features

* add full dev lifecycle toolchain (lint, format, test, build, doc) ([062a58f](https://github.com/sayanmohsin/arqen/commit/062a58fd1e085c494aa8cc030f6876902ad7fcc3))
* **phase-17:** developer experience, performance, and agent onboarding ([e55b4d5](https://github.com/sayanmohsin/arqen/commit/e55b4d50ebc996e18d10c41f28b9506a8d311ca7))

## [0.4.0](https://github.com/sayanmohsin/arqen/compare/arqen-v0.3.0...arqen-v0.4.0) (2026-08-05)


### Features

* **agent:** add tool execution boundary with invoke endpoint ([18d1c32](https://github.com/sayanmohsin/arqen/commit/18d1c3279155aee64001f9b9da1c44d4115872ff)), closes [#18](https://github.com/sayanmohsin/arqen/issues/18)
* **arqen:** complete production hardening gaps ([7630f25](https://github.com/sayanmohsin/arqen/commit/7630f2539be90c8ea17cc6bd464c28cf021daaeb))
* **auth:** real JWT, constant-time API keys, SHA-256 hashing ([e2d01a7](https://github.com/sayanmohsin/arqen/commit/e2d01a7b265b30758fcdeab8a7ef2699af6b048b))
* **cli:** add `arqen up` to run and supervise dev services ([3d5999b](https://github.com/sayanmohsin/arqen/commit/3d5999b21d14da4400eea8b136e79d61ea431f8e)), closes [#21](https://github.com/sayanmohsin/arqen/issues/21)
* **config:** add env var parsing for all config fields ([a6b9c0d](https://github.com/sayanmohsin/arqen/commit/a6b9c0ddc5141d9253a68e716419a9454c0a20f3))
* **config:** add layered loading and production settings ([83cf810](https://github.com/sayanmohsin/arqen/commit/83cf810a9220278a70507ce3342929278574020f))
* **config:** change default server port from 3000 to 8888 ([a78d2ad](https://github.com/sayanmohsin/arqen/commit/a78d2adb1ea82a1f2735f38b37037cefc25882ef))
* **error:** add timeout/dependency errors and more From impls ([5282102](https://github.com/sayanmohsin/arqen/commit/5282102cb7dec67ddd03a80871ad79a1019a45aa))
* **health:** parallel checks, liveness/readiness probes, HTTP status codes ([85fdfde](https://github.com/sayanmohsin/arqen/commit/85fdfde7b2e605bc8ea6766e05e3317dac66a08f))
* **http:** add auth middleware and Authenticated extractor ([036bc96](https://github.com/sayanmohsin/arqen/commit/036bc96e395c9bb933338f874052d12dd1bae3ba)), closes [#12](https://github.com/sayanmohsin/arqen/issues/12)
* **http:** add builtin_routes for composing built-in routes with custom state ([22cd9da](https://github.com/sayanmohsin/arqen/commit/22cd9da9adf698f461ee78ab45695d687e53d1db)), closes [#15](https://github.com/sayanmohsin/arqen/issues/15)
* **http:** add CORS middleware with permissive defaults ([cdd3fd2](https://github.com/sayanmohsin/arqen/commit/cdd3fd2ecabf244a2744a6ea232ca409b16063a2))
* **http:** add RequireAuth extractor and require_auth_middleware guard ([ff894e3](https://github.com/sayanmohsin/arqen/commit/ff894e3e195465ed890879c992a57cfb4a4daa70)), closes [#20](https://github.com/sayanmohsin/arqen/issues/20)
* **http:** add router composition API for application routes ([3d664f5](https://github.com/sayanmohsin/arqen/commit/3d664f5e73b7ce26c70da6835cd7e53105c50486)), closes [#11](https://github.com/sayanmohsin/arqen/issues/11)
* **http:** wire config values into router (request_timeout, max_body_size) ([c903ada](https://github.com/sayanmohsin/arqen/commit/c903ada71f0b09f9b6b238c13046d4e1412dae70))
* **http:** wire HealthRegistry into /health and /ready endpoints ([3993294](https://github.com/sayanmohsin/arqen/commit/3993294ddaea1d13e9c449df647711dc57ccd6a2)), closes [#13](https://github.com/sayanmohsin/arqen/issues/13)
* **jobs:** metrics, concurrency config, structured logging ([6ad3a95](https://github.com/sayanmohsin/arqen/commit/6ad3a95153c4c3bd4426ced60e1f9ff7afc12d35))
* merge CLI into single arqen package ([05d3b54](https://github.com/sayanmohsin/arqen/commit/05d3b54e37b29deb8ae9338d68a654dd8e0d956a))
* **module:** add explicit module composition and scaffolding ([e7c43f0](https://github.com/sayanmohsin/arqen/commit/e7c43f086dd27c86a6b36a0c25e40d1578db9545))
* **module:** lifecycle hooks, dependencies, health checks ([18b8660](https://github.com/sayanmohsin/arqen/commit/18b86603572ff2381dd38c273aa9335c9f5cffe5))
* **observability:** percentiles, uptime, error rate, by-status ([62fc993](https://github.com/sayanmohsin/arqen/commit/62fc9937758b6a9ff5a704118cf8136dde710ed8))
* **openapi:** add put/delete/patch builders to OpenApiGenerator ([c0b8328](https://github.com/sayanmohsin/arqen/commit/c0b83281682f003e88012f1107ba0db4d5f09ddb)), closes [#14](https://github.com/sayanmohsin/arqen/issues/14)
* **openapi:** proper 3.0 spec, security schemes, Swagger UI ([40ff00d](https://github.com/sayanmohsin/arqen/commit/40ff00d7d2ca92f38be31299720901c9c5ff954f))
* **state:** unify RuntimeInfo into AppState ([6d3d721](https://github.com/sayanmohsin/arqen/commit/6d3d721beea31a43e13dfc7e2702d1b382b40d0a))
* **testutil:** request builders, response readers, fixture helpers ([9b62d38](https://github.com/sayanmohsin/arqen/commit/9b62d38367dab0a43549b112d15d66752039f852))
* **thingd:** add QueryOptions with conjunctive filters and pagination ([89f944e](https://github.com/sayanmohsin/arqen/commit/89f944ef2011df6a9da48fe1e0a34ddfa2d07dc0)), closes [#17](https://github.com/sayanmohsin/arqen/issues/17)


### Bug Fixes

* **auth,validation:** feature-gate axum-dependent modules behind http-server ([4d66382](https://github.com/sayanmohsin/arqen/commit/4d66382772b6fd7e684869edad99b3021d35cc0b))
* **http:** propagate request correlation ID into error responses ([5bdb859](https://github.com/sayanmohsin/arqen/commit/5bdb859a793d31625a7e28b81b14d6c307e1b792)), closes [#16](https://github.com/sayanmohsin/arqen/issues/16)
* synchronize module composition docs and errors ([2783c97](https://github.com/sayanmohsin/arqen/commit/2783c97a9ae776c887cb97a11b0f1dac450997ae))
* **thingd:** add /v1 prefix to HttpThingdBackend for sidecar compatibility ([f9c3fb2](https://github.com/sayanmohsin/arqen/commit/f9c3fb243d8bc347e35629ea92842d42deb2b783))

## [0.3.0](https://github.com/sayanmohsin/arqen/compare/arqen-v0.2.0...arqen-v0.3.0) (2026-08-03)


### Features

* **auth:** add pluggable authentication with adapters ([d720a7a](https://github.com/sayanmohsin/arqen/commit/d720a7ae6e1fcc89c31ed2cf89095e62526d15f2))
* **config:** add typed configuration and AppState builder ([8410ec6](https://github.com/sayanmohsin/arqen/commit/8410ec6d65a115adf707779706bb1b0b216693d7))
* **error:** add stable error contracts with correlation IDs ([fce7c54](https://github.com/sayanmohsin/arqen/commit/fce7c54c35d2d179282ec42f6bc69387d21c2bbc))
* **health:** add health and readiness checks ([fb5365d](https://github.com/sayanmohsin/arqen/commit/fb5365d3dfd8318ffb11deb28e6661d544ed5467))
* **module:** add module composition system ([7d4629a](https://github.com/sayanmohsin/arqen/commit/7d4629a46870bfe198a3b05e6dda06a474231496))
* **observability:** add request metrics and monitoring ([2bf5b5b](https://github.com/sayanmohsin/arqen/commit/2bf5b5b19146971b547308937fb62ef5f579cb52))
* **openapi:** add OpenAPI spec generation ([2d5699a](https://github.com/sayanmohsin/arqen/commit/2d5699a94ad141faa19b64476861324d7cce7ac5))
* **testutil:** add testing utilities ([0a694b6](https://github.com/sayanmohsin/arqen/commit/0a694b656feb3a2957f5f235ffdd861c0acb2d9f))
* **validation:** add request validation extractors ([15bb8bc](https://github.com/sayanmohsin/arqen/commit/15bb8bce8b2dd7dc85d784a0e0e2d3586fcaf14c))


### Bug Fixes

* address all audit gaps (critical, high, medium) ([1ab44df](https://github.com/sayanmohsin/arqen/commit/1ab44df82cbde77a730f3ba67ddca4d0479f2621))
* remaining production gaps (CLI, examples, docs) ([47ef934](https://github.com/sayanmohsin/arqen/commit/47ef93447885b79d763ceba0e581771f2a10b18a))

## [0.2.0](https://github.com/sayanmohsin/arqen/compare/arqen-v0.1.1...arqen-v0.2.0) (2026-08-02)


### Features

* add public arqen facade crate ([5ea19ac](https://github.com/sayanmohsin/arqen/commit/5ea19ac5bba666d26b28f44089db0f4772125d44))


### Bug Fixes

* use literal version strings in crate Cargo.toml ([7340693](https://github.com/sayanmohsin/arqen/commit/7340693e9108a2247336234f59e13456d223bbce))
* use literal version strings in crate Cargo.toml for Release Please ([1467a58](https://github.com/sayanmohsin/arqen/commit/1467a58b3398b8a07f45591be5c6340043044f30))
