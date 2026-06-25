export interface KnowledgeIngestRequest {
  spaceId: number;
  title: string;
  payloadMarkdown?: string;
  sourceUrl?: string;
  idempotencyKey: string;
}
