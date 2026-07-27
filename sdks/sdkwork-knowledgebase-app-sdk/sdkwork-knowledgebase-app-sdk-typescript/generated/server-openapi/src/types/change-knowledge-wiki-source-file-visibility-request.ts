import type { KnowledgeWikiVisibility } from './knowledge-wiki-visibility';

/** Change a published Wiki source file visibility with version fencing. */
export interface ChangeKnowledgeWikiSourceFileVisibilityRequest {
  visibility: KnowledgeWikiVisibility;
  expectedPublicationVersion: string;
  expectedPageVersion: string;
}
