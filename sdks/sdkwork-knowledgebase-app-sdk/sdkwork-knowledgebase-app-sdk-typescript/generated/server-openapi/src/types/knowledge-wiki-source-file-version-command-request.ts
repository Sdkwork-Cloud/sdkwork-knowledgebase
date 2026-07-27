/** Optimistic Wiki source-file publication command. */
export interface KnowledgeWikiSourceFileVersionCommandRequest {
  expectedPublicationVersion: string;
  expectedPageVersion: string;
}
