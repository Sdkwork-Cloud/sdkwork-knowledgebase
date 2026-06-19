const fs = require('fs');

const filepath = './packages/sdkwork-knowledgebase-pc-knowledgebase/src/components/OfficialAccountModal.tsx';
let txt = fs.readFileSync(filepath, 'utf8');

// The file currently has a messed up middle. Let's find exactly the blocks.
// Let's print out lines 420-500 and 850-900 to see what's actually there.
const lines = txt.split('\n');
console.log("--- 430 to 480 ---");
console.log(lines.slice(430, 480).join('\n'));
console.log("--- 855 to 892 ---");
console.log(lines.slice(855, 892).join('\n'));
