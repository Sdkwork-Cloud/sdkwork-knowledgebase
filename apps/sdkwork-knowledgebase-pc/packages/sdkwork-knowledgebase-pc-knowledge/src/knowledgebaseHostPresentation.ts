export type KnowledgebaseHostPresentationMode = 'inline' | 'detached-iframe' | 'detached-window';

export type KnowledgebaseHostRuntimeTarget = 'browser' | 'desktop';
export interface KnowledgebaseHostContext {
  groupId?: string;
  groupName?: string;
}

type SdkworkTauriBridge = {
  __TAURI__?: {
    core?: {
      invoke?: unknown;
    };
  };
};

function detectHostRuntimeTarget(): KnowledgebaseHostRuntimeTarget {
  const configured = import.meta.env.VITE_SDKWORK_KNOWLEDGEBASE_HOST_RUNTIME_TARGET;
  if (configured === 'desktop' || configured === 'browser') {
    return configured;
  }

  const tauri = (globalThis as SdkworkTauriBridge).__TAURI__;
  if (typeof tauri?.core?.invoke === 'function') {
    return 'desktop';
  }

  return 'browser';
}

function resolveConfiguredPresentationMode(): KnowledgebaseHostPresentationMode | null {
  const configured = import.meta.env.VITE_SDKWORK_KNOWLEDGEBASE_HOST_PRESENTATION_MODE;
  if (
    configured === 'inline'
    || configured === 'detached-iframe'
    || configured === 'detached-window'
  ) {
    return configured;
  }
  return null;
}

export function resolveKnowledgebaseHostPresentationMode(): KnowledgebaseHostPresentationMode {
  const configured = resolveConfiguredPresentationMode();
  if (configured) {
    return configured;
  }

  return detectHostRuntimeTarget() === 'desktop' ? 'detached-iframe' : 'inline';
}

export function resolveKnowledgebaseHostRuntimeTarget(): KnowledgebaseHostRuntimeTarget {
  return detectHostRuntimeTarget();
}

export function isKnowledgebaseHostDesktopRuntime(): boolean {
  return resolveKnowledgebaseHostRuntimeTarget() === 'desktop';
}
