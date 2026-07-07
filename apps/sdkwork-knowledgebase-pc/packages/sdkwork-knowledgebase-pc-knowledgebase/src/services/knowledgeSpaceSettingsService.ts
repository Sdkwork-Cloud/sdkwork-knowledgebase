import type { KnowledgeAccessLevel, KnowledgeSpaceContextBinding } from 'sdkwork-knowledgebase-pc-core';
import { isBlank } from '@sdkwork/utils';
import {
  getKnowledgebaseAppSdkClient,
  readRegisteredSpaces,
  requireKnowledgebaseTenantId,
} from 'sdkwork-knowledgebase-pc-core';

import type { KnowledgeBase } from './document';
import { normalizeSdkWorkListPage } from './sdkWorkListPage';

const SPACE_AGENT_PROFILE_PREFIX = 'sdkwork.knowledgebase.spaceAgentProfile.v1';
const DEFAULT_MODEL_PROVIDER_ID = 'provider.model.knowledgebase-contract';
const DEFAULT_AGENT_IMPLEMENTATION_ID = 'plugin.intelligence.knowledgebase-contract';
const GUEST_CONTEXT_BINDING_ID = 'pc-knowledgebase-guest-link';
const GUEST_CONTEXT_BINDING_NAME = 'PC Guest Link';

interface ParsedModelParameters {
  temperature?: number;
  maxTokens?: number;
  uiProvider?: string;
  uiModelName?: string;
}

function readSpaceAgentProfileCache(tenantId: string, spaceId: string): string | null {
  if (typeof window === 'undefined') {
    return null;
  }
  return window.localStorage.getItem(`${SPACE_AGENT_PROFILE_PREFIX}.${tenantId}.${spaceId}`);
}

function writeSpaceAgentProfileCache(tenantId: string, spaceId: string, profileId: string): void {
  if (typeof window === 'undefined') {
    return;
  }
  window.localStorage.setItem(`${SPACE_AGENT_PROFILE_PREFIX}.${tenantId}.${spaceId}`, profileId);
}

function parseModelParameters(raw?: string | null): ParsedModelParameters {
  if (!raw) {
    return {};
  }
  try {
    return JSON.parse(raw) as ParsedModelParameters;
  } catch {
    return {};
  }
}

function buildModelParameters(
  updates: Partial<KnowledgeBase>,
  existing?: ParsedModelParameters,
): string {
  return JSON.stringify({
    temperature: updates.temperature ?? existing?.temperature,
    maxTokens: updates.maxTokens ?? existing?.maxTokens,
    uiProvider: updates.provider ?? existing?.uiProvider,
    uiModelName: updates.modelName ?? existing?.uiModelName,
  });
}

function toBackendModelId(provider?: string, modelName?: string): string {
  const trimmed = modelName?.trim();
  if (trimmed) {
    return trimmed;
  }
  if (provider === 'DeepSeek') {
    return 'deepseek-chat';
  }
  if (provider === 'OpenAI') {
    return 'gpt-4o-mini';
  }
  return 'contract';
}

function mapAccessLevel(
  publicPermission?: KnowledgeBase['publicPermission'],
): KnowledgeAccessLevel {
  if (publicPermission === 'write' || publicPermission === 'admin') {
    return 'writer';
  }
  return 'reader';
}

function resolveGuestContextId(): string {
  const tenantId = requireKnowledgebaseTenantId();
  return `tenant-${tenantId}`;
}

export async function ensureSpaceAgentProfile(spaceId: string): Promise<string> {
  const tenantId = requireKnowledgebaseTenantId();

  const cached = readSpaceAgentProfileCache(tenantId, spaceId);
  const client = getKnowledgebaseAppSdkClient().client;
  if (cached && !isBlank(cached)) {
    try {
      await client.knowledge.agentProfiles.retrieve(cached);
      return cached;
    } catch {
      // Stale cache entry; recreate profile.
    }
  }

  const profile = await client.knowledge.agentProfiles.create({
    name: `Knowledgebase Space ${spaceId}`,
    description: 'PC knowledge space assistant profile',
    systemInstruction:
      'You are a helpful knowledge assistant for this knowledge space. Answer accurately and cite sources when available.',
    modelProviderId: DEFAULT_MODEL_PROVIDER_ID,
    modelId: 'contract',
    modelParameters: JSON.stringify({
      temperature: 0.7,
      maxTokens: 2048,
      uiProvider: 'Google',
      uiModelName: 'gemini-3.5-flash',
    }),
    agentImplementationId: DEFAULT_AGENT_IMPLEMENTATION_ID,
    knowledgeMode: 'okf_bundle',
    status: 'active',
  });

  const profileId = String(profile.profileId ?? '').trim();
  if (isBlank(profileId)) {
    throw new Error('Agent profile create did not return profileId');
  }

  await client.knowledge.agentProfiles.bindings.bindings(profileId, {
    profileId,
    spaceId: String(spaceId),
    priority: 0,
    enabled: true,
  });

  writeSpaceAgentProfileCache(tenantId, spaceId, profileId);
  return profileId;
}

