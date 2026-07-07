import { HttpClient, createHttpClient } from './http/client';
import type { SdkworkBackendConfig } from './types/common';
import type { AuthTokenManager } from '@sdkwork/sdk-common';

import { KnowledgeApi, createKnowledgeApi } from './api/knowledge';

export class SdkworkKnowledgebaseBackendClient {
  private httpClient: HttpClient;

  public readonly knowledge: KnowledgeApi;

  constructor(config: SdkworkBackendConfig) {
    this.httpClient = createHttpClient(config);
    this.knowledge = createKnowledgeApi(this.httpClient);
  }
  setAuthToken(token: string): this {
    this.httpClient.setAuthToken(token);
    return this;
  }

  setAccessToken(token: string): this {
    this.httpClient.setAccessToken(token);
    return this;
  }

  setTokenManager(manager: AuthTokenManager): this {
    this.httpClient.setTokenManager(manager);
    return this;
  }

  get http(): HttpClient {
    return this.httpClient;
  }
}

export function createClient(config: SdkworkBackendConfig): SdkworkKnowledgebaseBackendClient {
  return new SdkworkKnowledgebaseBackendClient(config);
}

export default SdkworkKnowledgebaseBackendClient;
