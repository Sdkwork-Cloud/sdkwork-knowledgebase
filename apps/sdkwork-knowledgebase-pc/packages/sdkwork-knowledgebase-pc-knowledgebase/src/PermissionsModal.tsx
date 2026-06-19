import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';

export interface PermissionsModalProps {
  isOpen: boolean;
  item: { title: string; author?: string } | null;
  onClose: () => void;
  onSave?: (settings: any) => void;
}

export function PermissionsModal({ isOpen, item, onClose, onSave }: PermissionsModalProps) {
  const { t } = useTranslation(['kb', 'common']);
  const [publicLink, setPublicLink] = useState('closed');
  const [teamVisibility, setTeamVisibility] = useState('all-editor');

  if (!isOpen || !item) return null;

  const handleComplete = () => {
    if (onSave) {
      onSave({ publicLink, teamVisibility });
    }
    onClose();
  };

  return (
    <div className="fixed inset-0 bg-zinc-950/40 z-[1000] flex items-center justify-center backdrop-blur-sm p-4 select-none">
      <div className="bg-white dark:bg-[var(--color-kb-editor)] w-[500px] rounded-2xl shadow-2xl border border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] overflow-hidden animate-in zoom-in-95 duration-200">
        <div className="p-6 border-b border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] bg-[#fafafa] dark:bg-[var(--color-kb-panel)]/30">
          <h3 className="text-[16px] font-extrabold text-zinc-900 dark:text-[var(--color-kb-text-heading)] tracking-tight leading-tight mb-1">{t('memberPermissions')}</h3>
          <p className="text-[13px] font-medium text-zinc-500 dark:text-[var(--color-kb-text-muted)]">
            {t('setPermissionsFor')} {item.title}
          </p>
        </div>
        <div className="p-6 pb-2">
          <div className="border border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] rounded-2xl divide-y divide-zinc-200/80 dark:divide-[var(--color-kb-panel-border)] mb-6 overflow-hidden shadow-sm">
            <div className="flex justify-between items-center p-4 bg-white dark:bg-[var(--color-kb-editor)] hover:bg-zinc-50 dark:hover:bg-[var(--color-kb-panel)] transition-colors">
              <div className="flex flex-col">
                <span className="text-[14px] text-zinc-900 dark:text-[var(--color-kb-text-heading)] font-bold">{t('publicLink')}</span>
                <span className="text-[11px] text-zinc-500 dark:text-[var(--color-kb-text-muted)] font-medium mt-0.5 mt-0.5">{t('publicLinkDesc')}</span>
              </div>
              <select 
                value={publicLink}
                onChange={(e) => setPublicLink(e.target.value)}
                className="text-[13px] font-bold bg-zinc-50 dark:bg-[var(--color-kb-panel-border)] text-zinc-700 dark:text-[var(--color-kb-text-heading)] outline-none border border-zinc-200/80 dark:border-transparent rounded-xl px-3 py-2 cursor-pointer focus:border-indigo-400 focus:ring-4 focus:ring-indigo-500/10 transition-all hover:bg-zinc-100 dark:hover:bg-[var(--color-kb-panel-border)]"
              >
                <option value="closed">{t('closedPrivate')}</option>
                <option value="view">{t('publicVisible')}</option>
                <option value="edit">{t('publicEdit')}</option>
              </select>
            </div>
            <div className="flex justify-between items-center p-4 bg-white dark:bg-[var(--color-kb-editor)] hover:bg-zinc-50 dark:hover:bg-[var(--color-kb-panel)] transition-colors">
              <div className="flex flex-col">
                <span className="text-[14px] text-zinc-900 dark:text-[var(--color-kb-text-heading)] font-bold">{t('teamVisibility')}</span>
                <span className="text-[11px] text-zinc-500 dark:text-[var(--color-kb-text-muted)] font-medium mt-0.5 mt-0.5">{t('teamVisibilityDesc')}</span>
              </div>
              <select 
                value={teamVisibility}
                onChange={(e) => setTeamVisibility(e.target.value)}
                className="text-[13px] font-bold bg-zinc-50 dark:bg-[var(--color-kb-panel-border)] text-zinc-700 dark:text-[var(--color-kb-text-heading)] outline-none border border-zinc-200/80 dark:border-transparent rounded-xl px-3 py-2 cursor-pointer focus:border-indigo-400 focus:ring-4 focus:ring-indigo-500/10 transition-all hover:bg-zinc-100 dark:hover:bg-[var(--color-kb-panel-border)]"
              >
                <option value="all-editor">{t('allEditor')}</option>
                <option value="all-viewer">{t('allViewer')}</option>
                <option value="specific">{t('specific')}</option>
              </select>
            </div>
          </div>
        </div>
        <div className="flex justify-end p-5 pt-0">
          <button 
            type="button" 
            onClick={handleComplete} 
            className="px-6 py-2.5 text-[13px] font-extrabold bg-[#07C160] hover:bg-[#06ad56] text-white rounded-xl shadow-md transition-all active:scale-95 hover:shadow-lg focus:outline-none focus:ring-4 focus:ring-[#07C160]/20"
          >
            {t('save', { ns: 'common' })}
          </button>
        </div>
      </div>
    </div>
  );
}
