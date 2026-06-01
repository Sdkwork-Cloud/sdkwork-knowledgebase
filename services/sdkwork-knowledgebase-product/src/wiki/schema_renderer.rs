pub fn render_agents_md(space_name: &str) -> String {
    format!(
        r#"# AGENTS.md

You are maintaining the SDKWork LLM Wiki for `{space_name}`.

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
"#
    )
}

pub fn render_wiki_schema_yaml() -> String {
    r#"schemaVersion: "1.0"
profile: "docs/llm-wiki.md"
standardFiles:
  agentInstructions: "wiki/schema/AGENTS.md"
  schema: "wiki/schema/wiki_schema.yaml"
  index: "wiki/index.md"
  log: "wiki/log.md"
layers:
  raw:
    immutable: true
    root: "sources/raw/"
  wiki:
    persistent: true
    root: "wiki/"
  schema:
    root: "wiki/schema/"
workflows:
  ingest:
    requiredUpdates:
      - source_summary
      - related_pages
      - wiki/index.md
      - wiki/log.md
  query:
    readFirst:
      - wiki/index.md
    mayFileAnswer: true
  lint:
    checks:
      - broken_links
      - missing_citations
      - stale_claims
      - orphan_pages
      - missing_cross_references
"#
    .to_string()
}
