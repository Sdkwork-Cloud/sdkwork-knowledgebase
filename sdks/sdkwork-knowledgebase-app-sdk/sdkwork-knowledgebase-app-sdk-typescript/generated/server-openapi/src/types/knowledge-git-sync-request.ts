export interface KnowledgeGitSyncRequest {
  spaceId: number;
  repoUrl: string;
  branch?: string | null;
  commitMessage: string;
  idempotencyKey: string;
  gitAccessToken?: string | null;
}
