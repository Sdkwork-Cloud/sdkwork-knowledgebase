import { HttpClient, createHttpClient } from './http/client';
import type { SdkworkCustomConfig } from './types/common';

import { KnowledgeApi, createKnowledgeApi } from './api/knowledge';

export class SdkworkKnowledgebaseOpenClient {
  private httpClient: HttpClient;

  public readonly knowledge: KnowledgeApi;

  constructor(config: SdkworkCustomConfig) {
    this.httpClient = createHttpClient(config);
    this.knowledge = createKnowledgeApi(this.httpClient);
  }

  setApiKey(apiKey: string): this {
    this.httpClient.setApiKey(apiKey);
    return this;
  }
  get http(): HttpClient {
    return this.httpClient;
  }
}

export function createClient(config: SdkworkCustomConfig): SdkworkKnowledgebaseOpenClient {
  return new SdkworkKnowledgebaseOpenClient(config);
}

export default SdkworkKnowledgebaseOpenClient;
