const fs = require('fs');

const zhFile = './src/i18n/locales/zh/kb.json';
const zhData = JSON.parse(fs.readFileSync(zhFile, 'utf8'));
Object.assign(zhData, {
  "importFromGitMenu": "从 Git 仓库导入...",
  "syncToGitMenu": "同步仓库到 Git...",
  "settings": "知识库设置",
  "deployAsWebsite": "部署为网站"
});
fs.writeFileSync(zhFile, JSON.stringify(zhData, null, 2));

const enFile = './src/i18n/locales/en/kb.json';
const enData = JSON.parse(fs.readFileSync(enFile, 'utf8'));
Object.assign(enData, {
  "importFromGitMenu": "Import from Git Repository...",
  "syncToGitMenu": "Sync Repository to Git...",
  "settings": "Knowledge Base Settings",
  "deployAsWebsite": "Deploy as Website"
});
fs.writeFileSync(enFile, JSON.stringify(enData, null, 2));

let file = './packages/sdkwork-knowledgebase-pc-knowledgebase/src/KbMenuContent.tsx';
let content = fs.readFileSync(file, 'utf8');

content = content.replace(/>从 Git 仓库导入\.\.\.</g, ">{t('importFromGitMenu')}<");
content = content.replace(/>同步仓库到 Git\.\.\.</g, ">{t('syncToGitMenu')}<");

fs.writeFileSync(file, content);
