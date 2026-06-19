import { existsSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';

const iconPath = path.join('src-tauri', 'icons', 'icon.ico');

if (existsSync(iconPath)) {
  process.exit(0);
}

console.log('[sdkwork-knowledgebase-pc-desktop] generating missing desktop icons');
const result = spawnSync(
  process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm',
  ['desktop:icons'],
  { stdio: 'inherit', shell: true },
);

process.exit(result.status ?? 1);
