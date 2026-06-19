const fs = require('fs');
const file = 'packages/sdkwork-knowledgebase-pc-knowledgebase/src/KnowledgeBaseList.tsx';
let data = fs.readFileSync(file, 'utf8');

data = data.replace(
  /className="mt-1 mx-\[5px\] py-1 text-\[11px\] font-semibold text-indigo-600 hover:text-indigo-750 dark:text-indigo-400 dark:hover:text-indigo-300 hover:bg-indigo-50\/50 dark:hover:bg-indigo-950\/20 rounded-md transition-all flex items-center justify-center gap-1 shrink-0 select-none cursor-pointer"/g,
  'className="mt-1 mx-[5px] py-1 text-[11px] font-semibold text-[var(--color-kb-text-muted)] hover:text-[var(--color-kb-accent)] hover:bg-[var(--color-kb-panel-hover)] rounded-md transition-all flex items-center justify-center gap-1 shrink-0 select-none cursor-pointer"'
);

data = data.replace(
  /className="mt-1 mx-\[5px\] py-1 text-\[11px\] font-medium text-zinc-500 hover:text-zinc-900 dark:text-\[var(--color-kb-text-muted)\] dark:hover:text-zinc-200 hover:bg-\[var(--color-kb-panel-hover)\] rounded-md transition-all flex items-center justify-center gap-1 shrink-0 select-none cursor-pointer"/g,
  'className="mt-1 mx-[5px] py-1 text-[11px] font-medium text-[var(--color-kb-text-muted)] hover:text-[var(--color-kb-text-heading)] hover:bg-[var(--color-kb-panel-hover)] rounded-md transition-all flex items-center justify-center gap-1 shrink-0 select-none cursor-pointer"'
);

fs.writeFileSync(file, data, 'utf8');
