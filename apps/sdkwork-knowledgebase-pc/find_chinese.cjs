const fs = require('fs');
const content = fs.readFileSync('packages/sdkwork-knowledgebase-pc-knowledgebase/src/WechatPublishPage.tsx', 'utf8');
const lines = content.split('\n');
const chinesePattern = /[\u4e00-\u9fa5]/;
lines.forEach((line, index) => {
  if (chinesePattern.test(line)) {
    console.log(`${index + 1}: ${line}`);
  }
});
