import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  captureGroupKnowledgebaseLaunchTicket,
  GROUP_KNOWLEDGEBASE_LAUNCH_PATH,
  sanitizeGroupKnowledgebaseLaunchAuthLocation,
  takePendingGroupKnowledgebaseLaunchTicket,
} from './groupKnowledgebaseLaunchHandoff';
import { isValidGroupKnowledgebaseLaunchTicket } from './groupKnowledgebaseLaunchTicket';
import { buildKnowledgebaseAuthLoginRedirect } from '../auth/authGate';

const VALID_TICKET = `gklt_${'a'.repeat(43)}`;

afterEach(() => {
  takePendingGroupKnowledgebaseLaunchTicket();
  vi.unstubAllGlobals();
});

describe('group knowledgebase launch handoff', () => {
  it('removes the fragment while preserving a non-root browser mount path', () => {
    const replaceState = vi.fn();
    vi.stubGlobal('window', {
      history: {
        replaceState,
        state: { source: 'test' },
      },
      location: {
        pathname: '/apps/knowledgebase/group-launch',
        search: '?source=im',
      },
    });

    expect(captureGroupKnowledgebaseLaunchTicket({
      hash: `#ticket=${VALID_TICKET}`,
      pathname: GROUP_KNOWLEDGEBASE_LAUNCH_PATH,
      search: '?source=im',
    })).toBe(true);

    expect(replaceState).toHaveBeenCalledWith(
      { source: 'test' },
      '',
      '/apps/knowledgebase/group-launch?source=im',
    );
    expect(takePendingGroupKnowledgebaseLaunchTicket()).toBe(VALID_TICKET);
    expect(takePendingGroupKnowledgebaseLaunchTicket()).toBeNull();
  });

  it('scrubs malformed or ambiguous fragments before auth navigation', () => {
    const replaceState = vi.fn();
    vi.stubGlobal('window', {
      history: {
        replaceState,
        state: { source: 'test' },
      },
      location: {
        pathname: '/apps/knowledgebase/group-launch',
        search: '',
      },
    });

    expect(isValidGroupKnowledgebaseLaunchTicket(VALID_TICKET)).toBe(true);
    expect(isValidGroupKnowledgebaseLaunchTicket('gklt_short')).toBe(false);
    const unsafeLocation = {
      hash: `#ticket=${VALID_TICKET}&other=value`,
      pathname: GROUP_KNOWLEDGEBASE_LAUNCH_PATH,
    };
    expect(captureGroupKnowledgebaseLaunchTicket(unsafeLocation)).toBe(false);
    expect(replaceState).toHaveBeenCalledWith(
      { source: 'test' },
      '',
      '/apps/knowledgebase/group-launch',
    );
    expect(takePendingGroupKnowledgebaseLaunchTicket()).toBeNull();

    const authLocation = sanitizeGroupKnowledgebaseLaunchAuthLocation(unsafeLocation);
    expect(authLocation.hash).toBe('');
    const redirect = buildKnowledgebaseAuthLoginRedirect(authLocation);
    expect(redirect).toBe('/auth/login?redirect=%2Fgroup-launch');
    expect(redirect).not.toContain(VALID_TICKET);
  });
});
