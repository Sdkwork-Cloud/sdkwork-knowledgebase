import { isBlank, trim } from '@sdkwork/utils';

import type { SdkworkEnvironment } from './runtimeConfig';
export interface KnowledgebaseFeatureFlags {
  documentVersionHistory: boolean;
  documentPermissionsModal: boolean;
  knowledgeMarketCatalog: boolean;
  notesImport: boolean;
  chatFileImport: boolean;
  chatDialogImport: boolean;
  aiImageGeneration: boolean;
  websiteDeploy: boolean;
  wechatFullPublish: boolean;
}

function readBooleanEnv(value: string | undefined, defaultValue: boolean): boolean {
  if (value === undefined || isBlank(value)) {
    return defaultValue;
  }
  const normalized = trim(value).toLowerCase();  return normalized === '1' || normalized === 'true' || normalized === 'yes';
}

export function resolveKnowledgebaseFeatureFlags(
  environment: SdkworkEnvironment,
  env: Record<string, string | undefined> = import.meta.env as Record<string, string | undefined>,
): KnowledgebaseFeatureFlags {
  const isProduction = environment === 'production';
  const devPreview = readBooleanEnv(env.VITE_SDKWORK_KNOWLEDGEBASE_ENABLE_PREVIEW_FEATURES, !isProduction);

  return {
    documentVersionHistory: readBooleanEnv(env.VITE_SDKWORK_KNOWLEDGEBASE_FEATURE_VERSION_HISTORY, true),
    documentPermissionsModal: readBooleanEnv(env.VITE_SDKWORK_KNOWLEDGEBASE_FEATURE_DOCUMENT_PERMISSIONS, true),
    knowledgeMarketCatalog: devPreview && readBooleanEnv(env.VITE_SDKWORK_KNOWLEDGEBASE_FEATURE_KNOWLEDGE_MARKET, false),
    notesImport: devPreview && readBooleanEnv(env.VITE_SDKWORK_KNOWLEDGEBASE_FEATURE_NOTES_IMPORT, false),
    chatFileImport: devPreview && readBooleanEnv(env.VITE_SDKWORK_KNOWLEDGEBASE_FEATURE_CHAT_FILE_IMPORT, false),
    chatDialogImport: devPreview && readBooleanEnv(env.VITE_SDKWORK_KNOWLEDGEBASE_FEATURE_CHAT_DIALOG_IMPORT, false),
    aiImageGeneration: devPreview && readBooleanEnv(env.VITE_SDKWORK_KNOWLEDGEBASE_FEATURE_AI_IMAGE, false),
    websiteDeploy: devPreview && readBooleanEnv(env.VITE_SDKWORK_KNOWLEDGEBASE_FEATURE_WEBSITE_DEPLOY, false),
    wechatFullPublish: devPreview && readBooleanEnv(env.VITE_SDKWORK_KNOWLEDGEBASE_FEATURE_WECHAT_PUBLISH, false),
  };
}
