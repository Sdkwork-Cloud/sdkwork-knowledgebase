#!/usr/bin/env node
import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const files = [
  "apis/backend-api/knowledgebase-backend-api.openapi.json",
  "apis/app-api/knowledgebase-app-api.openapi.json",
  "apis/open-api/knowledgebase-open-api.openapi.json",
  "sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json",
  "sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json",
  "sdks/sdkwork-knowledgebase-sdk/openapi/knowledgebase-open-api.openapi.json",
];

const pathReplacements = [
  ["/knowledge/wiki_compile_jobs", "/knowledge/okf/compile-jobs"],
  ["/knowledge/wiki_candidates/", "/knowledge/okf/candidates/"],
  ["/knowledge/wiki_candidates", "/knowledge/okf/candidates"],
  ["/knowledge/wiki_exports/", "/knowledge/okf/exports/"],
  ["/knowledge/wiki_exports", "/knowledge/okf/exports"],
  ["/knowledge/wiki_file_entries", "/knowledge/okf/bundle/files"],
  ["/knowledge/wiki_lint_runs", "/knowledge/okf/lint-runs"],
  ["/knowledge/wiki_eval_runs", "/knowledge/okf/eval-runs"],
  ["/knowledge/okf/profile_profiles/", "/knowledge/okf/profile/"],
  ["/knowledge/okf/profile_profiles", "/knowledge/okf/profile"],
  ["/okf/concepts/{pageId}", "/okf/concepts/{conceptId}"],
];

const schemaRenames = [
  ["WikiFileEntryType", "OkfBundleFileKind"],
  ["WikiLogEntry", "OkfLogEntry"],
  ["WikiExportRequest", "OkfBundleExportRequest"],
  ["WikiCompileJobRequest", "OkfCompileJobRequest"],
  ["WikiCandidateResultList", "OkfCandidateResultList"],
  ["WikiCandidateResult", "OkfCandidateResult"],
  ["WikiCandidateReviewRequest", "OkfCandidateReviewRequest"],
  ["WikiIndexRebuildRequest", "OkfBundleIndexRebuildRequest"],
  ["WikiQualityRunRequest", "OkfQualityRunRequest"],
  ["WikiQualityRun", "OkfQualityRun"],
  ["WikiPageSummaryList", "OkfConceptSummaryList"],
  ["WikiPageSummary", "OkfConceptSummary"],
  ["KnowledgeWikiPageRevisionList", "KnowledgeOkfConceptRevisionList"],
  ["KnowledgeWikiPageRevision", "KnowledgeOkfConceptRevision"],
  ["KnowledgeWikiFileEntryList", "KnowledgeOkfBundleFileList"],
  ["KnowledgeWikiFileEntry", "KnowledgeOkfBundleFile"],
  ["KnowledgeWikiSchemaProfileRequest", "KnowledgeOkfProfileRequest"],
];

function transform(content) {
  let out = content;
  for (const [from, to] of pathReplacements) {
    out = out.split(from).join(to);
  }
  for (const [from, to] of schemaRenames) {
    out = out.split(from).join(to);
  }
  out = out.replaceAll("wiki.lintRuns.create", "okf.lintRuns.create");
  out = out.replaceAll("wiki.evalRuns.create", "okf.evalRuns.create");
  out = out.replaceAll('"wiki_schema"', '"bundle_profile"');
  out = out.replaceAll('"wiki_index"', '"bundle_index"');
  out = out.replaceAll('"wiki_log"', '"bundle_log"');
  out = out.replaceAll('"wiki_revision"', '"concept_revision"');
  out = out.replaceAll('"wikiPageId"', '"conceptId"');
  out = out.replaceAll('"wikiRevisionId"', '"conceptRevisionId"');
  out = out.replaceAll('"wikiState"', '"okfState"');
  out = out.replaceAll("List wiki pages", "List OKF concepts");
  out = out.replaceAll("Retrieve a wiki page", "Retrieve an OKF concept");
  out = out.replaceAll("List wiki page revisions", "List OKF concept revisions");
  out = out.replaceAll("Retrieve the wiki index", "Retrieve the OKF bundle index");
  out = out.replaceAll("Retrieve the wiki log", "Retrieve the OKF bundle log");
  out = out.replaceAll("Retrieve the wiki schema", "Retrieve the OKF bundle profile");
  out = out.replaceAll("Create a wiki query", "Create an OKF query");
  out = out.replaceAll("File an answer for a wiki query", "File an answer for an OKF query");
  out = out.replaceAll("Create a wiki context pack", "Create an OKF context pack");
  out = out.replaceAll("Create a wiki compile job", "Create an OKF compile job");
  out = out.replaceAll("List wiki candidates", "List OKF candidates");
  out = out.replaceAll("Approve a wiki candidate", "Approve an OKF candidate");
  out = out.replaceAll("Reject a wiki candidate", "Reject an OKF candidate");
  out = out.replaceAll("Publish a wiki page", "Publish an OKF concept");
  out = out.replaceAll("Create a wiki schema profile", "Create an OKF profile");
  out = out.replaceAll("Update a wiki schema profile", "Update an OKF profile");
  out = out.replaceAll("Rebuild the wiki index", "Rebuild the OKF bundle index");
  out = out.replaceAll("Create a wiki log entry", "Create an OKF log entry");
  out = out.replaceAll("Create a wiki export", "Create an OKF bundle export");
  out = out.replaceAll("Retrieve a wiki export", "Retrieve an OKF bundle export");
  out = out.replaceAll("List wiki file entries", "List OKF bundle files");
  out = out.replaceAll("Create a wiki lint run", "Create an OKF lint run");
  out = out.replaceAll("Create a wiki eval run", "Create an OKF eval run");
  return out;
}

for (const rel of files) {
  const file = path.join(root, rel);
  const before = await readFile(file, "utf8");
  const after = transform(before);
  if (before !== after) {
    await writeFile(file, after, "utf8");
    console.log(`updated ${rel}`);
  } else {
    console.log(`unchanged ${rel}`);
  }
}
