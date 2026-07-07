import type { DriveNode, DriveNodeProperty } from 'sdkwork-knowledgebase-pc-core';

import { normalizeSdkWorkListPage, type NormalizedSdkWorkListPage } from './sdkWorkListPage';

export interface DriveDownloadUrlResponse {
  downloadUrl?: string | null;
  signedSourceUrl?: string | null;
}

export function readDriveNode(value: unknown): DriveNode {
  const record = requireRecord(value, 'Drive node response');
  requireString(record.id, 'Drive node id');
  requireString(record.spaceId, 'Drive node spaceId');
  requireString(record.nodeName, 'Drive node nodeName');
  return record as unknown as DriveNode;
}

export function normalizeDriveNodePage(page: unknown): NormalizedSdkWorkListPage<DriveNode> {
  const normalized = normalizeSdkWorkListPage<unknown>(page);
  return {
    ...normalized,
    items: normalized.items.map(readDriveNode),
  };
}

export function normalizeDriveNodePropertyPage(
  page: unknown,
): NormalizedSdkWorkListPage<DriveNodeProperty> {
  const normalized = normalizeSdkWorkListPage<unknown>(page);
  return {
    ...normalized,
    items: normalized.items.map(readDriveNodeProperty),
  };
}

export function readDriveDownloadUrlResponse(value: unknown): DriveDownloadUrlResponse {
  const record = requireRecord(value, 'Drive download URL response');
  return {
    downloadUrl: typeof record.downloadUrl === 'string' ? record.downloadUrl : null,
    signedSourceUrl: typeof record.signedSourceUrl === 'string' ? record.signedSourceUrl : null,
  };
}

function readDriveNodeProperty(value: unknown): DriveNodeProperty {
  const record = requireRecord(value, 'Drive node property response');
  requireString(record.propertyKey, 'Drive node property key');
  requireString(record.propertyValue, 'Drive node property value');
  return record as unknown as DriveNodeProperty;
}

function requireRecord(value: unknown, label: string): Record<string, unknown> {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    throw new Error(`${label} must be an object.`);
  }
  return value as Record<string, unknown>;
}

function requireString(value: unknown, label: string): void {
  if (typeof value !== 'string' || value.length === 0) {
    throw new Error(`${label} must be a non-empty string.`);
  }
}
