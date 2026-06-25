import { isBlank } from '@sdkwork/sdkwork-knowledgebase-pc-commons/stringUtils';
import {
  KnowledgebaseErrorCodes,
  requireKnowledgebaseApiAvailable,
  requireNonEmptyString,
} from 'sdkwork-knowledgebase-pc-core';
import * as KnowledgebaseDocumentApiBridge from './knowledgebaseDocumentApiBridge';
import * as KnowledgeGitImportService from './knowledgeGitImportService';
import * as KnowledgeGitSyncService from './knowledgeGitSyncService';
import * as KnowledgeMarketService from './knowledgeMarketService';
import * as KnowledgeSiteDeploymentService from './knowledgeSiteDeploymentService';
import * as KnowledgeFileUploadService from './knowledgeFileUploadService';
import { importWebLinkToKnowledgeBase } from './knowledgeWebLinkImportService';
import {
  listKnowledgeAssetLibraryItems,
  searchKnowledgeMediaDocuments,
  type KnowledgeAssetLibraryItem,
} from './knowledgeAssetLibraryService';
import { KnowledgeSpaceMembersService, type KnowledgeSpaceMemberUi } from './knowledgeSpaceMembersService';
export type { KnowledgeSpaceMemberUi } from './knowledgeSpaceMembersService';

export interface DocumentMeta {
  id: string;
  title: string;
  type: 'richtext' | 'code' | 'markdown' | 'file' | 'image' | 'audio' | 'video' | 'folder' | 'pdf' | 'music';
  updatedAt: string;
  author: string;
  kbId?: string;
  size?: string;
  url?: string;
  content?: string;
  parentId?: string | null;
  order?: number;
  isPinned?: boolean;
  tags?: string[];
}

export interface FolderNode {
  id: string;
  title: string;
  type: 'folder';
  children: (FolderNode | DocumentMeta)[];
  parentId?: string | null;
  updatedAt?: string;
  author?: string;
  isPinned?: boolean;
  tags?: string[];
}

export interface KnowledgeBase {
  id: string;
  title: string;
  icon?: string;
  avatar?: string;
  type?: 'team' | 'personal' | 'public';
  isDeployed?: boolean;
  deployedUrl?: string;
  customDomain?: string;
  siteLogo?: string;
  siteName?: string;
  provider?: string;
  modelName?: string;
  temperature?: number;
  maxTokens?: number;
  systemPrompt?: string;
  publicPermission?: 'none' | 'read' | 'write' | 'admin';
  guestLinkEnabled?: boolean;
}

export interface MarketKnowledgeBase {
  id: string;
  title: string;
  icon: string;
  description: string;
  author: string;
  tags: string[];
  subscribersCount: number;
  documentsCount: number;
  provider: string;
  modelName: string;
  isSubscribed?: boolean;
}

export interface DocumentVersionSummary {
  id: number;
  documentId: number;
  versionNo: number;
  sizeBytes: number;
  mimeType?: string | null;
  parseState: string;
  indexState: string;
}

export type KnowledgeDocumentVisibility =
  | 'private'
  | 'space'
  | 'organization'
  | 'public';

export interface DocumentAccessSummary {
  documentId: number;
  spaceId: number;
  title: string;
  visibility: KnowledgeDocumentVisibility;
}

function requireKnowledgebaseApi(): void {
  requireKnowledgebaseApiAvailable();
}

async function withKnowledgebaseApi<T>(apiCall: () => Promise<T>): Promise<T> {
  requireKnowledgebaseApi();
  return apiCall();
}

/**
 * Frontend service facade over the generated Knowledgebase app SDK bridge.
 */
export class DocumentService {
  static async getKnowledgeBases(): Promise<{ team: KnowledgeBase[]; personal: KnowledgeBase[]; public: KnowledgeBase[] }> {
    return withKnowledgebaseApi(() => KnowledgebaseDocumentApiBridge.getKnowledgeBases());
  }

  static async createKnowledgeBase(kb: Partial<KnowledgeBase>): Promise<KnowledgeBase> {
    return withKnowledgebaseApi(() => KnowledgebaseDocumentApiBridge.createKnowledgeBase(kb));
  }

