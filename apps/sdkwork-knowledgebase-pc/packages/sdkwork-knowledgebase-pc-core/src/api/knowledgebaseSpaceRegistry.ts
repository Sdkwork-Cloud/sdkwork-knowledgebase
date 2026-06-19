import type { SessionStore } from '../session/sessionStore';

export type KnowledgebaseSpaceKbType = 'team' | 'personal' | 'public';

export interface RegisteredKnowledgebaseSpace {
  spaceId: number;
  kbType: KnowledgebaseSpaceKbType;
  icon?: string;
  createdAt: string;
}

const REGISTRY_KEY_PREFIX = 'sdkwork.knowledgebase.spaces.v1';

let sessionStoreRef: SessionStore | null = null;

export function bindKnowledgebaseSessionStore(session: SessionStore): void {
  sessionStoreRef = session;
}

export function getKnowledgebaseTenantId(): string | undefined {
  return sessionStoreRef?.getSnapshot().context?.tenantId;
}

function registryStorageKey(tenantId: string): string {
  return `${REGISTRY_KEY_PREFIX}.${tenantId}`;
}

export function readRegisteredSpaces(tenantId: string): RegisteredKnowledgebaseSpace[] {
  if (typeof window === 'undefined') {
    return [];
  }

  try {
    const raw = window.localStorage.getItem(registryStorageKey(tenantId));
    if (!raw) {
      return [];
    }
    const parsed = JSON.parse(raw) as RegisteredKnowledgebaseSpace[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

export function writeRegisteredSpaces(
  tenantId: string,
  spaces: RegisteredKnowledgebaseSpace[],
): void {
  if (typeof window === 'undefined') {
    return;
  }

  window.localStorage.setItem(registryStorageKey(tenantId), JSON.stringify(spaces));
}

export function upsertRegisteredSpace(
  tenantId: string,
  entry: RegisteredKnowledgebaseSpace,
): RegisteredKnowledgebaseSpace[] {
  const spaces = readRegisteredSpaces(tenantId);
  const next = spaces.filter((space) => space.spaceId !== entry.spaceId);
  next.push(entry);
  writeRegisteredSpaces(tenantId, next);
  return next;
}

export function removeRegisteredSpace(tenantId: string, spaceId: number): RegisteredKnowledgebaseSpace[] {
  const next = readRegisteredSpaces(tenantId).filter((space) => space.spaceId !== spaceId);
  writeRegisteredSpaces(tenantId, next);
  return next;
}

export function updateRegisteredSpace(
  tenantId: string,
  spaceId: number,
  patch: Partial<Pick<RegisteredKnowledgebaseSpace, 'kbType' | 'icon'>>,
): RegisteredKnowledgebaseSpace[] {
  const next = readRegisteredSpaces(tenantId).map((space) =>
    space.spaceId === spaceId ? { ...space, ...patch } : space,
  );
  writeRegisteredSpaces(tenantId, next);
  return next;
}
