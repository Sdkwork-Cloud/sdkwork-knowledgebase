import type { KnowledgeEngineProviderBindingPage } from './knowledge-engine-provider-binding-page';

export interface SpacesProviderBindingsListResponse {
  code: 0;
  data: unknown & KnowledgeEngineProviderBindingPage;
  /** Server-owned request correlation id. */
  traceId: string;
}
