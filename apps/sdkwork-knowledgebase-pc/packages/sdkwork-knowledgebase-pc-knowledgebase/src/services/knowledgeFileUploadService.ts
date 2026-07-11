import type { DriveUploaderProfile } from 'sdkwork-knowledgebase-pc-core';
import { formatBytes, isBlank } from '@sdkwork/utils';
import {
  getKnowledgebaseAppSdkClient,
  getKnowledgebaseTenantId,
  isKnowledgebaseDriveApiAvailable,
  KnowledgebaseErrorCodes,
  parseKnowledgeSpaceId,
  readRegisteredSpaces,
  requireDriveApiClient,
  requireKnowledgebaseAppSdkHttpClient,
  throwKnowledgebaseError,
} from 'sdkwork-knowledgebase-pc-core';

import type { DocumentMeta } from './document';
import { ensureDriveFolderPath } from './knowledgeDriveBrowserService';
import { invalidateKnowledgeBrowserNodeCacheForKbIds } from './knowledgeBrowserListService';
import { resolveKnowledgeBrowserParentDriveNodeId } from './knowledgeBrowserParentResolver';
import { placeDocumentInParentFolder } from './knowledgebaseDocumentApiBridge';
import { resolveDriveNodeDownloadUrl } from './knowledgeDriveMediaService';
import { resolveIngestedDocument, waitForIngestJob } from './knowledgeIngestService';

const MAX_TEXT_BYTES = 512 * 1024;
const TEXT_EXTENSIONS = new Set([
  '.md',
  '.markdown',
  '.txt',
  '.json',
  '.yaml',
  '.yml',
  '.toml',
  '.csv',
  '.ts',
  '.tsx',
  '.js',
  '.jsx',
  '.py',
  '.java',
  '.go',
  '.rs',
  '.html',
  '.htm',
  '.css',
  '.xml',
]);

export interface KnowledgeFileUploadFailure {
  fileName: string;
  message: string;
}

function spaceIdFromKbId(kbId: string): string {
  return parseKnowledgeSpaceId(kbId);
}

function inferDocumentType(
  file: File,
  overrideType?: DocumentMeta['type'],
): DocumentMeta['type'] {
  if (overrideType) {
    return overrideType;
  }
  if (file.type.startsWith('image/')) {
    return 'image';
  }
  if (file.type.startsWith('video/')) {
    return 'video';
  }
  if (file.type.startsWith('audio/')) {
    return 'audio';
  }
  if (file.type === 'application/pdf' || file.name.toLowerCase().endsWith('.pdf')) {
    return 'pdf';
  }
  if (file.name.toLowerCase().endsWith('.md')) {
    return 'markdown';
  }
  if (/\.(ts|js|jsx|tsx|html|htm|css|json|xml|py|java|cpp|c|go|rs|php|rb|swift|kt|sql|sh|yaml|yml)$/i.test(file.name)) {
    return 'code';
  }
  return 'file';
}

function isTextIngestible(file: File): boolean {
  if (file.size > MAX_TEXT_BYTES) {
    return false;
  }
  const lower = file.name.toLowerCase();
  const dot = lower.lastIndexOf('.');
  if (dot < 0) {
    return false;
  }
  return TEXT_EXTENSIONS.has(lower.slice(dot));
}

function inferUploaderProfile(file: File): DriveUploaderProfile {
  if (file.type.startsWith('image/')) {
    return 'image';
  }
  if (file.type.startsWith('video/')) {
    return 'video';
  }
  if (file.type.startsWith('audio/')) {
    return 'audio';
  }
  if (file.type === 'application/pdf' || file.name.toLowerCase().endsWith('.pdf')) {
    return 'document';
  }
  if (isTextIngestible(file)) {
    return 'text';
  }
  return 'attachment';
}

function buildIdempotencyKey(spaceId: string, file: File, index: number): string {
  const raw = `pc-upload-${spaceId}-${file.name}-${file.size}-${file.lastModified}-${index}`;
  return raw.replace(/[^a-zA-Z0-9._-]/g, '-').slice(0, 128);
}

