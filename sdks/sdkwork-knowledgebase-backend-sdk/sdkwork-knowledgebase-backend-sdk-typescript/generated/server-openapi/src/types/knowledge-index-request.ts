export interface KnowledgeIndexRequest {
  spaceId: string;
  collectionId?: string | null;
  indexKind: string;
  embeddingProviderId?: string | null;
  embeddingModel?: string | null;
  dimension?: number | null;
  metric?: string | null;
}
