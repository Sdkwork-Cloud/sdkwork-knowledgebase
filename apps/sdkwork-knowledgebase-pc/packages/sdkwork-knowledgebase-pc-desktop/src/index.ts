export const sdkworkKnowledgebasePcDesktopHost = {
  packageName: 'sdkwork-knowledgebase-pc-desktop',
  hostProfile: 'tauri-v2',
  rendererOwner: 'apps/sdkwork-knowledgebase-pc',
  runtimeTargets: ['desktop-windows', 'desktop-macos', 'desktop-linux'] as const,
} as const;

export type SdkworkKnowledgebasePcDesktopHost = typeof sdkworkKnowledgebasePcDesktopHost;
export type SdkworkKnowledgebasePcDesktopRuntimeTarget =
  SdkworkKnowledgebasePcDesktopHost['runtimeTargets'][number];
