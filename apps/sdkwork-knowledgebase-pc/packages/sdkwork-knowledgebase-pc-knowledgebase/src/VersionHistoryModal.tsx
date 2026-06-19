import React from 'react';
import { useTranslation } from 'react-i18next';

export interface VersionHistoryModalProps {
  isOpen: boolean;
  item: { title: string; author?: string } | null;
  onClose: () => void;
}

export function VersionHistoryModal({ isOpen, item, onClose }: VersionHistoryModalProps) {
  const { t } = useTranslation(['kb', 'common']);
  if (!isOpen || !item) return null;

  return (
    <div className="fixed inset-0 bg-zinc-950/40 z-[1000] flex items-center justify-center backdrop-blur-sm select-none p-4">
      <div className="bg-white dark:bg-[var(--color-kb-editor)] w-[450px] rounded-2xl shadow-2xl border border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] p-6 animate-in zoom-in-95 duration-200">
        <h3 className="text-[16px] font-extrabold text-zinc-900 dark:text-[var(--color-kb-text-heading)] mb-1.5 tracking-tight">{t('versionHistory')}</h3>
        <p className="text-[13px] font-medium text-zinc-500 dark:text-[var(--color-kb-text-muted)] mb-6">
          {t('historyOf', { title: item.title, defaultValue: '{{title}} 的历史记录' })}
        </p>
        <div className="space-y-4 mb-6 max-h-[300px] overflow-y-auto pr-2 no-scrollbar">
          <div className="flex items-start gap-4 p-3 bg-zinc-50 dark:bg-[var(--color-kb-panel)] border border-indigo-100 dark:border-[var(--color-kb-panel-border)] rounded-xl transition-colors relative shadow-sm">
            <div className="absolute left-4 top-4 w-2.5 h-2.5 rounded-full bg-indigo-500 shadow-[0_0_0_4px_rgba(99,102,241,0.1)]"></div>
            <div className="ml-6">
              <div className="text-[14px] text-zinc-900 dark:text-[var(--color-kb-text-heading)] font-extrabold tracking-tight">{t('currentVersion')}</div>
              <div className="text-[12px] font-medium text-zinc-500 dark:text-[var(--color-kb-text-muted)] mt-1">今天 10:30 • <span className="text-zinc-600 dark:text-zinc-400">{item.author || 'Milo'}</span></div>
            </div>
          </div>
          <div className="flex items-start gap-4 p-3 hover:bg-zinc-50 dark:hover:bg-[var(--color-kb-panel-hover)] rounded-xl transition-all cursor-pointer border border-transparent">
            <div className="mt-1 ml-1 w-2.5 h-2.5 rounded-full bg-zinc-200 dark:bg-zinc-700"></div>
            <div className="ml-1.5">
              <div className="text-[14px] text-zinc-700 dark:text-[var(--color-kb-text)] font-bold tracking-tight opacity-70">{t('initialVersion')}</div>
              <div className="text-[12px] font-medium text-zinc-500 dark:text-[var(--color-kb-text-muted)] mt-1">2023-10-01 12:00 • <span>{item.author || 'Milo'}</span></div>
            </div>
          </div>
        </div>
        <div className="flex justify-end border-t border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] pt-5">
          <button 
            type="button" 
            onClick={onClose} 
            className="px-6 py-2 text-[13px] font-bold bg-white dark:bg-[var(--color-kb-panel-hover)] border-2 border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] text-zinc-700 dark:text-[var(--color-kb-text-heading)] hover:bg-zinc-50 dark:hover:bg-[var(--color-kb-panel)] rounded-xl transition-all active:scale-95 shadow-sm cursor-pointer"
          >
            {t('close', { ns: 'common', defaultValue: '关闭' })}
          </button>
        </div>
      </div>
    </div>
  );
}
