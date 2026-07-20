export interface CreateKnowledgeEngineProviderMigrationOperationRequest {
  sourceBindingId: string;
  targetBindingId: string;
  idempotencyKey: string;
  expectedSourceVersion: string;
  expectedTargetVersion: string;
  observationSeconds: number;
}