function resolveUploadTitle(file: File): string {
  const relativePath = (file as File & { webkitRelativePath?: string }).webkitRelativePath;
  if (relativePath && relativePath.includes('/')) {
    return relativePath.split('/').pop() || file.name;
  }
  return file.name;
}

async function resolveDriveSpaceId(spaceId: string): Promise<string> {
  const client = getKnowledgebaseAppSdkClient();
  const space = await client.client.knowledge.spaces.retrieve(spaceId);
  const driveSpaceId = space.driveSpaceId?.trim();
  if (!driveSpaceId) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.DRIVE_SPACE_MISSING);
  }
  return driveSpaceId;
}

function toDocumentMeta(
  kbId: string,
  file: File,
  documentId: number | string,
  title: string,
  type: DocumentMeta['type'],
  parentId?: string | null,
): DocumentMeta {
  return {
    id: String(documentId),
    title,
    type,
    kbId,
    parentId: parentId ?? null,
    updatedAt: new Date().toISOString(),
    author: 'Knowledgebase',
    size: formatBytes(file.size),
  };
}

async function ingestTextFile(
  spaceId: string,
  kbId: string,
  file: File,
  type: DocumentMeta['type'],
  index: number,
  parentId?: string | null,
  folderCache?: Map<string, string>,
): Promise<DocumentMeta> {
  const client = getKnowledgebaseAppSdkClient();
  const title = resolveUploadTitle(file);
  const content = await file.text();
  if (isBlank(content)) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.FILE_EMPTY);
  }

  if (isKnowledgebaseDriveApiAvailable()) {
    const driveSpaceId = await resolveDriveSpaceId(spaceId);
    const resolvedParentId = await resolveUploadParentNodeId(
      kbId,
      driveSpaceId,
      file,
      parentId,
      folderCache ?? new Map<string, string>(),
    );
    return uploadBinaryThroughDrive(
      spaceId,
      kbId,
      file,
      type,
      index,
      parentId,
      folderCache,
      resolvedParentId,
    );
  }

  const job = await client.client.knowledge.ingests.create({
    spaceId,
    title,
    payloadMarkdown: content,
    idempotencyKey: buildIdempotencyKey(spaceId, file, index),
  });

  const finalJob = job.state === 'succeeded' ? job : await waitForIngestJob(job.id);
  if (finalJob.state !== 'succeeded') {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.INGEST_FAILED, {
      cause: finalJob.errorMessage ?? undefined,
    });
  }

  const document = await resolveIngestedDocument(spaceId, title);
  const meta = toDocumentMeta(kbId, file, document.id, document.title, type, parentId);
  if (parentId?.trim()) {
    await placeDocumentInParentFolder(meta.id, kbId, parentId);
  }
  return meta;
}

function resolveRelativeFolderPath(file: File): string | undefined {
  const relativePath = (file as File & { webkitRelativePath?: string }).webkitRelativePath?.trim();
  return relativePath && relativePath.includes('/') ? relativePath : undefined;
}

async function resolveUploadParentNodeId(
  kbId: string,
  driveSpaceId: string,
  file: File,
  parentId: string | null | undefined,
  folderCache: Map<string, string>,
): Promise<string | undefined> {
  const driveParentId = await resolveKnowledgeBrowserParentDriveNodeId(kbId, parentId);
  const relativePath = resolveRelativeFolderPath(file);
  if (!relativePath) {
    return driveParentId;
  }
  return ensureDriveFolderPath(driveSpaceId, driveParentId, relativePath, folderCache);
}

