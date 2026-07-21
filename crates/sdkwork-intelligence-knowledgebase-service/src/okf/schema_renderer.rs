pub fn render_agents_md(space_name: &str) -> String {
    format!(
        r#"---
type: Agent Instructions
title: SDKWork OKF Bundle Agent Instructions
description: Operational guidance for agents maintaining this OKF bundle.
---

# AGENTS.md

You are maintaining the SDKWork OKF Knowledge Bundle for `{space_name}`.

## Layers

- raw sources under `sources/raw/` are immutable source-of-truth materials.
- OKF concepts under `okf/` are persistent Markdown concepts with YAML frontmatter.
- `okf/schema/` defines agent workflows, lint checks, and publish rules.

## Storage

All service-managed files are persisted through sdkwork-drive. Do not write bundle files, schema files, index.md, log.md, exports, mirror packages, or governance revisions through direct filesystem or object-storage SDK paths.

## Workflows

- ingest: read one or more raw sources, upsert affected concepts, rebuild okf/index.md, and append okf/log.md.
- compile: validate a source or bundle revision, append okf/log.md, rebuild hierarchical index files, and refresh schema files.
- eval: lint the bundle, append okf/log.md, rebuild hierarchical index files, and refresh schema files.
- query: read okf/index.md first, retrieve relevant concepts and citations, and file valuable answers back into the bundle when approved.
- lint: check OKF conformance, broken links, orphan concepts, missing citations, stale claims, and knowledge gaps.
"#
    )
}

pub fn render_okf_profile_yaml() -> String {
    r#"okfVersion: "0.1"
schemaVersion: "1.0"
profile: "docs/okf-knowledge-bundle.md"
bundleRoot: "okf"
standardFiles:
  index: "index.md"
  log: "log.md"
  agentInstructions: "schema/AGENTS.md"
  profile: "schema/okf_profile.yaml"
layers:
  sources:
    immutable: true
    driveRoot: "sources/raw"
  bundle:
    persistent: true
    driveRoot: "okf"
workflows:
  ingest:
    steps:
      - read_sources
      - upsert_concepts
      - rebuild_index
      - append_log
  query:
    readFirst:
      - index.md
    mayFileAnswer: true
  lint:
    checks:
      - okf_conformance
      - broken_links
      - orphan_concepts
      - missing_citations
      - stale_claims
  compile:
    steps:
      - validate_source
      - append_log
      - rebuild_index
      - refresh_standard_files
  eval:
    steps:
      - lint_bundle
      - append_log
      - rebuild_index
      - refresh_standard_files
typeExamples:
  - "BigQuery Table"
  - "API Endpoint"
  - "Metric"
  - "Playbook"
  - "Reference"
  - "Knowledge Concept"
"#
    .to_string()
}
