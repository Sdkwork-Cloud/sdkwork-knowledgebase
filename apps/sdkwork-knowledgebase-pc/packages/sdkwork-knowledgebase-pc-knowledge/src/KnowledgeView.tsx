import React, { useEffect, useMemo } from 'react';
import { KnowledgebaseRuntimeProvider } from 'sdkwork-knowledgebase-pc-core';

import { KnowledgeBaseApp, ToastContainer } from '../../sdkwork-knowledgebase-pc-knowledgebase/src';
import '../../../src/index.css';
import '../../../src/i18n';

import { bindHostSessionToKnowledgebaseStore } from './sessionBridge';
import { createHostManagedKnowledgebaseRuntime } from './createHostManagedKnowledgebaseRuntime';
import {
  subscribeKnowledgebaseHostLanguage,
  syncKnowledgebaseHostLanguage,
} from './hostLanguageBridge';

function syncHostManagedKnowledgebaseAppearance(): void {
  if (typeof document === 'undefined') {
    return;
  }

  const isDark = document.documentElement.classList.contains('dark')
    || !document.documentElement.classList.contains('light-mode');
  document.documentElement.classList.toggle('dark', isDark);
}

export const KnowledgeView: React.FC = () => {
  const runtime = useMemo(() => {
    syncKnowledgebaseHostLanguage();
    return createHostManagedKnowledgebaseRuntime();
  }, []);

  useEffect(() => {
    return bindHostSessionToKnowledgebaseStore(runtime.session);
  }, [runtime.session]);

  useEffect(() => {
    syncKnowledgebaseHostLanguage();
    return subscribeKnowledgebaseHostLanguage();
  }, []);

  useEffect(() => {
    syncHostManagedKnowledgebaseAppearance();

    if (typeof document === 'undefined') {
      return undefined;
    }

    const observer = new MutationObserver(() => {
      syncHostManagedKnowledgebaseAppearance();
    });
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ['class'],
    });

    return () => observer.disconnect();
  }, []);

  return (
    <KnowledgebaseRuntimeProvider runtime={runtime}>
      <div className="flex flex-1 min-h-0 min-w-0 overflow-hidden bg-[var(--color-kb-bg-app,#1e1e1e)]">
        <React.Suspense
          fallback={(
            <div className="flex flex-1 items-center justify-center text-kb-text-muted">
              Loading knowledge base...
            </div>
          )}
        >
          <KnowledgeBaseApp />
        </React.Suspense>
        <ToastContainer />
      </div>
    </KnowledgebaseRuntimeProvider>
  );
};
