import type { KnowledgeWikiPublication } from './knowledge-wiki-publication';
import type { KnowledgeWikiSourceFile } from './knowledge-wiki-source-file';

/** Updated publication and source-file state after a Wiki command. */
export interface KnowledgeWikiSourceFileCommandResult {
  publication: KnowledgeWikiPublication;
  sourceFile: KnowledgeWikiSourceFile;
}