async function uploadBinaryThroughDrive(
  spaceId: string,
  kbId: string,
  file: File,
  type: DocumentMeta['type'],
  index: number,
  parentId?: string | null,
  folderCache?: Map<string, string>,
  resolvedDriveParentId?: string | undefined,
): Promise<DocumentMeta> {
  if (!isKnowledgebaseDriveApiAvailable()) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.API_UNAVAILABLE_DRIVE);
  }

  const knowledgeClient = getKnowledgebaseAppSdkClient();
  const driveClient = requireDriveApiClient();
  const driveSpaceId = await resolveDriveSpaceId(spaceId);
  const title = resolveUploadTitle(file);
  const resolvedParentId = resolvedDriveParentId ?? await resolveUploadParentNodeId(
    kbId,
    driveSpaceId,
    file,
    parentId,
    folderCache ?? new Map<string, string>(),
  );

  const uploadResult = await driveClient.uploader.upload({
    file,
    appResourceType: 'knowledgebase-pc-file-upload',
    appResourceId: String(spaceId),
    scene: 'knowledgebase_pc_upload',
    source: 'pc_local_file',
    spaceId: driveSpaceId,
    parentNodeId: resolvedParentId,
    uploadProfileCode: inferUploaderProfile(file),
    originalFileName: title,
    contentType: file.type || undefined,
  });

  const importResult = await knowledgeClient.client.knowledge.driveImports.create({
    spaceId,
    title,
    idempotencyKey: buildIdempotencyKey(spaceId, file, index),
    driveSpaceId,
    driveNodeId: uploadResult.uploadItem.nodeId,
    driveStorageProviderId: '',
    driveBucket: '',
    driveObjectKey: '',
    language: null,
  });

  const meta = toDocumentMeta(
    kbId,
    file,
    importResult.document.id,
    importResult.document.title,
    type,
    parentId ?? null,
  );

  try {
    const url = await resolveDriveNodeDownloadUrl(uploadResult.uploadItem.nodeId);
    if (url) {
      return { ...meta, url };
    }
  } catch {
    // Keep metadata without a transient download URL when Drive URL resolution fails.
  }

  return meta;
}

async function uploadSingleFile(
  spaceId: string,
  kbId: string,
  file: File,
  index: number,
  overrideType?: DocumentMeta['type'],
  parentId?: string | null,
  folderCache?: Map<string, string>,
): Promise<DocumentMeta> {
  const type = inferDocumentType(file, overrideType);
  if (isTextIngestible(file)) {
    return ingestTextFile(spaceId, kbId, file, type, index, parentId, folderCache);
  }
  return uploadBinaryThroughDrive(spaceId, kbId, file, type, index, parentId, folderCache);
}

export async function uploadKnowledgebaseFiles(
  files: File[],
  kbId: string,
  overrideType?: DocumentMeta['type'],
  parentId?: string | null,
): Promise<DocumentMeta[]> {
  const spaceId = spaceIdFromKbId(kbId);
  const results: DocumentMeta[] = [];
  const failures: KnowledgeFileUploadFailure[] = [];
  const folderCache = new Map<string, string>();

  for (let index = 0; index < files.length; index += 1) {
    const file = files[index];
    try {
      results.push(
        await uploadSingleFile(spaceId, kbId, file, index, overrideType, parentId, folderCache),
      );
    } catch (error) {
      failures.push({
        fileName: file.name,
        message: error instanceof Error ? error.message : String(error),
      });
    }
  }

  if (results.length === 0 && failures.length > 0) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.OPERATION_FAILED, { cause: failures });
  }

  if (failures.length > 0) {
    console.warn(
      '[KnowledgeFileUploadService] partial upload failures',
      failures,
    );
  }

  if (results.length > 0) {
    invalidateKnowledgeBrowserNodeCacheForKbIds(kbId);
  }

  return results;
}

export function resolvePrimaryKnowledgebaseKbId(): string | null {
  const tenantId = getKnowledgebaseTenantId();
  if (!tenantId) {
    return null;
  }
  const spaces = readRegisteredSpaces(tenantId);
  if (spaces.length === 0) {
    return null;
  }
  return String(spaces[0].spaceId);
}

export async function uploadKnowledgebaseMediaUrl(
  file: File,
  mediaType: DocumentMeta['type'],
  kbId: string,
): Promise<string> {
  const uploaded = await uploadKnowledgebaseFiles([file], kbId, mediaType);
  const url = uploaded[0]?.url;
  if (!url) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.MEDIA_URL_UNRESOLVED);
  }
  return url;
}
