/** Mirrors pc-core specs/component.spec.json#contracts.sdkDependencies workspace order. */
export const sdkworkKnowledgebasePcSdkInventory = [
  'sdkwork-appbase',
  'sdkwork-database',
  'sdkwork-drive',
  'sdkwork-drive-app-sdk',
  'sdkwork-iam-app-sdk',
  'sdkwork-id',
  'sdkwork-kernel',
  'sdkwork-knowledgebase-app-sdk',
  'sdkwork-knowledgebase-backend-sdk',
  'sdkwork-memory',
  'sdkwork-sdk-generator',
  'sdkwork-utils',
  'sdkwork-web-framework',
] as const;

export function listSdkworkKnowledgebasePcAppSdkFamilies() {
  return sdkworkKnowledgebasePcSdkInventory;
}
