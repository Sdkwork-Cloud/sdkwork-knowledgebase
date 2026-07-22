import type { DriveEventReceiptResourceData } from './drive-event-receipt-resource-data';

export interface DriveEventReceiptResponse {
  code: 0;
  data: unknown & DriveEventReceiptResourceData;
  traceId: string;
}
