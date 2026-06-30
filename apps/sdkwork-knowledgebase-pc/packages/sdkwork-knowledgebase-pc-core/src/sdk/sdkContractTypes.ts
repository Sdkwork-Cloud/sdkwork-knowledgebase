/**
 * Composed app SDK contract types re-exported for capability packages.
 * Feature modules must import SDK DTO types from pc-core, not from generated SDK packages.
 */
export type {
  IngestionJob,
  KnowledgeAccessLevel,
  KnowledgeBrowserNode,
  KnowledgeDocumentContent,
  KnowledgeSpaceMemberRole,
  KnowledgeWechatApplet,
  KnowledgeWechatArticle,
  KnowledgeWechatOfficialAccount,
  SdkworkAppClient,
} from '@sdkwork/knowledgebase-app-sdk';

export type {
  DriveNode,
  DriveUploaderProfile,
  SdkworkDriveAppClient,
} from '@sdkwork/drive-app-sdk';
