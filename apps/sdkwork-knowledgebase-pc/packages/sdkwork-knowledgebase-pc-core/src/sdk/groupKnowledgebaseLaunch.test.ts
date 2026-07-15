import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  createGroupKnowledgebaseLaunchConsumeIdempotencyKey,
  consumeGroupKnowledgebaseLaunchTicket,
  type GroupKnowledgebaseLaunchSdk,
} from './groupKnowledgebaseLaunch';

const VALID_TICKET = `gklt_${'a'.repeat(43)}`;

type GroupKnowledgebaseLaunchConsumeOperation = GroupKnowledgebaseLaunchSdk['client']['knowledge']['groupLaunches']['consume'];
type GroupKnowledgebaseLaunchConsumeRequest = Parameters<GroupKnowledgebaseLaunchConsumeOperation>[0];
type GroupKnowledgebaseLaunchConsumeParams = Parameters<GroupKnowledgebaseLaunchConsumeOperation>[1];
type GroupKnowledgebaseLaunchTarget = Awaited<ReturnType<GroupKnowledgebaseLaunchConsumeOperation>>;

function activeTarget(): GroupKnowledgebaseLaunchTarget {
  return {
    conversationId: 'conversation-1',
    groupName: 'Group one',
    lifecycleState: 'active',
    spaceId: '1001',
    spaceUuid: 'space-uuid-1',
  };
}

function createSdk(
  consume: GroupKnowledgebaseLaunchConsumeOperation,
): GroupKnowledgebaseLaunchSdk {
  return {
    client: {
      knowledge: {
        groupLaunches: { consume },
      },
    },
  };
}

describe('consumeGroupKnowledgebaseLaunchTicket', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('uses one Web Crypto idempotency key through the generated consume operation', async () => {
    const requests: GroupKnowledgebaseLaunchConsumeRequest[] = [];
    const params: GroupKnowledgebaseLaunchConsumeParams[] = [];
    const randomUUID = vi.fn(() => '46ce80da-b999-4264-8489-86d4cc2506f9');
    vi.stubGlobal('crypto', {
      getRandomValues: vi.fn(),
      randomUUID,
    });
    const result = await consumeGroupKnowledgebaseLaunchTicket(
      createSdk(async (request, requestParams) => {
        requests.push(request);
        params.push(requestParams);
        return activeTarget();
      }),
      VALID_TICKET,
    );

    expect(requests).toEqual([{ ticket: VALID_TICKET }]);
    expect(params).toEqual([{
      idempotencyKey: 'pc-knowledgebase-group-launch-46ce80da-b999-4264-8489-86d4cc2506f9',
    }]);
    expect(randomUUID).toHaveBeenCalledTimes(1);
    expect(result).toEqual({
      kind: 'ready',
      target: activeTarget(),
    });
  });

  it('fails closed for non-active or rejected ticket consumption', async () => {
    const provisioningResult = await consumeGroupKnowledgebaseLaunchTicket(
      createSdk(async () => ({
        ...activeTarget(),
        lifecycleState: 'provisioning',
      })),
      VALID_TICKET,
    );
    const rejectedResult = await consumeGroupKnowledgebaseLaunchTicket(
      createSdk(async () => {
        throw new Error('rejected ticket');
      }),
      VALID_TICKET,
    );

    expect(provisioningResult).toEqual({ kind: 'failed' });
    expect(rejectedResult).toEqual({ kind: 'failed' });
  });

  it('does not call the SDK for an invalid ticket', async () => {
    let calls = 0;
    const result = await consumeGroupKnowledgebaseLaunchTicket(
      createSdk(async () => {
        calls += 1;
        return activeTarget();
      }),
      'not-a-launch-ticket',
    );

    expect(result).toEqual({ kind: 'failed' });
    expect(calls).toBe(0);
  });

  it('fails closed before calling the SDK when Web Crypto is unavailable', async () => {
    vi.stubGlobal('crypto', undefined);
    let calls = 0;

    const result = await consumeGroupKnowledgebaseLaunchTicket(
      createSdk(async () => {
        calls += 1;
        return activeTarget();
      }),
      VALID_TICKET,
    );

    expect(result).toEqual({ kind: 'failed' });
    expect(calls).toBe(0);
  });

  it('uses the Web Crypto random-value fallback when randomUUID is unavailable', () => {
    const getRandomValues = vi.fn((bytes: Uint8Array) => {
      bytes.set([0x46, 0xce, 0x80, 0xda, 0xb9, 0x99, 0x26, 0x64, 0x84, 0x89, 0x86, 0xd4, 0xcc, 0x25, 0x06, 0xf9]);
      return bytes;
    });
    vi.stubGlobal('crypto', { getRandomValues });

    expect(createGroupKnowledgebaseLaunchConsumeIdempotencyKey()).toBe(
      'pc-knowledgebase-group-launch-46ce80da-b999-4664-8489-86d4cc2506f9',
    );
    expect(getRandomValues).toHaveBeenCalledTimes(1);
  });
});
