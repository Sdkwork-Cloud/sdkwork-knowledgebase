export interface KnowledgeTenantQuotaStatus {
  maxDocuments: string;
  documentCount: string;
  maxConcurrentIngestJobs: number;
  inflightIngestJobs: number;
  maxRetrievalsPerMinute: number;
  maxStorageBytes: string;
  storageBytesUsed: string;
}
