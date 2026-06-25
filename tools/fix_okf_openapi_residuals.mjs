import fs from "node:fs";

const files = [
  "sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json",
  "sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json",
];

const replacements = [
  ['"pageType"', '"conceptType"'],
  ['"slug"', '"conceptId"'],
  ['"summary"', '"description"'],
  ["WikiRevisionReviewState", "OkfRevisionReviewState"],
  ['"name": "pageId"', '"name": "conceptId"'],
  ['"pageId"', '"conceptRowId"'],
];

for (const file of files) {
  let content = fs.readFileSync(file, "utf8");
  for (const [from, to] of replacements) {
    content = content.split(from).join(to);
  }

  if (!content.includes('"OkfRevisionReviewState":')) {
    const schema = `      "OkfRevisionReviewState": {
        "type": "string",
        "enum": [
          "pending",
          "approved",
          "rejected"
        ]
      },
`;
    content = content.replace(
      '      "KnowledgeOkfConceptRevision":',
      `${schema}      "KnowledgeOkfConceptRevision":`,
    );
  }

  if (file.includes("app-api") && !content.includes("bundleRelativePath")) {
    content = content.replace(
      '"logicalPath",',
      '"logicalPath",\n          "bundleRelativePath",',
    );
    content = content.replace(
      `"logicalPath": {
            "type": "string"
          },`,
      `"logicalPath": {
            "type": "string"
          },
          "bundleRelativePath": {
            "type": "string"
          },`,
    );
  }

  fs.writeFileSync(file, content);
  console.log(`fixed ${file}`);
}
