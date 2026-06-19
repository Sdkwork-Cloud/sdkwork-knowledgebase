export interface CompleteKnowledgeUploadSessionRequest {
  spaceId: string;
  title: string;
  idempotencyKey: string;
  payloadMarkdown?: string;
}
