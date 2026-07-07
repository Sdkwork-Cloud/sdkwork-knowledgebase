export interface KnowledgeGitSyncRequest {
  spaceId: string;
  repoUrl: string;
  branch?: string | null;
  commitMessage: string;
  idempotencyKey: string;
  gitAccessToken?: string | null;
}
