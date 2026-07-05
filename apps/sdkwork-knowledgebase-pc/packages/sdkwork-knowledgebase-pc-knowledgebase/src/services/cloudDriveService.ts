import {
  requireKnowledgebaseApiAvailable,
  requireKnowledgebaseNetworkOnline,
} from 'sdkwork-knowledgebase-pc-core';

import * as KnowledgeDriveImportService from './knowledgeDriveImportService';

function withCloudDriveApi<T>(apiCall: () => Promise<T>): Promise<T> {
  requireKnowledgebaseApiAvailable();
  requireKnowledgebaseNetworkOnline();
  return apiCall();
}

/**
 * Frontend service facade for enterprise Drive browse/import flows.
 */
export class CloudDriveService {
  static async listBrowserItems(
    kbId: string,
    folderId: string | null,
  ): Promise<KnowledgeDriveImportService.CloudDriveBrowserItem[]> {
    return withCloudDriveApi(() => KnowledgeDriveImportService.listCloudDriveBrowserItems(kbId, folderId));
  }

  static async listStarredItems(
    kbId: string,
  ): Promise<KnowledgeDriveImportService.CloudDriveBrowserItem[]> {
    return withCloudDriveApi(() => KnowledgeDriveImportService.listStarredCloudDriveItems(kbId));
  }

  static async listRecentItems(
    kbId: string,
  ): Promise<KnowledgeDriveImportService.CloudDriveBrowserItem[]> {
    return withCloudDriveApi(() => KnowledgeDriveImportService.listRecentCloudDriveItems(kbId));
  }

  static async listSharedItems(
    kbId: string,
  ): Promise<KnowledgeDriveImportService.CloudDriveBrowserItem[]> {
    return withCloudDriveApi(() => KnowledgeDriveImportService.listSharedCloudDriveItems(kbId));
  }

  static async importItems(
    kbId: string,
    items: KnowledgeDriveImportService.CloudDriveBrowserItem[],
    targetParentFolderId?: string | null,
  ): Promise<KnowledgeDriveImportService.CloudDriveImportResultItem[]> {
    return withCloudDriveApi(() =>
      KnowledgeDriveImportService.importCloudDriveItems(kbId, items, targetParentFolderId),
    );
  }
}

export type {
  CloudDriveBrowserItem,
  CloudDriveImportFailure,
  CloudDriveImportResultItem,
} from './knowledgeDriveImportService';
