/** Publish the exact current Drive version as PUBLIC or UNLISTED. */
export interface PublishKnowledgeWikiSourceFileRequest {
  visibility: 'unlisted' | 'public';
  expectedPublicationVersion: string;
  expectedPageVersion: string;
}
