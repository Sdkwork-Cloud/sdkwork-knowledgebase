import type { KnowledgeSiteReleaseState } from './knowledge-site-release-state';

export interface KnowledgeSiteRelease {
  id: string;
  uuid: string;
  siteId: string;
  lifecycleState: KnowledgeSiteReleaseState;
  sourceContentHash: string;
  manifestDriveUri?: string | null;
  manifestDriveSpaceId?: string | null;
  manifestDriveNodeId?: string | null;
  manifestChecksumSha256Hex?: string | null;
  pageCount: number;
  assetCount: number;
  previousReleaseId?: string | null;
  errorCode?: string | null;
  createdAt: string;
  completedAt?: string;
  version: string;
}
