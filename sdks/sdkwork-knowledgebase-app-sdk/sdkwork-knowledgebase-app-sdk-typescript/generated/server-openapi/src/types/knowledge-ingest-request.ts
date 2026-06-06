export interface KnowledgeIngestRequest {
  spaceId: number;
  title: string;
  payloadMarkdown: string;
  idempotencyKey: string;
}
