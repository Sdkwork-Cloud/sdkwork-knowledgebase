import type { DriveEventReceiveDisposition } from './drive-event-receive-disposition';
import type { PositiveInt64String } from './positive-int64-string';

export interface DriveEventReceipt {
  eventId: string;
  checkpointId: PositiveInt64String;
  sequenceNo: PositiveInt64String;
  disposition: DriveEventReceiveDisposition;
}
