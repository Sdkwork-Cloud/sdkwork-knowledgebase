import type { KnowledgeSourceType } from './knowledge-source-type';

export interface KnowledgeSource {
  id: string;
  spaceId: string;
  sourceType: KnowledgeSourceType;
  provider?: string | null;
  driveBucket?: string | null;
  drivePrefix?: string | null;
}
