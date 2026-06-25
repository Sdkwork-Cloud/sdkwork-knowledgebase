import type { SessionSnapshot } from 'sdkwork-knowledgebase-pc-core';
import { KnowledgebaseErrorCodes, throwKnowledgebaseError } from 'sdkwork-knowledgebase-pc-core';
import type { SdkworkAppClient } from '@sdkwork/knowledgebase-app-sdk';
import type { SdkworkDriveAppClient } from '@sdkwork/drive-app-sdk';

export interface KnowledgebasePcSdkPorts {
  getKnowledgebaseClient: () => SdkworkAppClient;
  getDriveClient: () => SdkworkDriveAppClient;
  readHostSession: () => SessionSnapshot | null;
  subscribeHostSession?: (listener: () => void) => () => void;
}

let sdkPorts: KnowledgebasePcSdkPorts | null = null;

export function configureKnowledgebasePcSdkPorts(ports: KnowledgebasePcSdkPorts): void {
  sdkPorts = ports;
}

export function getKnowledgebasePcSdkPorts(): KnowledgebasePcSdkPorts {
  if (!sdkPorts) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.SDK_PORTS_MISSING);
  }
  return sdkPorts;
}

export function getConfiguredKnowledgebaseAppSdkClient(): SdkworkAppClient {
  return getKnowledgebasePcSdkPorts().getKnowledgebaseClient();
}

export function getConfiguredKnowledgebaseDriveAppSdkClient(): SdkworkDriveAppClient {
  return getKnowledgebasePcSdkPorts().getDriveClient();
}
