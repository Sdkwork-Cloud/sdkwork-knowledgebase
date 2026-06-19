const fs = require('fs');

const file = 'packages/sdkwork-knowledgebase-pc-knowledgebase/src/WechatPublishPage.tsx';
let content = fs.readFileSync(file, 'utf8');

const tOverrides = [
    ["公众号编辑器", "titleOAEditor"],
    ["返回", "back"],
    ["未选择公众号", "noOASelected"],
    ["已选 ${selectedOfficialAccounts.length} 个公众号", "selectedOACount"],
    ["已选 1 个公众号", "selectedOneOA"],
    ["切换/管理公众号", "switchManageOA"],
    ["新建内容", "createNewContent"],
    ["写新文章", "writeNewArticle"],
    ["从笔记导入", "importFromNotes"],
    ["从网盘导入", "importFromCloudDrive"],
    ["历史版本", "historyVersions"],
    ["上移", "moveUp"],
    ["下移", "moveDown"],
    ["删除", "delete"],
    ["模拟微信扫码上传", "mockWechatScanUpload"],
    ["发布失败，请检查选中的公众号或文章内容。", "publishErrorMsg"],
    ["请先定位编辑器光标", "positionCursorFirst"],
    ["正在分析正文语义层级结构...", "analyzeSemanticStructure"],
    ["正在注入高级公众号专属配色...", "injectExclusiveColors"],
    ["正在设置段落呼吸留白及极限间距...", "setParagraphSpacing"],
    ["正在美化引用区块与代码高亮卡片...", "beautifyQuoteBlock"],
    ["大师级黄金排版注入成功！", "goldenTypographySuccess"]
];

if (!content.includes('useTranslation')) {
    content = content.replace("import React,", "import React, { ");
    content = content.replace("import { \n  X,", "import { useTranslation } from 'react-i18next';\nimport { \n  X,");
}

if (!content.includes("const { t } = useTranslation('editor');")) {
    content = content.replace(
        "const location = useLocation();",
        "const location = useLocation();\n  const { t } = useTranslation('editor');"
    );
}

for (const [zh, key] of tOverrides) {
    if (content.includes(`"${zh}"`)) {
        content = content.replace(new RegExp(`="${zh}"`, 'g'), `={t('${key}')}`);
        content = content.replace(new RegExp(`"${zh}"`, 'g'), `t('${key}')`);
    } else if (content.includes(`'${zh}'`)) {
        content = content.replace(new RegExp(`'${zh}'`, 'g'), `t('${key}')`);
    } else if (content.includes(`>${zh}<`)) {
        content = content.replace(new RegExp(`>${zh}<`, 'g'), `>{t('${key}')}<`);
    }
}

// Special case for JS template strings
content = content.replace(/`已选 \$\{selectedOfficialAccounts\.length\} 个公众号`/g, "`已选 ${selectedOfficialAccounts.length} ${t('oaCountSuffix')}`");
content = content.replace(/>写新文章</g, ">{t('writeNewArticle')}<");
content = content.replace(/>新建内容</g, ">{t('createNewContent')}<");
content = content.replace(/>从笔记导入</g, ">{t('importFromNotes')}<");
content = content.replace(/>从网盘导入</g, ">{t('importFromCloudDrive')}<");
content = content.replace(/title="上移"/g, "title={t('moveUp')}");
content = content.replace(/title="下移"/g, "title={t('moveDown')}");
content = content.replace(/title="删除"/g, "title={t('delete')}");
content = content.replace(/title="切换\/管理公众号"/g, "title={t('switchManageOA')}");
content = content.replace(/title="返回"/g, "title={t('back')}");

const editorJsonPath = 'src/i18n/locales/zh/editor.json';
let editorJson = JSON.parse(fs.readFileSync(editorJsonPath, 'utf8'));

editorJson.titleOAEditor = "公众号编辑器";
editorJson.back = "返回";
editorJson.noOASelected = "未选择公众号";
editorJson.oaCountSuffix = "个公众号";
editorJson.selectedOneOA = "已选 1 个公众号";
editorJson.switchManageOA = "切换/管理公众号";
editorJson.createNewContent = "新建内容";
editorJson.writeNewArticle = "写新文章";
editorJson.importFromNotes = "从笔记导入";
editorJson.importFromCloudDrive = "从网盘导入";
editorJson.historyVersions = "历史版本";
editorJson.moveUp = "上移";
editorJson.moveDown = "下移";
editorJson.delete = "删除";
editorJson.publishErrorMsg = "发布失败，请检查选中的公众号或文章内容。";
editorJson.positionCursorFirst = "请先定位编辑器光标";
editorJson.analyzeSemanticStructure = "🔍 正在分析正文语义层级结构...";
editorJson.injectExclusiveColors = "🎨 正在注入高级公众号专属配色...";
editorJson.setParagraphSpacing = "📏 正在设置段落呼吸留白及极限间距...";
editorJson.beautifyQuoteBlock = "✨ 正在美化引用区块与代码高亮卡片...";
editorJson.goldenTypographySuccess = "🎉 大师级黄金排版注入成功！";

fs.writeFileSync(editorJsonPath, JSON.stringify(editorJson, null, 2));

/// EN 
const enEditorJsonPath = 'src/i18n/locales/en/editor.json';
let enEditorJson = JSON.parse(fs.readFileSync(enEditorJsonPath, 'utf8'));
for(let k in editorJson) {
   if(!enEditorJson[k]) {
      enEditorJson[k] = k;
   }
}
fs.writeFileSync(enEditorJsonPath, JSON.stringify(enEditorJson, null, 2));

fs.writeFileSync(file, content, 'utf8');
console.log('done wechat publish page');
