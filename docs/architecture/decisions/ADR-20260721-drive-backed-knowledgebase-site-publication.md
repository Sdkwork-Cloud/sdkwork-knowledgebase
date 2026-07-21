# ADR-20260721 Drive-Backed Knowledgebase Site Publication

Status: superseded
Owner: SDKWork Knowledgebase maintainers
Date: 2026-07-21
Superseded by: [ADR-20260721 Live Mounted Wiki Publication](ADR-20260721-live-mounted-wiki-publication.md)

## Resolution

The prelaunch immutable SiteRelease design was never released and is not an implementation,
compatibility, migration, API, SDK, database, or product authority. Its Site, SiteRelease,
SiteHostBinding, artifact-copy, public-router, and PC deployment implementations were removed as a
clean break.

All new Wiki publication work must follow the superseding ADR, the live Wiki machine contract, and
the current requirement and architecture documents. Do not restore old tables, routes, generated
SDK methods, aliases, dual reads, dual writes, release pointers, or UI entrypoints.

This file is retained only because architecture decision governance requires a stable supersession
record. It contains no historical design contract.
