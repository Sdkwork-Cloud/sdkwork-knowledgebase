import { HttpClient, createHttpClient } from './http/client';
import type { SdkworkCustomConfig } from './types/common';

import { KnowledgebaseInternalWikiApi, createKnowledgebaseInternalWikiApi } from './api/knowledgebase-internal-wiki';

export class SdkworkKnowledgebaseInternalClient {
  private httpClient: HttpClient;

  public readonly knowledgebaseInternalWiki: KnowledgebaseInternalWikiApi;

  constructor(config: SdkworkCustomConfig) {
    this.httpClient = createHttpClient(config);
    this.knowledgebaseInternalWiki = createKnowledgebaseInternalWikiApi(this.httpClient);
  }

  setApiKey(apiKey: string): this {
    this.httpClient.setApiKey(apiKey);
    return this;
  }
  get http(): HttpClient {
    return this.httpClient;
  }
}

export function createClient(config: SdkworkCustomConfig): SdkworkKnowledgebaseInternalClient {
  return new SdkworkKnowledgebaseInternalClient(config);
}

export default SdkworkKnowledgebaseInternalClient;
