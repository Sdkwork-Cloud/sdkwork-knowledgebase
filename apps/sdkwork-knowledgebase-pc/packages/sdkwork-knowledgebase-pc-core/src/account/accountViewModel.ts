import type { SessionSnapshot } from '../session/sessionStore';
import { isBlank, trim } from '@sdkwork/sdkwork-knowledgebase-pc-commons/stringUtils';

export interface KnowledgebaseAccountViewModel {
  id: string;
  displayName: string;
  email?: string;
  avatarUrl?: string;
  initials: string;
  tenantId?: string;
  organizationId?: string;
  sessionId?: string;
  environmentLabel: string;
  authLevel: string;
}

export function createKnowledgebaseAccountViewModel(
  session: SessionSnapshot,
): KnowledgebaseAccountViewModel {
  const userId = session.user?.id ?? session.context?.userId ?? 'knowledgebase-user';
  const displayName = session.user?.displayName?.trim() || 'SDKWork Knowledgebase User';
  const environment = session.context?.environment?.trim();
  const iamDeploymentMode = session.context?.iamDeploymentMode?.trim();

  return {
    id: userId,
    displayName,
    email: session.user?.email,
    avatarUrl: session.user?.avatarUrl,
    initials: buildInitials(displayName, userId),
    tenantId: session.context?.tenantId,
    organizationId: session.context?.organizationId,
    sessionId: session.sessionId ?? session.context?.sessionId,
    environmentLabel:
      environment && iamDeploymentMode
        ? `${environment} / ${iamDeploymentMode}`
        : environment || iamDeploymentMode || 'standard',
    authLevel: session.context?.authLevel ?? 'standard',
  };
}

function buildInitials(displayName: string, fallbackId: string): string {
  const normalized = displayName.trim();
  if (normalized) {
    const asciiWords = normalized.match(/[A-Za-z0-9]+/g);
    if (asciiWords?.length) {
      return asciiWords
        .slice(0, 2)
        .map((word) => word.charAt(0).toUpperCase())
        .join('');
    }
    return Array.from(normalized).slice(0, 2).join('');
  }

  return fallbackId.charAt(0).toUpperCase() || 'K';
}
