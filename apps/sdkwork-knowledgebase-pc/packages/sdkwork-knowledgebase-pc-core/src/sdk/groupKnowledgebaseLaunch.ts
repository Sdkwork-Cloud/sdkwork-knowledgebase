import { uuid } from '@sdkwork/utils';

import type { KnowledgebaseAppSdkClient } from './knowledgebaseAppSdkClient';
import { isValidGroupKnowledgebaseLaunchTicket } from '../runtime/groupKnowledgebaseLaunchTicket';

type GroupKnowledgebaseLaunchConsumeOperation = KnowledgebaseAppSdkClient['client']['knowledge']['groupLaunches']['consume'];
type GroupKnowledgebaseLaunchConsumeArguments = Parameters<GroupKnowledgebaseLaunchConsumeOperation>;
type GroupKnowledgebaseLaunchConsumeRequest = GroupKnowledgebaseLaunchConsumeArguments[0];
type GroupKnowledgebaseLaunchConsumeParams = GroupKnowledgebaseLaunchConsumeArguments[1];

const GROUP_KNOWLEDGEBASE_LAUNCH_IDEMPOTENCY_KEY_PREFIX = 'pc-knowledgebase-group-launch-';

export interface GroupKnowledgebaseLaunchSdk {
  readonly client: {
    readonly knowledge: {
      readonly groupLaunches: {
        consume: GroupKnowledgebaseLaunchConsumeOperation;
      };
    };
  };
}

export interface GroupKnowledgebaseLaunchTarget {
  conversationId: string;
  groupName: string;
  lifecycleState: 'active';
  spaceId: string;
  spaceUuid: string;
}

export type GroupKnowledgebaseLaunchConsumeResult =
  | { kind: 'ready'; target: GroupKnowledgebaseLaunchTarget }
  | { kind: 'unavailable' }
  | { kind: 'failed' };

export function createGroupKnowledgebaseLaunchConsumeIdempotencyKey(): string | null {
  try {
    return `${GROUP_KNOWLEDGEBASE_LAUNCH_IDEMPOTENCY_KEY_PREFIX}${uuid()}`;
  } catch {
    return null;
  }
}

export async function consumeGroupKnowledgebaseLaunchTicket(
  appSdk: GroupKnowledgebaseLaunchSdk,
  ticket: string,
): Promise<GroupKnowledgebaseLaunchConsumeResult> {
  if (!isValidGroupKnowledgebaseLaunchTicket(ticket)) {
    return { kind: 'failed' };
  }

  const idempotencyKey = createGroupKnowledgebaseLaunchConsumeIdempotencyKey();
  if (!idempotencyKey) {
    return { kind: 'failed' };
  }

  try {
    const request: GroupKnowledgebaseLaunchConsumeRequest = { ticket };
    const params: GroupKnowledgebaseLaunchConsumeParams = { idempotencyKey };
    const target = await appSdk.client.knowledge.groupLaunches.consume(request, params);
    if (
      target.lifecycleState !== 'active'
      || typeof target.conversationId !== 'string'
      || typeof target.groupName !== 'string'
      || typeof target.spaceId !== 'string'
      || typeof target.spaceUuid !== 'string'
    ) {
      return { kind: 'failed' };
    }

    const conversationId = target.conversationId.trim();
    const groupName = target.groupName.trim();
    const spaceId = target.spaceId.trim();
    const spaceUuid = target.spaceUuid.trim();
    if (!conversationId || !groupName || !spaceId || !spaceUuid) {
      return { kind: 'failed' };
    }

    return {
      kind: 'ready',
      target: {
        conversationId,
        groupName,
        lifecycleState: 'active',
        spaceId,
        spaceUuid,
      },
    };
  } catch {
    return { kind: 'failed' };
  }
}
