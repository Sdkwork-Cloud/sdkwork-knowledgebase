import { createContext, useContext, type ReactNode } from 'react';

import type { HostAdapter } from '../host/hostAdapter';
import type { KnowledgebaseRuntimeConfig } from '../config/runtimeConfig';
import type { KnowledgebaseAppSdkClient } from '../sdk/knowledgebaseAppSdkClient';
import type { SessionStore } from '../session/sessionStore';

export interface KnowledgebaseCoreRuntime {
  config: KnowledgebaseRuntimeConfig;
  auth?: {
    iamRuntime: unknown;
  };
  sdk: {
    app: KnowledgebaseAppSdkClient;
  };
  session: SessionStore;
  host: HostAdapter;
}

export interface KnowledgebasePcRuntime extends KnowledgebaseCoreRuntime {}

const KnowledgebaseRuntimeContext = createContext<KnowledgebasePcRuntime | null>(null);

export interface KnowledgebaseRuntimeProviderProps {
  runtime: KnowledgebasePcRuntime;
  children: ReactNode;
}

export function KnowledgebaseRuntimeProvider({
  runtime,
  children,
}: KnowledgebaseRuntimeProviderProps) {
  return (
    <KnowledgebaseRuntimeContext.Provider value={runtime}>
      {children}
    </KnowledgebaseRuntimeContext.Provider>
  );
}

export function useKnowledgebaseRuntime(): KnowledgebasePcRuntime {
  const runtime = useContext(KnowledgebaseRuntimeContext);
  if (!runtime) {
    throw new Error('useKnowledgebaseRuntime must be used within KnowledgebaseRuntimeProvider');
  }
  return runtime;
}

export function useKnowledgebaseRuntimeConfig(): KnowledgebaseRuntimeConfig {
  return useKnowledgebaseRuntime().config;
}

export function useKnowledgebaseHostAdapter(): HostAdapter {
  return useKnowledgebaseRuntime().host;
}
