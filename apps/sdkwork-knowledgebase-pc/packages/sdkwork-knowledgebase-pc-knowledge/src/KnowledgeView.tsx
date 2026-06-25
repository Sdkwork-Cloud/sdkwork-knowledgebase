import React, { useEffect, useMemo } from 'react';
import { KnowledgebaseRuntimeProvider } from 'sdkwork-knowledgebase-pc-core';

import { KnowledgeBaseApp, ToastContainer } from '../../sdkwork-knowledgebase-pc-knowledgebase/src';
import '../../../src/index.css';
import '../../../src/i18n';

import { bindHostSessionToKnowledgebaseStore } from './sessionBridge';
import { createHostManagedKnowledgebaseRuntime } from './createHostManagedKnowledgebaseRuntime';

export const KnowledgeView: React.FC = () => {
  const runtime = useMemo(() => createHostManagedKnowledgebaseRuntime(), []);

  useEffect(() => {
    return bindHostSessionToKnowledgebaseStore(runtime.session);
  }, [runtime.session]);

  return (
    <KnowledgebaseRuntimeProvider runtime={runtime}>
      <div className="flex flex-1 min-h-0 min-w-0 overflow-hidden bg-[var(--color-kb-bg-app,#1e1e1e)]">
        <React.Suspense
          fallback={(
            <div className="flex flex-1 items-center justify-center text-gray-400">
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
