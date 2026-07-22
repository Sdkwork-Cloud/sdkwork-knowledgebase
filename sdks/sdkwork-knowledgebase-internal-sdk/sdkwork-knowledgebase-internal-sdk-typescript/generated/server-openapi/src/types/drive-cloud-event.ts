import type { NonNegativeInt64String } from './non-negative-int64-string';
import type { PositiveInt64String } from './positive-int64-string';

export interface DriveCloudEvent {
  specversion: '1.0';
  id: string;
  source: 'sdkwork.drive';
  type: 'drive.node.version.committed.v1' | 'drive.node.path.changed.v1' | 'drive.node.eligibility.changed.v1' | 'drive.node.deleted.v1';
  time: string;
  tenantId: PositiveInt64String;
  organizationId?: NonNegativeInt64String;
  subject?: string;
  actorId?: PositiveInt64String;
  sequenceNo: PositiveInt64String;
  data: Record<string, unknown>;
}
