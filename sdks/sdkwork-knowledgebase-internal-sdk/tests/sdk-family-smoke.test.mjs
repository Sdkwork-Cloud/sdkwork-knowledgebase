import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const testDir = path.dirname(fileURLToPath(import.meta.url));
const sdkRoot = path.resolve(testDir, "..");
const sdkName = "sdkwork-knowledgebase-internal-sdk";
const languages = ["typescript", "rust"];

function operationIds(openapi) {
  return Object.values(openapi.paths || {}).flatMap((pathItem) =>
    Object.entries(pathItem || {})
      .filter(([method]) => ["get", "post", "put", "patch", "delete"].includes(method))
      .map(([, operation]) => operation.operationId),
  );
}

test("internal SDK family declares the Knowledgebase application-ingress authority", () => {
  const manifest = JSON.parse(readFileSync(path.join(sdkRoot, "sdk-manifest.json"), "utf8"));
  assert.equal(manifest.sdkOwner, "sdkwork-knowledgebase");
  assert.equal(manifest.apiAuthority, "sdkwork-knowledgebase-internal-api");
  assert.equal(manifest.sdkSurface, "internal");
  assert.equal(manifest.sdkType, "custom");
  assert.equal(manifest.packageName, "@sdkwork/knowledgebase-internal-sdk");
  assert.equal(
    manifest.transportPackageName,
    "sdkwork-knowledgebase-internal-sdk-generated-typescript",
  );
  assert.equal(manifest.apiPrefix, "/internal/v3/api");
  assert.equal(manifest.standardProfile, "sdkwork-v3");
  assert.deepEqual(manifest.sdkDependencies, []);
  assert.equal(manifest.ownerOnlyOperationCount, 6);
  assert.deepEqual(manifest.typescript, {
    composedRoot: "sdkwork-knowledgebase-internal-sdk-typescript",
    composedEntry: "sdkwork-knowledgebase-internal-sdk-typescript/src/index.ts",
    transportRoot:
      "sdkwork-knowledgebase-internal-sdk-typescript/generated/server-openapi",
    transportEntry:
      "sdkwork-knowledgebase-internal-sdk-typescript/generated/server-openapi/src/index.ts",
  });
  assert(existsSync(path.join(sdkRoot, manifest.authoritySpec)));
  assert(existsSync(path.join(sdkRoot, manifest.generationInputSpec)));
});

test("TypeScript consumers use the private composed facade", () => {
  const composedRoot = path.join(
    sdkRoot,
    "sdkwork-knowledgebase-internal-sdk-typescript",
  );
  const packageJson = JSON.parse(
    readFileSync(path.join(composedRoot, "package.json"), "utf8"),
  );
  const entrySource = readFileSync(path.join(composedRoot, "src/index.ts"), "utf8");

  assert.equal(packageJson.name, "@sdkwork/knowledgebase-internal-sdk");
  assert.equal(packageJson.private, true);
  assert.equal(packageJson.exports["."].types, "./src/index.ts");
  assert.match(entrySource, /createGeneratedInternalClient/u);
  assert.match(entrySource, /SdkworkKnowledgebaseInternalClient/u);
  assert.doesNotMatch(entrySource, /fetch\(|axios|Authorization|Access-Token/u);
});

test("generated transports contain the owner-only Drive ingress and Wiki provider surface", () => {
  for (const language of languages) {
    const output = path.join(
      sdkRoot,
      `${sdkName}-${language}`,
      "generated/server-openapi",
    );
    const report = JSON.parse(
      readFileSync(path.join(output, ".sdkwork/sdkwork-generator-report.json"), "utf8"),
    );
    assert.equal(report.generator, "@sdkwork/sdk-generator");
    assert.equal(report.sdk.name, sdkName);
    assert.equal(report.sdk.language, language);
    assert.equal(report.stats.apis, 1);
    assert.equal(existsSync(path.join(output, "sdk-manifest.json")), false);
  }
});

test("authority and derived generator input are exact materializations", () => {
  const authority = readFileSync(
    path.join(sdkRoot, "../../apis/internal-api/knowledgebase/sdkwork-knowledgebase-internal-api.openapi.yaml"),
    "utf8",
  );
  const mirror = readFileSync(
    path.join(sdkRoot, "openapi/sdkwork-knowledgebase-internal-api.openapi.yaml"),
    "utf8",
  );
  const derived = readFileSync(
    path.join(sdkRoot, "openapi/sdkwork-knowledgebase-internal-api.sdkgen.yaml"),
    "utf8",
  );
  assert.equal(mirror, authority);
  assert.equal(derived, authority);
  assert.deepEqual(operationIds(JSON.parse(JSON.stringify(parseYamlLike(authority)))).sort(), [
    "driveEvents.receive",
    "wikiPublications.contents.retrieve",
    "wikiPublications.navigation.list",
    "wikiPublications.pages.search",
    "wikiPublications.retrieve",
    "wikiPublications.routes.resolve",
  ]);
});

function parseYamlLike(source) {
  const ids = [...source.matchAll(/^\s+operationId:\s+([^\s]+)\s*$/gmu)].map((match) => match[1]);
  return {
    paths: Object.fromEntries(ids.map((id, index) => [`/${index}`, { get: { operationId: id } }])),
  };
}
