import { isKnowledgebaseApiAvailable, KnowledgebaseErrorCodes, throwKnowledgebaseError } from 'sdkwork-knowledgebase-pc-core';

import * as KnowledgeDriveImportService from './knowledgeDriveImportService';

function requireKnowledgebaseApi(): void {
  if (!isKnowledgebaseApiAvailable()) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.API_UNAVAILABLE);
  }
}

/**
 * Frontend service facade for enterprise Drive browse/import flows.
 */
export class CloudDriveService {
  static async listBrowserItems(
    kbId: string,
    folderId: string | null,
  ): Promise<KnowledgeDriveImportService.CloudDriveBrowserItem[]> {
    requireKnowledgebaseApi();
    return KnowledgeDriveImportService.listCloudDriveBrowserItems(kbId, folderId);
  }

  static async listStarredItems(
    kbId: string,
  ): Promise<KnowledgeDriveImportService.CloudDriveBrowserItem[]> {
    requireKnowledgebaseApi();
    return KnowledgeDriveImportService.listStarredCloudDriveItems(kbId);
  }

  static async listRecentItems(
    kbId: string,
  ): Promise<KnowledgeDriveImportService.CloudDriveBrowserItem[]> {
    requireKnowledgebaseApi();
    return KnowledgeDriveImportService.listRecentCloudDriveItems(kbId);
  }

  static async listSharedItems(
    kbId: string,
  ): Promise<KnowledgeDriveImportService.CloudDriveBrowserItem[]> {
    requireKnowledgebaseApi();
    return KnowledgeDriveImportService.listSharedCloudDriveItems(kbId);
  }

  static async importItems(
    kbId: string,
    items: KnowledgeDriveImportService.CloudDriveBrowserItem[],
    targetParentFolderId?: string | null,
  ): Promise<KnowledgeDriveImportService.CloudDriveImportResultItem[]> {
    requireKnowledgebaseApi();
    return KnowledgeDriveImportService.importCloudDriveItems(kbId, items, targetParentFolderId);
  }
}

export type {
  CloudDriveBrowserItem,
  CloudDriveImportFailure,
  CloudDriveImportResultItem,
} from './knowledgeDriveImportService';
