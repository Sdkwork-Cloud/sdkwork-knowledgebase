export interface OkfBundleExportRequest {
  spaceId: number;
  exportType: string;
  stageForImport?: boolean;
  importId?: string;
}
