#!/usr/bin/env node
import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const forbiddenPatterns = [
  /\bllm_wiki\b/,
  /\bLlmWiki\b/,
  /\bWikiPageType\b/,
  /\bWikiPage\b/,
  /\bkb_wiki_/,
  /\bwiki\.pages\b/,
  /\bwiki_schema\.yaml\b/,
  /\bKnowledgeWiki\b/,
  /\bprovider\.knowledge\.llm-wiki\b/,
  /\bupsert_page\b/,
  /\bget_page_by_id\b/,
  /\blist_okf_pages\b/,
  /\bwiki_page\b/,
  /\bwikiPageId\b/,
  /\bwikiRevisionId\b/,
  /\bwikiState\b/,
  /\| 'wiki' \|/,
];

const skipPathParts = [
  "target",
  "node_modules",
  "dist",
  "external",
  ".git",
  "2026-06-01-knowledgebase-backend-phase1-implementation.md",
  "okf-knowledge-bundle.spec.json",
  "2026-06-19-okf-knowledge-bundle-design.md",
  "migrate_openapi_wiki_to_okf.mjs",
];

const allowedExtensions = new Set([
  ".rs",
  ".ts",
  ".tsx",
  ".json",
  ".md",
  ".yaml",
  ".yml",
  ".sql",
  ".mjs",
  ".ps1",
]);

async function walk(dir, out = []) {
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    const rel = path.relative(root, full).replaceAll("\\", "/");
    if (skipPathParts.some((part) => rel.includes(part))) {
      continue;
    }
    if (entry.isDirectory()) {
      await walk(full, out);
      continue;
    }
    if (!allowedExtensions.has(path.extname(entry.name))) {
      continue;
    }
    out.push(full);
  }
  return out;
}

const requiredOkfServiceModules = [
  "concept_service.rs",
  "document.rs",
  "validator.rs",
  "index_renderer.rs",
  "log_renderer.rs",
  "bundle_linter.rs",
  "importer.rs",
  "exporter.rs",
  "initializer.rs",
  "storage.rs",
  "linter.rs",
  "bundle_workflow.rs",
  "catalog_log.rs",
  "governance_drive.rs",
  "standard_bundle_catalog_sync.rs",
  "standard_bundle_refresh.rs",
];

const requiredOkfStorageSymbols = [
  "read_managed_object_bytes",
  "get_object_bytes",
  "export_manifest.yaml",
  "validate_concept_bundle_relative_path",
  "extract_index_linked_concept_ids",
  "kb_okf_candidate",
  "SqliteKnowledgeOkfCandidateStore",
  "stage_concept_candidate",
  "stage_export_bundle_for_drive_import",
  "lint_stale_claims_against_source_lineage",
  "lint_concept_stale_claims",
  "extract_citation_urls",
  "list_space_source_lineage",
  "canonicalize_imported_concept_id",
  "validate_catalog_concept_id",
  "OkfBundleWorkflowEngine",
  "persist_standard_files_after_index_rebuild",
  "persist_dynamic_standard_bundle_files",
  "ensure_drive_permission_anchor",
];

const requiredOkfMigrationTables = ["kb_okf_concept", "kb_okf_concept_link", "kb_okf_candidate"];

const requiredObjectKeyAlignmentFiles = [
  {
    file: "crates/sdkwork-knowledgebase-drive/src/adapter.rs",
    symbols: ["KnowledgeObjectKeyPlanner", "space_uuid"],
  },
  {
    file: "crates/sdkwork-intelligence-knowledgebase-service/src/ports/knowledge_drive_storage.rs",
    symbols: ["space_uuid_from_drive_space_id", "with_drive_space_id"],
  },
  {
    file: "crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/okf_native.rs",
    symbols: ["read_managed_markdown"],
  },
];

const requiredOkfObservabilitySymbols = [
  "kb_okf_concept_publish_total",
  "kb_okf_concept_upsert_total",
  "kb_okf_bundle_lint_issues_total",
  "kb_okf_conformance_failures_total",
  "kb_okf_bundle_import_total",
  "record_okf_concept_publish",
  "record_okf_bundle_lint_completed",
  "record_okf_bundle_imported",
  "record_okf_bundle_exported",
];

