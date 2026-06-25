import React from 'react';
import { X } from 'lucide-react';
import { useTranslation } from 'react-i18next';

interface WechatScanModalProps {
  isOpen: boolean;
  scanStatus: 'pending' | 'scanning' | 'success';
  scannedCover: string | null;
  onClose: () => void;
  triggerScanSimulation: () => void;
}

export function WechatScanModal({
  isOpen,
  scanStatus,
  scannedCover,
  onClose,
  triggerScanSimulation
}: WechatScanModalProps) {
  const { t } = useTranslation('editor');

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-zinc-950/40 backdrop-blur-md z-[300] flex items-center justify-center p-4 animate-in fade-in duration-200">
      <div className="bg-[var(--color-kb-panel)] border border-[var(--color-kb-panel-border)] rounded-2xl max-w-sm w-full overflow-hidden shadow-2xl flex flex-col">
        <div className="p-5 border-b border-[var(--color-kb-panel-border)] flex items-center justify-between">
          <span className="text-base font-bold text-[var(--color-kb-text-heading)]">{t('scanUploadTitle', { ns: 'editor' })}</span>
          <button onClick={onClose} className="text-[var(--color-kb-text-muted)] hover:text-[var(--color-kb-text)] transition-colors">
            <X size={18} />
          </button>
        </div>
        
        <div className="p-8 flex flex-col items-center">
          {scanStatus === 'pending' && (
            <div className="flex flex-col items-center text-center">
              <div className="relative border-4 border-[var(--color-kb-accent)] p-3 rounded-xl bg-white mb-4 shadow-md group">
                <div className="absolute inset-x-0 h-1 bg-[var(--color-kb-accent)] animate-bounce shadow-[0_0_10px_var(--color-kb-accent)]"></div>
                <div className="w-[180px] h-[180px] flex items-center justify-center bg-gray-50 rounded border border-gray-100">
                  <svg viewBox="0 0 24 24" className="w-[160px] h-[160px] text-gray-800" fill="currentColor">
                    <path d="M3 3h6v6H3V3zm2 2v2h2V5H5zm8-2h6v6h-6V3zm2 2v2h2V5h-2zM3 15h6v6H3v-6zm2 2v2h2v-2H5zm10-2h2v2h-2v-2zm2 2h2v2h-2v-2zm-2 2h2v2h-2v-2zm-2-2h2v2h-2v-2zm0 2h2v2h-2v-2zm4-4h2v2h-2v-2zm-4 0h2v2h-2v-2zm6-4h-2V9h2v2zm0-4h-2V3h2v2zm-2 4h-2v2h2v-2zm2 6h-2v-2h2v2zm-2 2h-2v2h2v-2z" />
                  </svg>
                </div>
              </div>
              <p className="text-[13.5px] font-semibold text-[var(--color-kb-text-heading)] mb-1">{t('scanPrompt', { ns: 'editor' })}</p>
              <p className="text-xs text-[var(--color-kb-text-muted)] mb-6">{t('scanDesc', { ns: 'editor' })}</p>
              
              <button 
                onClick={triggerScanSimulation}
                className="px-6 py-3 bg-[var(--color-kb-accent)] hover:bg-[var(--color-kb-accent-hover)] text-white text-xs font-bold rounded-xl shadow-md transition-all flex items-center gap-2 animate-pulse"
              >
                <span>{t('mockScanUpload', { ns: 'editor' })}</span>
              </button>
            </div>
          )}

          {scanStatus === 'scanning' && (
            <div className="flex flex-col items-center py-8 text-center">
              <div className="w-12 h-12 border-4 border-[var(--color-kb-accent)] border-t-transparent rounded-full animate-spin mb-4"></div>
              <p className="text-[14px] font-bold text-[var(--color-kb-text-heading)] mb-1">{t('waitingUpload', { ns: 'editor' })}</p>
              <p className="text-xs text-[var(--color-kb-text-muted)]">{t('doNotClose', { ns: 'editor' })}</p>
            </div>
          )}

          {scanStatus === 'success' && (
            <div className="flex flex-col items-center py-6 text-center">
              <div className="w-12 h-12 rounded-full bg-emerald-500/10 text-emerald-500 flex items-center justify-center mb-4 border border-emerald-500/20">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3">
                  <polyline points="20 6 9 17 4 12"></polyline>
                </svg>
              </div>
              <p className="text-[14px] font-bold text-emerald-500 mb-1">{t('uploadSuccess', { ns: 'editor' })}</p>
              <p className="text-xs text-[var(--color-kb-text-muted)]">{t('updatedCover', { ns: 'editor' })}</p>
              {scannedCover && (
                <img src={scannedCover} referrerPolicy="no-referrer" alt="Success cover" className="w-[140px] aspect-[900/383] object-cover rounded-md mt-4 border border-[var(--color-kb-panel-border)] shadow-sm animate-fade-in" />
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
