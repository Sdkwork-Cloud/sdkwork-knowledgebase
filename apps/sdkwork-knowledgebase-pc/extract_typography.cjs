const fs = require('fs');

const zhJson = {
    "oneClickTypography": "一键排版与AI重写",
    "goldenTypography": "大师级黄金排版",
    "goldenTypographyDesc": "系统默认高端配色与呼吸留白，适合知识分享",
    "minimalistTypography": "极简现代风",
    "minimalistTypographyDesc": "大面积留白，无明显边框，低饱和度，适合摄影/艺术",
    "techTypography": "极客科技风",
    "techTypographyDesc": "突出代码与高亮，适合技术文章、教程",
    "currentContent": "当前正文",
    "oneClickAIRewrite": "一键AI重写",
    "rewritingAI": "正在利用AI智能重写...",
    "selectTypographyStyle": "选择排版样式",
    "clickCardToPreview": "点击卡片切换预览",
    "confirmApplyTypography": "确认应用排版",
    "aiRewrittenFeedback": " (已通过AI优化语法与表达，使其更具吸引力与专业度)"
};

const editorJsonPath = 'src/i18n/locales/zh/editor.json';
let editorJson = JSON.parse(fs.readFileSync(editorJsonPath, 'utf8'));

for(let k in zhJson) {
   editorJson[k] = zhJson[k];
}

fs.writeFileSync(editorJsonPath, JSON.stringify(editorJson, null, 2));

const enEditorJsonPath = 'src/i18n/locales/en/editor.json';
let enEditorJson = JSON.parse(fs.readFileSync(enEditorJsonPath, 'utf8'));
for(let k in editorJson) {
   if(!enEditorJson[k]) {
      enEditorJson[k] = k;
   }
}
fs.writeFileSync(enEditorJsonPath, JSON.stringify(enEditorJson, null, 2));

console.log('done typography translation extracting');
