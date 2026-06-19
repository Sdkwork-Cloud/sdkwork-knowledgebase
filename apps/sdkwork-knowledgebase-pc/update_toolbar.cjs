const fs = require('fs');
let file = 'packages/sdkwork-knowledgebase-pc-knowledgebase/src/components/UniversalToolbar.tsx';
let txt = fs.readFileSync(file, 'utf8');
txt = txt.replace(/size=\{15\}/g, "size={14}");
txt = txt.replace(/p-1\.5/g, "p-1 md:p-1.5");
txt = txt.replace(/text-\[13px\]/g, "text-[11.5px] md:text-[13px]");
txt = txt.replace(/px-2 py-1/g, "px-1 md:px-2 py-0.5 md:py-1");
fs.writeFileSync(file, txt);
