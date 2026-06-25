import { DocumentService, type KnowledgeBase as KnowledgebaseRecord } from '../../sdkwork-knowledgebase-pc-knowledgebase/src/services/document';

export interface KnowledgeSelectionItem {
  id: string;
  name: string;
  description: string;
  logo: string;
  count: number;
  updatedAt: number;
  type: 'personal' | 'team';
}

function mapKnowledgeBase(base: KnowledgebaseRecord): KnowledgeSelectionItem {
  return {
    id: base.id,
    name: base.title,
    description: '',
    logo: base.icon || base.avatar || 'KB',
    count: 0,
    updatedAt: Date.now(),
    type: base.type === 'personal' ? 'personal' : 'team',
  };
}

export type KnowledgeBase = KnowledgeSelectionItem;

export const knowledgeSelectionService = {
  async getBases(): Promise<KnowledgeSelectionItem[]> {
    const grouped = await DocumentService.getKnowledgeBases();
    return [
      ...grouped.team.map(mapKnowledgeBase),
      ...grouped.personal.map(mapKnowledgeBase),
      ...grouped.public.map((base) => ({
        ...mapKnowledgeBase(base),
        type: 'team' as const,
      })),
    ];
  },
};
