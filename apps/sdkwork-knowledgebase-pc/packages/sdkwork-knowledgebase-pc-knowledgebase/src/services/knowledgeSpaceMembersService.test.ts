import { describe, expect, it } from 'vitest';

import { buildPartialMemberSyncPayload } from './knowledgeSpaceMembersService';

const member = (email: string, role: 'admin' | 'editor' | 'viewer' = 'viewer') => ({
  name: email.split('@')[0],
  email,
  role,
  avatar: '',
});

describe('buildPartialMemberSyncPayload', () => {
  it('preserves baseline members that were never loaded in the UI', () => {
    const baseline = [member('alice@company.com'), member('bob@company.com'), member('carol@company.com')];
    const ui = [member('alice@company.com', 'editor')];
    const loaded = new Set(['alice@company.com']);

    const { desired, previous } = buildPartialMemberSyncPayload(ui, baseline, loaded);

    expect(previous).toEqual(baseline);
    expect(desired.map((entry) => entry.email).sort()).toEqual([
      'alice@company.com',
      'bob@company.com',
      'carol@company.com',
    ]);
    expect(desired.find((entry) => entry.email === 'alice@company.com')?.role).toBe('editor');
  });

  it('revokes only loaded members removed from the UI', () => {
    const baseline = [member('alice@company.com'), member('bob@company.com')];
    const ui = [member('alice@company.com')];
    const loaded = new Set(['alice@company.com', 'bob@company.com']);

    const { desired } = buildPartialMemberSyncPayload(ui, baseline, loaded);

    expect(desired.map((entry) => entry.email)).toEqual(['alice@company.com']);
  });

  it('includes newly added UI members', () => {
    const baseline = [member('alice@company.com')];
    const ui = [member('alice@company.com'), member('new@company.com', 'admin')];
    const loaded = new Set(['alice@company.com']);

    const { desired } = buildPartialMemberSyncPayload(ui, baseline, loaded);

    expect(desired).toHaveLength(2);
    expect(desired.find((entry) => entry.email === 'new@company.com')?.role).toBe('admin');
  });
});
