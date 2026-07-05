import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { BookOpen, ExternalLink, X } from 'lucide-react';

import '@sdkwork/knowledgebase-pc-knowledge/i18n';
import { KnowledgebaseHostSurface } from './KnowledgebaseHostSurface';
import type { KnowledgebaseHostContext } from './knowledgebaseHostPresentation';
import {
  resolveKnowledgebaseHostPresentationMode,
  resolveKnowledgebaseHostRuntimeTarget,
} from './knowledgebaseHostPresentation';
import { openKnowledgebaseDesktopWindow } from './openKnowledgebaseDesktopWindow';

export interface KnowledgebaseModalProps {
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  context?: KnowledgebaseHostContext;
}

export const KnowledgebaseModal: React.FC<KnowledgebaseModalProps> = ({
  isOpen,
  onClose,
  title,
  context,
}) => {
  const { t } = useTranslation('shell');
  const [detachedWindowError, setDetachedWindowError] = useState<string | null>(null);
  const presentationMode = useMemo(() => resolveKnowledgebaseHostPresentationMode(), []);
  const runtimeTarget = useMemo(() => resolveKnowledgebaseHostRuntimeTarget(), []);
  const resolvedTitle = title?.trim() || context?.groupName?.trim() || t('knowledgeBase');

  useEffect(() => {
    if (!isOpen) {
      setDetachedWindowError(null);
      return undefined;
    }

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [isOpen, onClose]);

  const openDetachedWindow = useCallback(async () => {
    try {
      const opened = await openKnowledgebaseDesktopWindow({
        title: resolvedTitle,
        context,
      });
      if (!opened) {
        setDetachedWindowError(t('hostModalDesktopBridgeUnavailable'));
        return;
      }
      setDetachedWindowError(null);
      onClose();
    } catch (error) {
      const message = error instanceof Error ? error.message : t('hostModalOpenWindowFailed');
      setDetachedWindowError(message);
    }
  }, [context, onClose, resolvedTitle, t]);

  useEffect(() => {
    if (!isOpen || presentationMode !== 'detached-window') {
      return;
    }
    void openDetachedWindow();
  }, [isOpen, openDetachedWindow, presentationMode]);

  if (!isOpen) {
    return null;
  }

  if (presentationMode === 'detached-window') {
    return (
      <div className="fixed inset-0 z-[120] flex items-center justify-center bg-black/50 backdrop-blur-sm">
        <div className="w-[min(92vw,420px)] rounded-2xl border border-white/10 bg-[#202020] p-6 text-center shadow-2xl">
          <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-xl bg-blue-500/10 text-blue-400">
            <BookOpen size={22} />
          </div>
          <h2 className="text-lg font-semibold text-gray-100">{resolvedTitle}</h2>
          <p className="mt-2 text-sm text-gray-400">
            {t('hostModalOpeningDetachedWindow')}
          </p>
          {detachedWindowError ? (
            <p className="mt-3 text-sm text-red-400">{detachedWindowError}</p>
          ) : null}
          <div className="mt-6 flex items-center justify-center gap-3">
            <button
              type="button"
              onClick={() => void openDetachedWindow()}
              className="rounded-xl bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-500"
            >
              {t('hostModalOpenWindow')}
            </button>
            <button
              type="button"
              onClick={onClose}
              className="rounded-xl px-4 py-2 text-sm font-medium text-gray-300 hover:bg-white/10"
            >
              {t('hostModalCancel')}
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 z-[120] flex items-center justify-center bg-black/55 p-4 backdrop-blur-sm md:p-8">
      <div
        className="absolute inset-0"
        onClick={onClose}
        aria-hidden
      />
      <div className="relative flex h-[min(88vh,860px)] w-[min(96vw,1280px)] flex-col overflow-hidden rounded-2xl border border-white/10 bg-[#1e1e1e] shadow-2xl">
        <div className="flex shrink-0 items-center gap-3 border-b border-white/10 bg-[#202020] px-4 py-3">
          <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-blue-500/10 text-blue-400">
            <BookOpen size={18} />
          </div>
          <div className="min-w-0 flex-1">
            <h2 className="truncate text-base font-semibold text-gray-100">{resolvedTitle}</h2>
            {context?.groupName ? (
              <p className="truncate text-xs text-gray-500">
                {t('hostModalGroupContext', { groupName: context.groupName })}
              </p>
            ) : null}
          </div>
          {runtimeTarget === 'desktop' ? (
            <button
              type="button"
              onClick={() => void openDetachedWindow()}
              className="flex items-center gap-1 rounded-lg px-3 py-2 text-xs font-medium text-gray-300 transition-colors hover:bg-white/10 hover:text-white"
              title={t('hostModalOpenDetachedWindow')}
            >
              <ExternalLink size={14} />
              <span>{t('hostModalOpenDetachedWindow')}</span>
            </button>
          ) : null}
          <button
            type="button"
            onClick={onClose}
            className="flex h-9 w-9 items-center justify-center rounded-lg text-gray-400 transition-colors hover:bg-white/10 hover:text-white"
            aria-label={t('close')}
          >
            <X size={18} />
          </button>
        </div>

        <div className="flex min-h-0 flex-1 overflow-hidden">
          <KnowledgebaseHostSurface
            presentationMode={presentationMode}
            title={resolvedTitle}
            context={context}
          />
        </div>
      </div>
    </div>
  );
};
