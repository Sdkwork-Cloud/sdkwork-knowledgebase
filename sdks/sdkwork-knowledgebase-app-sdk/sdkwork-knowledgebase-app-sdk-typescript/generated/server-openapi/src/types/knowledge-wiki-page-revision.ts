export interface KnowledgeWikiPageRevision {
  id: number;
  pageId: number;
  revisionNo: number;
  markdownObjectRefId: number;
  contentHash: string;
  reviewState: unknown;
  createdAt: string;
}
