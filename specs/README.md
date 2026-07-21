# Component Specs

This directory is the local SDKWork component contract for `sdkwork-knowledgebase`.

- Component root: `sdkwork-knowledgebase`
- Canonical standards: `../sdkwork-specs/README.md`
- Machine-readable contract: `specs/component.spec.json`

Read `specs/component.spec.json` before changing this component's public exports, runtime entrypoints, SDK clients, generated artifacts, config keys, or verification commands.

- OKF bundle contract: `specs/okf-knowledge-bundle.spec.json` owns OKF bundle layers, browser view mapping, original-source file list semantics, and root upload parent resolution rules.
- Accepted Live Wiki publication contract: `specs/live-wiki-publication.spec.json` owns the `sources/raw` projection, per-file public state, typed provider, cache freshness, and no-SiteRelease behavior. Implementation is approved; public/realtime/commercial claims remain blocked until its executable readiness gates close.
- Knowledge Engine SPI (switchable backends): `specs/knowledge-engine-spi.spec.json`
- External OSS engine catalog: `specs/external-knowledge-engine-catalog.spec.json`

Do not copy root standards into this directory. Link to files under `../sdkwork-specs/` instead.
