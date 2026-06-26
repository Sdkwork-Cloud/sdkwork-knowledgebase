import path from 'node:path';
import { fileURLToPath } from 'node:url';

const appRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)));

export const vitestSharedAliases = {
  '@sdkwork/utils': path.resolve(
    appRoot,
    '../../../sdkwork-utils/packages/sdkwork-utils-typescript/src/index.ts',
  ),
};

export { appRoot };
