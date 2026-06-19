export interface KnowledgeAgentChatCitation {
  documentId?: string | null;
  wikiPageId?: string | null;
  title: string;
  sourceUri?: string | null;
  logicalPath?: string | null;
  locator?: string | null;
  score?: number | null;
  snippet?: string | null;
}
