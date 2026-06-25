import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';

const packageRoot = path.dirname(fileURLToPath(import.meta.url));
const appRoot = path.resolve(packageRoot, '../..');

export default defineConfig({
  resolve: {
    alias: {
      '@sdkwork/sdkwork-knowledgebase-pc-commons': path.join(
        appRoot,
        'packages/sdkwork-knowledgebase-pc-commons/src',
      ),
      '@sdkwork/sdkwork-knowledgebase-pc-commons/stringUtils': path.join(
        appRoot,
        'packages/sdkwork-knowledgebase-pc-commons/src/stringUtils.ts',
      ),
      'sdkwork-knowledgebase-pc-core': path.join(appRoot, 'packages/sdkwork-knowledgebase-pc-core/src'),
    },
  },
  test: {
    environment: 'node',
    include: ['src/**/*.test.ts'],
  },
});
