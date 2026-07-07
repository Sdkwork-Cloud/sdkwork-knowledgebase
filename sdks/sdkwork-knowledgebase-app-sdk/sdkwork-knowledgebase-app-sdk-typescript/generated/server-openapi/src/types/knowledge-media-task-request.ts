import type { KnowledgeMediaTaskType } from './knowledge-media-task-type';

export interface KnowledgeMediaTaskRequest {
  spaceId: string;
  taskType: KnowledgeMediaTaskType;
  prompt?: string | null;
  aspectMode?: string | null;
  styleMode?: string | null;
  sourceUrl?: string | null;
  documentId?: string | null;
}
