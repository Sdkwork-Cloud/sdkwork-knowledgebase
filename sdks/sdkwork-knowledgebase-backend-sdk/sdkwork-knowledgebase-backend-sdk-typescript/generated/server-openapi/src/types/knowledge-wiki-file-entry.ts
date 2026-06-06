import type { WikiFileEntryType } from './wiki-file-entry-type';

export interface KnowledgeWikiFileEntry {
  id: number;
  spaceId: number;
  logicalPath: string;
  entryType: WikiFileEntryType;
  artifactRole: string;
  driveBucket: string;
  driveObjectKey: string;
  checksumSha256Hex?: string | null;
}
