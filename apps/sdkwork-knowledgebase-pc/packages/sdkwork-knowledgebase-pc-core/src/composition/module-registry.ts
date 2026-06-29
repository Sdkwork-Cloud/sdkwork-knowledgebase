export function createSdkworkKnowledgebasePcModuleRegistry() {
  return {
    knowledgebase: 'sdkwork-knowledgebase-pc-knowledgebase',
    search: 'sdkwork-knowledgebase-pc-search',
    knowledge: 'sdkwork-knowledgebase-pc-knowledge',
    shell: 'sdkwork-knowledgebase-pc-shell',
    commons: 'sdkwork-knowledgebase-pc-commons',
  } as const;
}
