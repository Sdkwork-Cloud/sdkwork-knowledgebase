export interface KnowledgeIndexRequest {
  tenantId: string;
  spaceId: string;
  collectionId?: string | null;
  indexKind: string;
  embeddingProviderId?: string | null;
  embeddingModel?: string | null;
  dimension?: number | null;
  metric?: string | null;
}
