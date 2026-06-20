export interface KnowledgeAgentChatCitation {
  documentId?: string | null;
  conceptId?: string | null;
  title: string;
  sourceUri?: string | null;
  logicalPath?: string | null;
  locator?: string | null;
  score?: number | null;
  snippet?: string | null;
}
