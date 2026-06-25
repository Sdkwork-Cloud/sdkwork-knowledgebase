export type FileType = 'doc' | 'mp3' | 'jpg' | 'mp4' | 'fig' | 'folder';

export interface DocNode {
  id: string;
  name: string;
  type: FileType;
  children?: DocNode[];
  updatedAt?: string;
  author?: string;
  visibility?: 'public' | 'team' | 'private';
  content?: string;
}

export interface KnowledgeBase {
  id: string;
  name: string;
  category: 'team' | 'personal';
  icon?: string;
  docs: DocNode[];
}
