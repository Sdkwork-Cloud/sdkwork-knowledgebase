import type { KnowledgeWikiIndexState } from './knowledge-wiki-index-state';
import type { KnowledgeWikiPagePublicationState } from './knowledge-wiki-page-publication-state';
import type { KnowledgeWikiSourceFileKind } from './knowledge-wiki-source-file-kind';
import type { KnowledgeWikiSourceState } from './knowledge-wiki-source-state';
import type { KnowledgeWikiVisibility } from './knowledge-wiki-visibility';

/** Projected sources/raw file state and its pinned public version. */
export interface KnowledgeWikiSourceFile {
  uuid: string;
  driveNodeUuid: string;
  driveVersionUuid: string;
  sourcePath: string;
  canonicalRoute: string | null;
  fileKind: KnowledgeWikiSourceFileKind;
  mediaType: string;
  sizeBytes: string;
  contentSha256: string;
  sourceState: KnowledgeWikiSourceState;
  publicationState: KnowledgeWikiPagePublicationState;
  visibility: KnowledgeWikiVisibility;
  indexState: KnowledgeWikiIndexState;
  publicDriveVersionUuid: string | null;
  pagePublicVersion: string;
  version: string;
}