  static async hydrateKnowledgeBase(kb: KnowledgeBase): Promise<KnowledgeBase> {
    return withKnowledgebaseApi(() => KnowledgebaseDocumentApiBridge.hydrateKnowledgeBase(kb));
  }

  static async updateKnowledgeBase(id: string, updates: Partial<KnowledgeBase>): Promise<KnowledgeBase> {
    return withKnowledgebaseApi(() => KnowledgebaseDocumentApiBridge.updateKnowledgeBase(id, updates));
  }

  static async deleteKnowledgeBase(id: string): Promise<boolean> {
    return withKnowledgebaseApi(() => KnowledgebaseDocumentApiBridge.deleteKnowledgeBase(id));
  }

  static async getDocuments(kbId: string): Promise<(FolderNode | DocumentMeta)[]> {
    return withKnowledgebaseApi(() => KnowledgebaseDocumentApiBridge.getDocuments(kbId));
  }

  static async ensureFolderChildrenLoaded(
    kbId: string,
    folderId: string | null,
  ): Promise<void> {
    return withKnowledgebaseApi(() => KnowledgebaseDocumentApiBridge.ensureFolderChildrenLoaded(kbId, folderId));
  }

  static async getDocumentContent(id: string): Promise<string> {
    return withKnowledgebaseApi(() => KnowledgebaseDocumentApiBridge.getDocumentContent(id));
  }

  static async hydrateDocumentForViewer(doc: DocumentMeta): Promise<DocumentMeta> {
    return withKnowledgebaseApi(() => KnowledgebaseDocumentApiBridge.hydrateDocumentForViewer(doc));
  }

  static async saveDocumentContent(id: string, content: string): Promise<boolean> {
    return withKnowledgebaseApi(() => KnowledgebaseDocumentApiBridge.saveDocumentContent(id, content));
  }

  static async updateDocument(id: string, updates: Partial<DocumentMeta>): Promise<boolean> {
    return withKnowledgebaseApi(() => KnowledgebaseDocumentApiBridge.updateDocument(id, updates));
  }

  static async copyDocument(
    sourceId: string,
    targetKbId: string,
    targetParentId: string | null,
    options?: { titleSuffix?: string },
  ): Promise<DocumentMeta> {
    return withKnowledgebaseApi(() =>
      KnowledgebaseDocumentApiBridge.copyDocument(sourceId, targetKbId, targetParentId, options),
    );
  }

  static async createDocument(doc: Partial<DocumentMeta>): Promise<DocumentMeta> {
    return withKnowledgebaseApi(() => KnowledgebaseDocumentApiBridge.createDocument(doc));
  }

  static async uploadFiles(
    files: File[],
    kbId: string,
    parentId?: string,
    overrideType?: DocumentMeta['type'],
  ): Promise<DocumentMeta[]> {
    requireKnowledgebaseApi();
    return KnowledgeFileUploadService.uploadKnowledgebaseFiles(
      files,
      kbId,
      overrideType,
      parentId,
    );
  }

  static async importWebLink(params: {
    kbId: string;
    parentId?: string | null;
    url: string;
    title?: string;
  }): Promise<DocumentMeta> {
    return withKnowledgebaseApi(() => importWebLinkToKnowledgeBase(params));
  }

  static async deleteDocument(id: string): Promise<boolean> {
    return withKnowledgebaseApi(() => KnowledgebaseDocumentApiBridge.deleteDocument(id));
  }

  static async getMarketKnowledgeBases(): Promise<MarketKnowledgeBase[]> {
    return withKnowledgebaseApi(() => KnowledgeMarketService.listMarketKnowledgeBases());
  }

  static async subscribeMarketKb(id: string): Promise<boolean> {
    return withKnowledgebaseApi(() => KnowledgeMarketService.subscribeMarketListing(id));
  }

  static async unsubscribeMarketKb(id: string): Promise<boolean> {
    return withKnowledgebaseApi(() => KnowledgeMarketService.unsubscribeMarketListing(id));
  }

