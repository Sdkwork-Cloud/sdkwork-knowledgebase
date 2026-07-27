import type { KnowledgeWikiPublicationMode } from './knowledge-wiki-publication-mode';
import type { KnowledgeWikiPublicationStatus } from './knowledge-wiki-publication-status';
import type { KnowledgeWikiUpdatePolicy } from './knowledge-wiki-update-policy';
import type { KnowledgeWikiVisibility } from './knowledge-wiki-visibility';

/** Canonical Wiki publication state for one Knowledgebase. */
export interface KnowledgeWikiPublication {
  uuid: string;
  spaceId: string;
  driveSpaceUuid: string;
  sourceRootNodeUuid: string | null;
  status: KnowledgeWikiPublicationStatus;
  title: string;
  homepageSourcePath: string;
  publicationMode: KnowledgeWikiPublicationMode;
  defaultVisibility: KnowledgeWikiVisibility;
  updatePolicy: KnowledgeWikiUpdatePolicy;
  providerGeneration: string;
  navigationGeneration: string;
  searchGeneration: string;
  lastProjectedDriveCheckpoint: string;
  version: string;
}
