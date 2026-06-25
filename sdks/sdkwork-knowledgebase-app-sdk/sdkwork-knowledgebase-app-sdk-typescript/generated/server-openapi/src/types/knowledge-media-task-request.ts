import type { KnowledgeMediaTaskType } from './knowledge-media-task-type';

export interface KnowledgeMediaTaskRequest {
  spaceId: number;
  taskType: KnowledgeMediaTaskType;
  prompt?: string | null;
  aspectMode?: string | null;
  styleMode?: string | null;
  sourceUrl?: string | null;
  documentId?: number | null;
}
