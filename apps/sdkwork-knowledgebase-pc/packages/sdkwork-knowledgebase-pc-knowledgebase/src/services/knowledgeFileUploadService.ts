import type { DriveUploaderProfile } from '@sdkwork/drive-app-sdk';
import { isBlank } from '@sdkwork/sdkwork-knowledgebase-pc-commons/stringUtils';
import {
  getKnowledgebaseAppSdkClient,
  getKnowledgebaseDriveAppSdkClient,
  isKnowledgebaseDriveApiAvailable,
} from 'sdkwork-knowledgebase-pc-core';

import type { DocumentMeta } from './document';
import { ensureDriveFolderPath } from './knowledgeDriveBrowserService';
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

function spaceIdFromKbId(kbId: string): number {
  const spaceId = Number(kbId);
  if (!Number.isFinite(spaceId) || spaceId <= 0) {
    throw new Error(`Invalid knowledge space id: ${kbId}`);
  }
  return spaceId;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
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

function buildIdempotencyKey(spaceId: number, file: File, index: number): string {
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

async function resolveDriveSpaceId(spaceId: number): Promise<string> {
  const client = getKnowledgebaseAppSdkClient();
  const space = await client.client.knowledge.spaces.retrieve(spaceId);
  const driveSpaceId = space.driveSpaceId?.trim();
  if (!driveSpaceId) {
    throw new Error('Knowledge space does not have an associated Drive space for file upload.');
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
  spaceId: number,
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
    throw new Error(`File "${file.name}" is empty.`);
  }

  if (isKnowledgebaseDriveApiAvailable() && resolveRelativeFolderPath(file)) {
    const driveSpaceId = await resolveDriveSpaceId(spaceId);
    const resolvedParentId = await resolveUploadParentNodeId(
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
      resolvedParentId ?? null,
      folderCache,
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
    throw new Error(finalJob.errorMessage ?? `Ingest failed for "${file.name}".`);
  }

  const document = await resolveIngestedDocument(spaceId, title);
  return toDocumentMeta(kbId, file, document.id, document.title, type, parentId);
}

function resolveRelativeFolderPath(file: File): string | undefined {
  const relativePath = (file as File & { webkitRelativePath?: string }).webkitRelativePath?.trim();
  return relativePath && relativePath.includes('/') ? relativePath : undefined;
}

async function resolveUploadParentNodeId(
  driveSpaceId: string,
  file: File,
  parentId: string | null | undefined,
  folderCache: Map<string, string>,
): Promise<string | undefined> {
  const relativePath = resolveRelativeFolderPath(file);
  if (!relativePath) {
    return parentId?.trim() || undefined;
  }
  return ensureDriveFolderPath(driveSpaceId, parentId, relativePath, folderCache);
}

async function uploadBinaryThroughDrive(
  spaceId: number,
  kbId: string,
  file: File,
  type: DocumentMeta['type'],
  index: number,
  parentId?: string | null,
  folderCache?: Map<string, string>,
): Promise<DocumentMeta> {
  if (!isKnowledgebaseDriveApiAvailable()) {
    throw new Error('Drive upload is required for binary files but the Drive SDK is not configured.');
  }

  const knowledgeClient = getKnowledgebaseAppSdkClient();
  const driveClient = getKnowledgebaseDriveAppSdkClient();
  const driveSpaceId = await resolveDriveSpaceId(spaceId);
  const title = resolveUploadTitle(file);
  const resolvedParentId = await resolveUploadParentNodeId(
    driveSpaceId,
    file,
    parentId,
    folderCache ?? new Map<string, string>(),
  );

  const uploadResult = await driveClient.client.uploader.upload({
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

  return toDocumentMeta(
    kbId,
    file,
    importResult.document.id,
    importResult.document.title,
    type,
    resolvedParentId ?? null,
  );
}

async function uploadSingleFile(
  spaceId: number,
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
    throw new Error(
      `Upload failed for all files: ${failures.map((entry) => `${entry.fileName} (${entry.message})`).join('; ')}`,
    );
  }

  if (failures.length > 0) {
    console.warn(
      '[KnowledgeFileUploadService] partial upload failures',
      failures,
    );
  }

  return results;
}
