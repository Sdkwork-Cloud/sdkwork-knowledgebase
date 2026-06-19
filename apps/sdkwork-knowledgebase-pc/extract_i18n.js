import { Project, SyntaxKind, StringLiteral, JsxText, NoSubstitutionTemplateLiteral } from 'ts-morph';
import fs from 'fs';
import path from 'path';
import crypto from 'crypto';

const project = new Project({
  tsConfigFilePath: "tsconfig.json",
});

const sourceFiles = project.getSourceFiles("packages/sdkwork-knowledgebase-pc-knowledgebase/src/**/*.tsx");

const chineseRegex = /[\u4e00-\u9fa5]/;
const extracted = new Map();

for (const sourceFile of sourceFiles) {
  let hasChanges = false;
  const filePath = sourceFile.getFilePath();
  const relPath = path.relative(process.cwd(), filePath);
  
  // Find all textual literal nodes
  sourceFile.forEachDescendant(node => {
    let text = null;
    let isJsxText = false;
    let isStringLiteral = false;
    let isTemplate = false;

    if (node.getKind() === SyntaxKind.JsxText) {
      isJsxText = true;
      text = node.getText();
    } else if (node.getKind() === SyntaxKind.StringLiteral) {
      isStringLiteral = true;
      text = node.getLiteralText();
    } else if (node.getKind() === SyntaxKind.NoSubstitutionTemplateLiteral) {
      isTemplate = true;
      text = node.getLiteralText();
    }

    if (text && chineseRegex.test(text)) {
      const trimmedText = text.trim();
      if (!trimmedText) return;
      
      const hash = crypto.createHash('md5').update(trimmedText).digest('hex').substring(0, 8);
      const shortPath = path.basename(filePath, path.extname(filePath));
      const key = `${shortPath}_${hash}`;
      
      extracted.set(key, { text: trimmedText, node, isJsxText, isStringLiteral, isTemplate, filePath: relPath, key });
    }
  });
}

const result = Array.from(extracted.values()).map(e => ({ key: e.key, text: e.text, file: e.filePath }));
fs.writeFileSync('extracted_i18n.json', JSON.stringify(result, null, 2));
console.log(`Extracted ${result.length} unique Chinese texts.`);