export async function loadKnowledgeSpaceModelSettings(
  spaceId: string,
): Promise<Partial<KnowledgeBase>> {
  const profileId = await ensureSpaceAgentProfile(spaceId);
  const profile = await getKnowledgebaseAppSdkClient().client.knowledge.agentProfiles.retrieve(profileId);
  const params = parseModelParameters(profile.modelParameters);

  return {
    provider: params.uiProvider ?? 'Google',
    modelName: params.uiModelName ?? profile.modelId ?? 'gemini-3.5-flash',
    temperature: params.temperature ?? 0.7,
    maxTokens: params.maxTokens ?? 2048,
    systemPrompt: profile.systemInstruction,
  };
}

export async function applyKnowledgeSpaceModelSettings(
  spaceId: string,
  updates: Partial<KnowledgeBase>,
): Promise<void> {
  const profileId = await ensureSpaceAgentProfile(spaceId);
  const client = getKnowledgebaseAppSdkClient().client;
  const existing = await client.knowledge.agentProfiles.retrieve(profileId);
  const existingParams = parseModelParameters(existing.modelParameters);

  await client.knowledge.agentProfiles.update(profileId, {
    name: existing.name,
    description: existing.description,
    systemInstruction: updates.systemPrompt ?? existing.systemInstruction,
    modelProviderId: DEFAULT_MODEL_PROVIDER_ID,
    modelId: toBackendModelId(
      updates.provider ?? existingParams.uiProvider,
      updates.modelName ?? existingParams.uiModelName ?? existing.modelId,
    ),
    modelParameters: buildModelParameters(updates, existingParams),
    retrievalProfileId: existing.retrievalProfileId,
    citationPolicy: existing.citationPolicy,
    memoryPolicyRef: existing.memoryPolicyRef,
    toolPolicyRef: existing.toolPolicyRef,
    answerPolicy: existing.answerPolicy,
    knowledgeMode: existing.knowledgeMode ?? 'okf_bundle',
    agentImplementationId: existing.agentImplementationId ?? DEFAULT_AGENT_IMPLEMENTATION_ID,
    status: existing.status,
  });
}

async function findGuestContextBinding(spaceId: string) {
  const client = getKnowledgebaseAppSdkClient().client;
  const bindings = normalizeSdkWorkListPage<KnowledgeSpaceContextBinding>(
    await client.knowledge.spaces.contextBindings.list(String(spaceId)),
  );
  return bindings.items.find(
    (binding) => binding.contextId === GUEST_CONTEXT_BINDING_ID,
  );
}

export async function loadKnowledgeSpacePermissionSettings(
  spaceId: string,
): Promise<Pick<KnowledgeBase, 'publicPermission' | 'guestLinkEnabled'>> {
  const binding = await findGuestContextBinding(spaceId);
  if (!binding) {
    return { publicPermission: 'none', guestLinkEnabled: false };
  }

  const publicPermission =
    binding.accessLevel === 'writer' ? 'write' : 'read';
  return {
    publicPermission,
    guestLinkEnabled: true,
  };
}

export async function applyKnowledgeSpacePermissionSettings(
  spaceId: string,
  updates: Partial<KnowledgeBase>,
): Promise<void> {
  const shouldEnable =
    updates.guestLinkEnabled === true
    || (updates.publicPermission && updates.publicPermission !== 'none');

  const client = getKnowledgebaseAppSdkClient().client;
  const existing = await findGuestContextBinding(spaceId);

  if (!shouldEnable) {
    if (existing) {
      await client.knowledge.contextBindings.delete(existing.id);
    }
    return;
  }

  const accessLevel = mapAccessLevel(updates.publicPermission);
  if (existing) {
    await client.knowledge.contextBindings.update(existing.id, {
      contextName: GUEST_CONTEXT_BINDING_NAME,
      accessLevel,
    });
    return;
  }

  await client.knowledge.spaces.contextBindings.contextBindings(String(spaceId), {
    spaceId: String(spaceId),
    contextType: 'team',
    contextId: GUEST_CONTEXT_BINDING_ID,
    contextName: GUEST_CONTEXT_BINDING_NAME,
    accessLevel,
  });
}

export async function applyKnowledgeSpaceSettings(
  spaceId: string,
  updates: Partial<KnowledgeBase>,
): Promise<void> {
  const hasModelUpdates =
    updates.provider !== undefined
    || updates.modelName !== undefined
    || updates.temperature !== undefined
    || updates.maxTokens !== undefined
    || updates.systemPrompt !== undefined;

  if (hasModelUpdates) {
    await applyKnowledgeSpaceModelSettings(spaceId, updates);
  }

  if (updates.publicPermission !== undefined || updates.guestLinkEnabled !== undefined) {
    await applyKnowledgeSpacePermissionSettings(spaceId, updates);
  }
}

export async function hydrateKnowledgeBaseFromApi(
  kb: KnowledgeBase,
): Promise<KnowledgeBase> {
  const spaceId = String(kb.id);
  const numericSpaceId = Number(spaceId);
  if (!Number.isFinite(numericSpaceId) || numericSpaceId <= 0) {
    return kb;
  }

  try {
    const modelSettings = await loadKnowledgeSpaceModelSettings(spaceId);
    const permissionSettings = await loadKnowledgeSpacePermissionSettings(spaceId);
    return {
      ...kb,
      ...modelSettings,
      ...permissionSettings,
    };
  } catch {
    return kb;
  }
}