  static async importGitRepository(
    kbId: string,
    repoUrl: string,
    branch: string = 'main',
    options?: {
      accessToken?: string;
      onProgress?: (progress: KnowledgeGitImportService.GitImportProgress) => void;
    },
  ): Promise<boolean> {
    requireKnowledgebaseApi();
    const result = await KnowledgeGitImportService.importGitRepository(
      kbId,
      repoUrl,
      branch,
      options?.accessToken,
      options?.onProgress,
    );
    return result.importedCount > 0;
  }

  static async syncGitRepository(
    kbId: string,
    commitMessage: string,
    options?: {
      repoUrl: string;
      branch?: string;
      accessToken?: string;
      onProgress?: (progress: KnowledgeGitSyncService.GitSyncProgress) => void;
    },
  ): Promise<{ success: boolean; hash: string }> {
    if (isBlank(options?.repoUrl)) {
      requireNonEmptyString('', KnowledgebaseErrorCodes.REPO_URL_REQUIRED);
    }

    return withKnowledgebaseApi(async () => {
      const result = await KnowledgeGitSyncService.syncGitRepository(
        kbId,
        options.repoUrl,
        options.branch ?? 'main',
        commitMessage,
        options.accessToken,
        options.onProgress,
      );
      return { success: result.success, hash: result.hash };
    });
  }

  static async publishWebsite(
    platform: string,
    targetId: string,
    options?: {
      siteName?: string;
      customDomain?: string;
      siteLogo?: string;
    },
  ): Promise<{ success: boolean; url?: string }> {
    return withKnowledgebaseApi(async () => {
      const result = await KnowledgeSiteDeploymentService.publishKnowledgeSite(
        targetId,
        platform,
        {
          siteName: options?.siteName,
          customDomain: options?.customDomain,
          siteLogoDataUrl: options?.siteLogo,
        },
      );
      return { success: result.success, url: result.url };
    });
  }

  static async searchAll(query: string): Promise<{
    kbs: KnowledgeBase[];
    docs: DocumentMeta[];
  }> {
    return withKnowledgebaseApi(() => KnowledgebaseDocumentApiBridge.searchAll(query));
  }

  static async getRecentDocuments(limit: number = 8): Promise<DocumentMeta[]> {
    return withKnowledgebaseApi(() => KnowledgebaseDocumentApiBridge.getRecentDocuments(limit));
  }

  static async touchDocument(id: string): Promise<boolean> {
    return withKnowledgebaseApi(() => KnowledgebaseDocumentApiBridge.touchDocument(id));
  }

  static async listDocumentVersions(documentId: string): Promise<DocumentVersionSummary[]> {
    return withKnowledgebaseApi(() => KnowledgebaseDocumentApiBridge.listDocumentVersions(documentId));
  }

  static async getDocumentAccess(documentId: string): Promise<DocumentAccessSummary> {
    return withKnowledgebaseApi(() => KnowledgebaseDocumentApiBridge.getDocumentAccess(documentId));
  }

  static async updateDocumentVisibility(
    documentId: string,
    visibility: KnowledgeDocumentVisibility,
  ): Promise<DocumentAccessSummary> {
    return withKnowledgebaseApi(() =>
      KnowledgebaseDocumentApiBridge.updateDocumentVisibility(documentId, visibility),
    );
  }

  static async loadKnowledgeSpaceMembers(spaceId: number): Promise<KnowledgeSpaceMemberUi[]> {
    return withKnowledgebaseApi(() => KnowledgeSpaceMembersService.loadMembers(spaceId));
  }

  static async syncKnowledgeSpaceMembers(
    spaceId: number,
    desired: KnowledgeSpaceMemberUi[],
    previous: KnowledgeSpaceMemberUi[],
  ): Promise<void> {
    return withKnowledgebaseApi(() => KnowledgeSpaceMembersService.syncMembers(spaceId, desired, previous));
  }

  static async listAssetLibraryItems(
    kbId: string,
    assetType: 'image' | 'audio' | 'video',
  ): Promise<KnowledgeAssetLibraryItem[]> {
    return withKnowledgebaseApi(() => listKnowledgeAssetLibraryItems(kbId, assetType));
  }

  static async searchMediaDocuments(query: string, limit: number = 8): Promise<DocumentMeta[]> {
    return withKnowledgebaseApi(() => searchKnowledgeMediaDocuments(query, limit));
  }
}
