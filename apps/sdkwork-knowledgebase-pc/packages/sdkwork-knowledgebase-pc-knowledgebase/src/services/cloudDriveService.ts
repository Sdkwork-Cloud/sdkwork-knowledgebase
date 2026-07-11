import {
  requireKnowledgebaseApiAvailable,
  requireKnowledgebaseNetworkOnline,
} from 'sdkwork-knowledgebase-pc-core';

import type { CloudDriveBrowserItemsPage } from './knowledgeDriveImportService';
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
  static async listBrowserItemsPage(
    kbId: string,
    folderId: string | null,
    cursor?: string | null,
  ): Promise<CloudDriveBrowserItemsPage> {
    return withCloudDriveApi(() =>
      KnowledgeDriveImportService.listCloudDriveBrowserItemsPage(kbId, folderId, cursor),
    );
  }

  static async listStarredItemsPage(
    kbId: string,
    cursor?: string | null,
  ): Promise<CloudDriveBrowserItemsPage> {
    return withCloudDriveApi(() => KnowledgeDriveImportService.listStarredCloudDriveItemsPage(kbId, cursor));
  }

  static async listRecentItemsPage(
    kbId: string,
    cursor?: string | null,
  ): Promise<CloudDriveBrowserItemsPage> {
    return withCloudDriveApi(() => KnowledgeDriveImportService.listRecentCloudDriveItemsPage(kbId, cursor));
  }

  static async listSharedItemsPage(
    kbId: string,
    cursor?: string | null,
  ): Promise<CloudDriveBrowserItemsPage> {
    return withCloudDriveApi(() => KnowledgeDriveImportService.listSharedCloudDriveItemsPage(kbId, cursor));
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
  CloudDriveBrowserItemsPage,
  CloudDriveBrowserItem,
  CloudDriveImportFailure,
  CloudDriveImportResultItem,
} from './knowledgeDriveImportService';
