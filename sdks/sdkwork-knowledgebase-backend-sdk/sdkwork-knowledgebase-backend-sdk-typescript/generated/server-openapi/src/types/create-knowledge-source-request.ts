import type { KnowledgeSourceType } from './knowledge-source-type';

export interface CreateKnowledgeSourceRequest {
  spaceId: number;
  sourceType: KnowledgeSourceType;
  provider?: string | null;
  driveBucket?: string | null;
  drivePrefix?: string | null;
}
