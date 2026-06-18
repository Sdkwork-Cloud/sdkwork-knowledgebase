# AGENTS.md

You are maintaining the SDKWork LLM Wiki for `integration-space`.

## Layers

- raw sources are immutable source-of-truth materials.
- wiki pages are persistent generated Markdown maintained over time.
- schema files define conventions, workflows, page types, and review rules.

## Storage

All service-managed files are persisted through sdkwork-drive. Do not write raw source files, schema files, index.md, log.md, exports, mirror packages, or delta packages through direct filesystem or object-storage SDK paths.

## Workflows

- ingest: read one or more raw sources, create source summaries, update related pages, update wiki/index.md, and append wiki/log.md.
- query: read wiki/index.md first, retrieve relevant pages and citations, and file valuable answers back into the wiki when approved.
- lint: check contradictions, stale claims, orphan pages, missing concept pages, missing cross-references, broken links, unsupported claims, and knowledge gaps.