const requiredOkfBrowserContract = {
  responseDataSchema: "KnowledgeBrowserListData",
  rootParentSource: "spaces.browser.list.data.parentId",
  views: {
    files: {
      okfDriveRoot: "sources/raw",
      nonOkfDriveRoot: null,
      purpose: "original_source_file_list",
      visibleInFrontendFileList: true,
      missingOkfRootPolicy: "empty_page",
      parentBoundary: "mustStayWithinViewRoot",
      forbiddenDriveRoots: ["okf", "output", ".sdkwork/governance"],
    },
    okf_bundle: {
      driveRoot: "okf",
      purpose: "generated_okf_bundle_tree",
      visibleInFrontendFileList: false,
      parentBoundary: "mustStayWithinViewRoot",
    },
    outputs: {
      driveRoot: "output",
      purpose: "generated_outputs",
      visibleInFrontendFileList: false,
      parentBoundary: "mustStayWithinViewRoot",
    },
  },
  frontendRules: {
    fileListView: "files",
    rootUploadParent: "response.data.parentId",
    forbidHardCodedRootUploadPath: true,
    forbidDriveRootUploadForOkf: true,
    okfConceptToolsView: "okf_bundle",
  },
};

function assertEqual(actual, expected, label) {
  if (actual !== expected) {
    violations.push(`${label}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
}

function assertArrayIncludesAll(actual, expected, label) {
  if (!Array.isArray(actual)) {
    violations.push(`${label}: expected array, got ${JSON.stringify(actual)}`);
    return;
  }
  for (const item of expected) {
    if (!actual.includes(item)) {
      violations.push(`${label}: missing ${JSON.stringify(item)}`);
    }
  }
}

async function readJson(relativePath) {
  return JSON.parse(await readFile(path.join(root, relativePath), "utf8"));
}

async function assertOkfBrowserContract() {
  const spec = await readJson("specs/okf-knowledge-bundle.spec.json");
  assertEqual(
    spec.layers?.sources?.driveRoot,
    requiredOkfBrowserContract.views.files.okfDriveRoot,
    "okf spec layers.sources.driveRoot",
  );
  assertEqual(
    spec.layers?.bundle?.driveRoot,
    requiredOkfBrowserContract.views.okf_bundle.driveRoot,
    "okf spec layers.bundle.driveRoot",
  );
  assertEqual(
    spec.layers?.governance?.driveRoot,
    ".sdkwork/governance",
    "okf spec layers.governance.driveRoot",
  );
  assertEqual(
    spec.browser?.responseDataSchema,
    requiredOkfBrowserContract.responseDataSchema,
    "okf browser responseDataSchema",
  );
  assertEqual(
    spec.browser?.rootParentSource,
    requiredOkfBrowserContract.rootParentSource,
    "okf browser rootParentSource",
  );

  const files = spec.browser?.views?.files;
  assertEqual(
    files?.okfDriveRoot,
    requiredOkfBrowserContract.views.files.okfDriveRoot,
    "okf browser views.files.okfDriveRoot",
  );
  assertEqual(
    files?.nonOkfDriveRoot ?? null,
    requiredOkfBrowserContract.views.files.nonOkfDriveRoot,
    "okf browser views.files.nonOkfDriveRoot",
  );
  assertEqual(
    files?.purpose,
    requiredOkfBrowserContract.views.files.purpose,
    "okf browser views.files.purpose",
  );
  assertEqual(
    files?.visibleInFrontendFileList,
    requiredOkfBrowserContract.views.files.visibleInFrontendFileList,
    "okf browser views.files.visibleInFrontendFileList",
  );
  assertEqual(
    files?.missingOkfRootPolicy,
    requiredOkfBrowserContract.views.files.missingOkfRootPolicy,
    "okf browser views.files.missingOkfRootPolicy",
  );
  assertEqual(
    files?.parentBoundary,
    requiredOkfBrowserContract.views.files.parentBoundary,
    "okf browser views.files.parentBoundary",
  );
  assertArrayIncludesAll(
    files?.forbiddenDriveRoots,
    requiredOkfBrowserContract.views.files.forbiddenDriveRoots,
    "okf browser views.files.forbiddenDriveRoots",
  );

  const okfBundle = spec.browser?.views?.okf_bundle;
  assertEqual(
    okfBundle?.driveRoot,
    requiredOkfBrowserContract.views.okf_bundle.driveRoot,
    "okf browser views.okf_bundle.driveRoot",
  );
  assertEqual(
    okfBundle?.purpose,
    requiredOkfBrowserContract.views.okf_bundle.purpose,
    "okf browser views.okf_bundle.purpose",
  );
  assertEqual(
    okfBundle?.visibleInFrontendFileList,
    requiredOkfBrowserContract.views.okf_bundle.visibleInFrontendFileList,
    "okf browser views.okf_bundle.visibleInFrontendFileList",
  );
  assertEqual(
    okfBundle?.parentBoundary,
    requiredOkfBrowserContract.views.okf_bundle.parentBoundary,
    "okf browser views.okf_bundle.parentBoundary",
  );

  const outputs = spec.browser?.views?.outputs;
  assertEqual(
    outputs?.driveRoot,
    requiredOkfBrowserContract.views.outputs.driveRoot,
    "okf browser views.outputs.driveRoot",
  );
  assertEqual(
    outputs?.purpose,
    requiredOkfBrowserContract.views.outputs.purpose,
    "okf browser views.outputs.purpose",
  );
  assertEqual(
    outputs?.visibleInFrontendFileList,
    requiredOkfBrowserContract.views.outputs.visibleInFrontendFileList,
    "okf browser views.outputs.visibleInFrontendFileList",
  );
  assertEqual(
    outputs?.parentBoundary,
    requiredOkfBrowserContract.views.outputs.parentBoundary,
    "okf browser views.outputs.parentBoundary",
  );

  const frontendRules = spec.browser?.frontendRules;
  for (const [key, expected] of Object.entries(requiredOkfBrowserContract.frontendRules)) {
    assertEqual(frontendRules?.[key], expected, `okf browser frontendRules.${key}`);
  }

  const browserService = await readFile(
    path.join(root, "crates/sdkwork-intelligence-knowledgebase-service/src/browser.rs"),
    "utf8",
  );
  for (const symbol of [
    "FILES_VIEW_OKF_ROOT_PATH",
    "sources/raw",
    "MissingViewRootPolicy::EmptyPage",
    "KnowledgeBrowserView::OkfBundle",
  ]) {
    if (!browserService.includes(symbol)) {
      violations.push(`missing OKF browser service symbol: ${symbol}`);
    }
  }

  const contractSource = await readFile(
    path.join(root, "crates/sdkwork-knowledgebase-contract/src/browser.rs"),
    "utf8",
  );
  if (!contractSource.includes("KnowledgeBrowserListData")) {
    violations.push("missing KnowledgeBrowserListData contract type");
  }

  const frontendFiles = await Promise.all(
    [
      "apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-knowledgebase/src/services/knowledgeBrowserListService.ts",
      "apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-knowledgebase/src/services/knowledgeBrowserParentResolver.ts",
      "apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-knowledgebase/src/services/knowledgeFileUploadService.ts",
      "apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-knowledgebase/src/services/knowledgeOkfConceptTransferService.ts",
    ].map((file) => readFile(path.join(root, file), "utf8")),
  );
  const frontendCombined = frontendFiles.join("\n");
  for (const symbol of [
    "DEFAULT_BROWSER_VIEW",
    "parentCacheKey(spaceId, view, parentId)",
    "listKnowledgeBrowserNodesPage(spaceId, null",
    "OKF_BUNDLE_BROWSER_VIEW",
  ]) {
    if (!frontendCombined.includes(symbol)) {
      violations.push(`missing OKF browser frontend symbol: ${symbol}`);
    }
  }

  const prd = await readFile(path.join(root, "docs/product/prd/PRD.md"), "utf8");
  for (const phrase of [
    "OKF original file list",
    "view=files",
    "sources/raw",
    "view=okf_bundle",
    "data.parentId",
  ]) {
    if (!prd.includes(phrase)) {
      violations.push(`PRD missing OKF browser rule phrase: ${phrase}`);
    }
  }

  const mvpLaunch = await readFile(
    path.join(root, "docs/product/prd/PRD-mvp-launch.md"),
    "utf8",
  );
  for (const phrase of [
    "KnowledgeBrowserListData",
    "view=files",
    "sources/raw",
    "view=okf_bundle",
    "data.parentId",
    "original-source file surface",
  ]) {
    if (!mvpLaunch.includes(phrase)) {
      violations.push(`MVP launch PRD missing OKF browser rule phrase: ${phrase}`);
    }
  }

  const specsReadme = await readFile(path.join(root, "specs/README.md"), "utf8");
  for (const phrase of [
    "browser view mapping",
    "original-source file list",
    "root upload parent resolution",
  ]) {
    if (!specsReadme.includes(phrase)) {
      violations.push(`specs README missing OKF browser rule phrase: ${phrase}`);
    }
  }
}

async function assertRequiredObjectKeyAlignment() {
  for (const entry of requiredObjectKeyAlignmentFiles) {
    const content = await readFile(path.join(root, entry.file), "utf8");
    for (const symbol of entry.symbols) {
      if (!content.includes(symbol)) {
        violations.push(`missing per-space object key symbol ${symbol} in ${entry.file}`);
      }
    }
  }
}

async function assertRequiredOkfObservabilitySymbols() {
  const files = [
    path.join(root, "crates/sdkwork-knowledgebase-observability/src/okf_metrics.rs"),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/okf/concept_service.rs",
    ),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/okf/bundle_linter.rs",
    ),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/okf/importer.rs",
    ),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/okf/exporter.rs",
    ),
  ];
  const combined = (
    await Promise.all(files.map((file) => readFile(file, "utf8")))
  ).join("\n");
  for (const symbol of requiredOkfObservabilitySymbols) {
    if (!combined.includes(symbol)) {
      violations.push(`missing okf observability symbol: ${symbol}`);
    }
  }
}

async function assertRequiredOkfModules() {
  const okfDir = path.join(
    root,
    "crates/sdkwork-intelligence-knowledgebase-service/src/okf",
  );
  const entries = await readdir(okfDir);
  for (const moduleName of requiredOkfServiceModules) {
    if (!entries.includes(moduleName)) {
      violations.push(`missing okf service module: okf/${moduleName}`);
    }
  }
}

async function assertAgentInstructionsConformance() {
  const contract = JSON.parse(
    await readFile(path.join(root, "specs/okf-knowledge-bundle.spec.json"), "utf8"),
  );
  const conceptType = contract.standardFiles?.agentInstructionsConceptType;
  if (conceptType !== "Agent Instructions") {
    violations.push("OKF agent instructions must declare their concept type");
  }
  const renderer = await readFile(
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/okf/schema_renderer.rs",
    ),
    "utf8",
  );
  if (!renderer.includes('r#"---') || !renderer.includes("type: Agent Instructions")) {
    violations.push("schema/AGENTS.md renderer must emit conformant OKF frontmatter");
  }
}

async function assertBackendReviewerIdentity() {
  const backend = await readFile(
    path.join(
      root,
      "crates/sdkwork-routes-knowledgebase-app-api/src/hosted_backend.rs",
    ),
    "utf8",
  );
  if (!backend.includes("let reviewer_id = self.runtime.operator_id().parse::<u64>().ok();")) {
    violations.push("OKF candidate reviewer identity must come from authenticated runtime context");
  }
  if (backend.includes("request.reviewer_id,")) {
    violations.push("OKF candidate reviewer identity must not trust the request body");
  }
}

async function assertRequiredOkfStorageSymbols() {
  const files = [
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/okf/storage.rs",
    ),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/ports/knowledge_drive_storage.rs",
    ),
    path.join(root, "crates/sdkwork-knowledgebase-drive/src/adapter.rs"),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/okf/exporter.rs",
    ),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/okf/validator.rs",
    ),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/okf/linter.rs",
    ),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-repository-sqlx/src/okf_candidate_store.rs",
    ),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/okf/concept_service.rs",
    ),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/okf/importer.rs",
    ),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/okf/bundle_linter.rs",
    ),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/okf/bundle_workflow.rs",
    ),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/okf/governance_drive.rs",
    ),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/okf/standard_bundle_catalog_sync.rs",
    ),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/okf/file_registry.rs",
    ),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/ports/knowledge_source_store.rs",
    ),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-repository-sqlx/src/sqlite_import_stores.rs",
    ),
  ];
  const combined = (
    await Promise.all(files.map((file) => readFile(file, "utf8")))
  ).join("\n");
  for (const symbol of requiredOkfStorageSymbols) {
    if (!combined.includes(symbol)) {
      violations.push(`missing okf storage/export symbol: ${symbol}`);
    }
  }
}

async function assertRequiredOkfMigrations() {
  const migrationsDir = path.join(
    root,
    "crates/sdkwork-intelligence-knowledgebase-repository-sqlx/migrations",
  );
  const migrationRoots = await readdir(migrationsDir, { withFileTypes: true });
  let migrationSql = "";
  for (const entry of migrationRoots) {
    if (!entry.isDirectory()) {
      continue;
    }
    const files = await readdir(path.join(migrationsDir, entry.name));
    for (const file of files) {
      if (file.endsWith(".sql")) {
        migrationSql += await readFile(
          path.join(migrationsDir, entry.name, file),
          "utf8",
        );
      }
    }
  }
  for (const tableName of requiredOkfMigrationTables) {
    if (!migrationSql.includes(tableName)) {
      violations.push(`missing okf migration table: ${tableName}`);
    }
  }
}

const violations = [];
await assertRequiredOkfModules();
await assertAgentInstructionsConformance();
await assertBackendReviewerIdentity();
await assertRequiredOkfStorageSymbols();
await assertRequiredObjectKeyAlignment();
await assertRequiredOkfMigrations();
await assertRequiredOkfObservabilitySymbols();
await assertOkfBrowserContract();

for (const file of await walk(root)) {
  const content = await readFile(file, "utf8");
  for (const pattern of forbiddenPatterns) {
    if (pattern.test(content)) {
      violations.push(`${path.relative(root, file)}: ${pattern}`);
    }
  }
}

if (violations.length > 0) {
  console.error("OKF legacy violations found:\n" + violations.join("\n"));
  process.exit(1);
}

console.log("OKF knowledge bundle standard check passed.");
