import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import path from 'path';
import { fileURLToPath } from 'node:url';
import {defineConfig, loadEnv} from 'vite';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const repoRoot = path.resolve(__dirname, '../..');
const appbaseRoot = path.resolve(repoRoot, '../sdkwork-appbase');
const DEFAULT_PLATFORM_API_GATEWAY_TARGET = 'http://127.0.0.1:3900';

export default defineConfig(({mode}) => {
  const env = loadEnv(mode, __dirname, '');
  const platformApiGatewayTarget =
    env.VITE_SDKWORK_KNOWLEDGEBASE_PLATFORM_API_GATEWAY_HTTP_URL
    || process.env.VITE_SDKWORK_KNOWLEDGEBASE_PLATFORM_API_GATEWAY_HTTP_URL
    || env.VITE_SDKWORK_APPBASE_APP_API_BASE_URL
    || process.env.VITE_SDKWORK_APPBASE_APP_API_BASE_URL
    || DEFAULT_PLATFORM_API_GATEWAY_TARGET;

  return {
    define: {
      'process.env.SDKWORK_ACCESS_TOKEN': JSON.stringify(env.SDKWORK_ACCESS_TOKEN ?? ''),
    },
            plugins: [react(), tailwindcss()],
    optimizeDeps: {
      include: [
        'react',
        'react-dom',
        'react-dom/client',
        'react/jsx-dev-runtime',
        'react/jsx-runtime',
        'react-router-dom',
        'lucide-react',
        'i18next',
        'react-i18next',
        'marked',
        'dompurify',
        'clsx',
        'tailwind-merge',
        'html2canvas-pro',
        '@monaco-editor/react',
        '@radix-ui/react-context-menu',
        '@radix-ui/react-dropdown-menu',
        '@tiptap/core',
        '@tiptap/react',
        '@tiptap/react/menus',
        '@tiptap/starter-kit',
        '@tiptap/extension-bubble-menu',
        '@tiptap/extension-image',
        '@tiptap/extension-placeholder',
        'tiptap-markdown',
        'react-pdf',
      ],
      exclude: ['pdfjs-dist'],
    },
    resolve: {
      alias: {
        '@': path.resolve(__dirname, '.'),
        '@packages': path.resolve(__dirname, 'packages'),
        '@sdkwork/knowledgebase-app-sdk': path.resolve(
          repoRoot,
          'sdks/sdkwork-knowledgebase-app-sdk/sdkwork-knowledgebase-app-sdk-typescript/generated/server-openapi/src/index.ts',
        ),
        '@sdkwork/appbase-app-sdk': path.resolve(
          appbaseRoot,
          'sdks/sdkwork-appbase-app-sdk/sdkwork-appbase-app-sdk-typescript/generated/server-openapi/src/index.ts',
        ),
        '@sdkwork/appbase-backend-sdk': path.resolve(
          appbaseRoot,
          'sdks/sdkwork-appbase-backend-sdk/sdkwork-appbase-backend-sdk-typescript/generated/server-openapi/src/index.ts',
        ),
        '@sdkwork/appbase-pc-react': path.resolve(
          appbaseRoot,
          'packages/pc-react/foundation/sdkwork-appbase-pc-react/src/index.ts',
        ),
        '@sdkwork/auth-runtime-pc-react': path.resolve(
          appbaseRoot,
          'packages/pc-react/iam/sdkwork-auth-runtime-pc-react/src/index.ts',
        ),
        '@sdkwork/iam-contracts': path.resolve(
          appbaseRoot,
          'packages/common/iam/sdkwork-iam-contracts/src/index.ts',
        ),
        '@sdkwork/sdk-common': path.resolve(
          repoRoot,
          '../sdkwork-sdk-commons/sdkwork-sdk-common-typescript/src/index.ts',
        ),
        '@sdkwork/ui-pc-react': path.resolve(
          repoRoot,
          '../sdkwork-ui/sdkwork-ui-pc-react/src/index.ts',
        ),
        '@sdkwork/core-pc-react': path.resolve(
          __dirname,
          'src/bootstrap/sdkworkCorePcReactShim.ts',
        ),
        'sdkwork-knowledgebase-pc-core': path.resolve(
          __dirname,
          'packages/sdkwork-knowledgebase-pc-core/src',
        ),
        'sdkwork-knowledgebase-pc-core/host/hostAdapter': path.resolve(
          __dirname,
          'packages/sdkwork-knowledgebase-pc-core/src/host/hostAdapter.ts',
        ),
      },
    },
    server: {
      port: 5184,
      strictPort: true,
      // HMR is disabled in AI Studio via DISABLE_HMR env var.
      // Do not modifyâfile watching is disabled to prevent flickering during agent edits.
      hmr: process.env.DISABLE_HMR !== 'true',
      // Disable file watching when DISABLE_HMR is true to save CPU during agent edits.
      watch: process.env.DISABLE_HMR === 'true' ? null : {},
      proxy: {
        '/app/v3/api': {
          target: platformApiGatewayTarget,
          changeOrigin: true,
        },
        '/backend/v3/api': {
          target: platformApiGatewayTarget,
          changeOrigin: true,
        },
        '/knowledge/v3/api': {
          target: platformApiGatewayTarget,
          changeOrigin: true,
        },
      },
    },
  };
});
