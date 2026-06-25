import type { KnowledgeSpaceMemberRole } from '@sdkwork/knowledgebase-app-sdk';
import { getKnowledgebaseAppSdkClient } from 'sdkwork-knowledgebase-pc-core';

export interface KnowledgeSpaceMemberUi {
  name: string;
  email: string;
  role: 'admin' | 'editor' | 'viewer';
  avatar: string;
  inherited?: boolean;
}

function toUiRole(role: KnowledgeSpaceMemberRole): KnowledgeSpaceMemberUi['role'] {
  if (role === 'owner') {
    return 'admin';
  }
  if (role === 'writer') {
    return 'editor';
  }
  return 'viewer';
}

function toApiRole(role: KnowledgeSpaceMemberUi['role']): KnowledgeSpaceMemberRole {
  if (role === 'admin') {
    return 'owner';
  }
  if (role === 'editor') {
    return 'writer';
  }
  return 'reader';
}

function displayNameFromEmail(email: string): string {
  const localPart = email.split('@')[0] ?? email;
  return localPart.charAt(0).toUpperCase() + localPart.slice(1);
}

function avatarFromEmail(email: string): string {
  const name = displayNameFromEmail(email);
  return `https://ui-avatars.com/api/?name=${encodeURIComponent(name)}&background=random`;
}

/**
 * Frontend service facade for knowledge space member management.
 */
export class KnowledgeSpaceMembersService {
  static loadMembers(spaceId: number): Promise<KnowledgeSpaceMemberUi[]> {
    return loadKnowledgeSpaceMembers(spaceId);
  }

  static syncMembers(
    spaceId: number,
    desired: KnowledgeSpaceMemberUi[],
    previous: KnowledgeSpaceMemberUi[],
  ): Promise<void> {
    return syncKnowledgeSpaceMembers(spaceId, desired, previous);
  }
}

export async function loadKnowledgeSpaceMembers(
  spaceId: number,
): Promise<KnowledgeSpaceMemberUi[]> {
  const spaceKey = String(spaceId);
  const list = await getKnowledgebaseAppSdkClient().client.knowledge.spaces.members.list(spaceKey);
  return list.members
    .filter((member) => member.subjectType === 'user')
    .map((member) => ({
      name: displayNameFromEmail(member.subjectId),
      email: member.subjectId,
      role: toUiRole(member.role),
      avatar: avatarFromEmail(member.subjectId),
      inherited: member.inherited,
    }));
}

export async function syncKnowledgeSpaceMembers(
  spaceId: number,
  desired: KnowledgeSpaceMemberUi[],
  previous: KnowledgeSpaceMemberUi[],
): Promise<void> {
  const client = getKnowledgebaseAppSdkClient().client;
  const spaceKey = String(spaceId);
  const previousByEmail = new Map(previous.map((member) => [member.email.toLowerCase(), member]));
  const desiredByEmail = new Map(desired.map((member) => [member.email.toLowerCase(), member]));

  for (const [email, member] of desiredByEmail) {
    const existing = previousByEmail.get(email);
    if (!existing || existing.role !== member.role) {
      await client.knowledge.spaces.members.grant(spaceKey, {
        subjectType: 'user',
        subjectId: member.email,
        role: toApiRole(member.role),
      });
    }
  }

  for (const [email] of previousByEmail) {
    if (!desiredByEmail.has(email)) {
      await client.knowledge.spaces.members.revoke(spaceKey, {
        subjectType: 'user',
        subjectId: email,
      });
    }
  }
}
