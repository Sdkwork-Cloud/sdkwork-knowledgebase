import { describe, expect, it } from 'vitest';
import {
  extractSdkWorkListItems,
  extractSdkWorkMemberItems,
} from './knowledgebaseBackendAdminService';
import { canAccessKnowledgebaseAdminConsole } from './knowledgebaseBackendApiRegistry';

describe('knowledgebase backend admin service helpers', () => {
  it('extracts only object rows from SDKWork list payloads', () => {
    expect(extractSdkWorkListItems({
      items: [
        { id: 'space-1' },
        null,
        ['not-a-row'],
        'not-a-row',
        { id: 'space-2' },
      ],
    })).toEqual([
      { id: 'space-1' },
      { id: 'space-2' },
    ]);
  });

  it('supports both items and members result shapes for member lists', () => {
    expect(extractSdkWorkMemberItems({
      members: [
        { subjectId: 'user-1' },
        12,
        { subjectId: 'user-2' },
      ],
    })).toEqual([
      { subjectId: 'user-1' },
      { subjectId: 'user-2' },
    ]);
  });

  it('grants admin console access only to knowledge admin scopes', () => {
    expect(canAccessKnowledgebaseAdminConsole(['knowledge.read'])).toBe(false);
    expect(canAccessKnowledgebaseAdminConsole(['knowledge.platform.manage'])).toBe(true);
    expect(canAccessKnowledgebaseAdminConsole(['knowledge.*'])).toBe(true);
  });
});
