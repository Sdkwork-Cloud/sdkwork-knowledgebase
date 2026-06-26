import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';
import { appRoot, vitestSharedAliases } from '../../vitest.shared';

const packageRoot = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  resolve: {
    alias: {
      ...vitestSharedAliases,
      '@sdkwork/sdkwork-knowledgebase-pc-commons/stringUtils': path.join(
        appRoot,
        'packages/sdkwork-knowledgebase-pc-commons/src/stringUtils.ts',
      ),
    },
  },
  test: {
    environment: 'node',
    include: ['src/**/*.test.ts'],
  },
});
