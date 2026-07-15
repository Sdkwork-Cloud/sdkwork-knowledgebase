import { describe, expect, it } from 'vitest';

import { parseGroupKnowledgebaseLaunchRoute } from './groupKnowledgebaseLaunchHost';

const VALID_TICKET = `gklt_${'a'.repeat(43)}`;

describe('group knowledgebase desktop launch route', () => {
  it('accepts only the exact route with a server-shaped ticket', () => {
    expect(parseGroupKnowledgebaseLaunchRoute(
      `/group-launch#ticket=${VALID_TICKET}`,
    )).toBe(`/group-launch#ticket=${VALID_TICKET}`);
  });

  it('rejects malformed tickets and route extensions', () => {
    for (const route of [
      '/group-launch#ticket=gklt_short',
      `/group-launch#ticket=${VALID_TICKET}&other=value`,
      `/group-launch#ticket=${VALID_TICKET}#again`,
      `/other#ticket=${VALID_TICKET}`,
    ]) {
      expect(parseGroupKnowledgebaseLaunchRoute(route)).toBeNull();
    }
  });
});
