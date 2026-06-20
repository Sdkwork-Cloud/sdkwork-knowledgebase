import type { OkfBundleFileKind } from './okf-bundle-file-kind';

export interface KnowledgeOkfBundleFile {
  id: number;
  spaceId: number;
  logicalPath: string;
  entryType: OkfBundleFileKind;
  artifactRole: string;
  driveBucket: string;
  driveObjectKey: string;
  checksumSha256Hex?: string | null;
}
