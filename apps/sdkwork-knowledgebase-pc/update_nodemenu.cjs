const fs = require('fs');

const zhFile = './src/i18n/locales/zh/kb.json';
const zhData = JSON.parse(fs.readFileSync(zhFile, 'utf8'));
Object.assign(zhData, {
  "moveTo": "移动到...",
  "copyTo": "复制到...",
  "pinItem": "置顶",
  "editTags": "编辑标签",
  "memberPermissionsMenu": "成员权限",
  "openInNewTab": "在新标签页中打开",
  "openInSplitView": "右侧拆分打开",
  "viewHistory": "查看历史版本"
});
fs.writeFileSync(zhFile, JSON.stringify(zhData, null, 2));

const enFile = './src/i18n/locales/en/kb.json';
const enData = JSON.parse(fs.readFileSync(enFile, 'utf8'));
Object.assign(enData, {
  "moveTo": "Move to...",
  "copyTo": "Copy to...",
  "pinItem": "Pin",
  "editTags": "Edit Tags",
  "memberPermissionsMenu": "Permissions",
  "openInNewTab": "Open in New Tab",
  "openInSplitView": "Open in Split View",
  "viewHistory": "View History"
});
fs.writeFileSync(enFile, JSON.stringify(enData, null, 2));

let file = './packages/sdkwork-knowledgebase-pc-knowledgebase/src/NodeMenuContent.tsx';
let content = fs.readFileSync(file, 'utf8');

content = content.replace(/>移动到\.\.\.</g, ">{t('moveTo')}<");
content = content.replace(/>复制到\.\.\.</g, ">{t('copyTo')}<");
content = content.replace(/>置顶</g, ">{t('pinItem')}<");
content = content.replace(/>编辑标签</g, ">{t('editTags')}<");
content = content.replace(/>成员权限</g, ">{t('memberPermissionsMenu')}<");
content = content.replace(/>在新标签页中打开</g, ">{t('openInNewTab')}<");
content = content.replace(/>右侧拆分打开</g, ">{t('openInSplitView')}<");
content = content.replace(/>查看历史版本</g, ">{t('viewHistory')}<");

fs.writeFileSync(file, content);
