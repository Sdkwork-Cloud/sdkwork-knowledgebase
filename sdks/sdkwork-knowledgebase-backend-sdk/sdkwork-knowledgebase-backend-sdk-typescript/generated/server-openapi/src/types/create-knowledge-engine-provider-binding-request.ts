export interface CreateKnowledgeEngineProviderBindingRequest {
  implementationId: string;
  remoteResourceType: string;
  remoteResourceId: string;
  credentialReferenceId?: string | null;
}
