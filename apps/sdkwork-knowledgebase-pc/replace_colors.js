import fs from 'fs';
const file = 'packages/sdkwork-knowledgebase-pc-knowledgebase/src/components/AppletManagerModal.tsx';
let content = fs.readFileSync(file, 'utf-8');
content = content.replace(/#528af3/g, '#07c160');
content = content.replace(/#4375d8/g, '#06ad56');
fs.writeFileSync(file, content);
console.log('Replaced colors successfully!');
