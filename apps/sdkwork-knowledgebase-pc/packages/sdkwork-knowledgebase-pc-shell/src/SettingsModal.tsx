import React, { useEffect, useState } from 'react';
import { X, Monitor, Palette, MousePointer2, Info, LogOut, UserRound } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useLocalStorage } from '@packages/sdkwork-knowledgebase-pc-commons/src';
import type { KnowledgebaseAccountViewModel, KnowledgebaseRuntimeConfig } from 'sdkwork-knowledgebase-pc-core';

export interface SettingsModalProps {
  account?: KnowledgebaseAccountViewModel;
  hosting?: KnowledgebaseRuntimeConfig['hosting'];
  initialTab?: string;
  isOpen: boolean;
  onClose: () => void;
  onSignOut?: () => void | Promise<void>;
  runtimeConfig?: KnowledgebaseRuntimeConfig;
  theme: 'light' | 'dark' | 'system';
  setTheme: (theme: 'light' | 'dark' | 'system') => void;
}

export function SettingsModal({
  account,
  hosting,
  initialTab,
  isOpen,
  onClose,
  onSignOut,
  runtimeConfig,
  theme,
  setTheme,
}: SettingsModalProps) {
  const { t } = useTranslation('shell');
  const [activeTab, setActiveTab] = useLocalStorage('app-settings-tab', 'appearance');
  const [activeColor, setActiveColor] = useLocalStorage('app-accent-color', '#2563eb');
  const [fontSize, setFontSize] = useLocalStorage('app-font-size', 'normal');

  useEffect(() => {
    if (isOpen && initialTab) {
      setActiveTab(initialTab);
    }
  }, [initialTab, isOpen, setActiveTab]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/50 backdrop-blur-[2px] transition-opacity">
      <div className="w-[720px] h-[500px] bg-[var(--color-kb-editor)] border border-[var(--color-kb-panel-border)] rounded-xl shadow-[0_20px_50px_rgba(0,0,0,0.3)] flex overflow-hidden transform transition-all">
        
        {/* Sidebar */}
        <div className="w-[200px] bg-[var(--color-kb-panel)] border-r border-[var(--color-kb-panel-border)] p-4 flex flex-col">
          <div className="text-sm font-semibold text-[var(--color-kb-text-heading)] mb-6 mt-2 ml-2">{t('systemSettings')}</div>
          
          <div className="space-y-1">
            {account ? (
              <TabButton active={activeTab === 'account'} onClick={() => setActiveTab('account')} icon={<UserRound size={16}/>} label={t('account')} />
            ) : null}
            <TabButton active={activeTab === 'general'} onClick={() => setActiveTab('general')} icon={<Monitor size={16}/>} label={t('general')} />
            <TabButton active={activeTab === 'appearance'} onClick={() => setActiveTab('appearance')} icon={<Palette size={16}/>} label={t('appearance')} />
            <TabButton active={activeTab === 'shortcuts'} onClick={() => setActiveTab('shortcuts')} icon={<MousePointer2 size={16}/>} label={t('shortcut')} />
            <TabButton active={activeTab === 'about'} onClick={() => setActiveTab('about')} icon={<Info size={16}/>} label={t('about')} />
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 flex flex-col relative w-full overflow-hidden bg-[var(--color-kb-editor)]">
          <div className="h-12 flex items-center justify-end px-4 flex-shrink-0 drag-area">
            <button onClick={onClose} title={t('close')} className="p-1.5 text-[var(--color-kb-text-muted)] hover:text-[var(--color-kb-text-heading)] hover:bg-[var(--color-kb-panel-hover)] rounded-md transition-colors z-10">
              <X size={18} />
            </button>
          </div>
          
          <div className="flex-1 px-10 pb-10 overflow-y-auto w-full scrollbar-y">
            {activeTab === 'account' && account ? (
              <div className="space-y-6 max-w-lg">
                <div>
                  <h3 className="text-lg font-medium text-[var(--color-kb-text-heading)] mb-1">{t('account')}</h3>
                  <p className="text-xs text-[var(--color-kb-text-muted)] mb-6">{t('accountDescription')}</p>
                </div>
                <div className="space-y-4 rounded-xl border border-[var(--color-kb-panel-border)] bg-[var(--color-kb-panel)] p-4">
                  <AccountInfoRow label={t('displayName')} value={account.displayName} />
                  <AccountInfoRow label={t('email')} value={account.email ?? '—'} />
                  <AccountInfoRow label={t('tenant')} value={account.tenantId ?? '—'} />
                  <AccountInfoRow label={t('environment')} value={account.environmentLabel} />
                  <AccountInfoRow label={t('hosting')} value={hosting ?? runtimeConfig?.hosting ?? '—'} />
                </div>
                {onSignOut ? (
                  <button
                    type="button"
                    onClick={() => {
                      onClose();
                      void onSignOut();
                    }}
                    className="inline-flex items-center gap-2 rounded-lg border border-rose-200 px-4 py-2 text-sm font-medium text-rose-600 transition-colors hover:bg-rose-50 dark:border-rose-900/40 dark:text-rose-400 dark:hover:bg-rose-950/30"
                  >
                    <LogOut size={16} />
                    {t('signOut')}
                  </button>
                ) : null}
              </div>
            ) : null}

            {activeTab === 'appearance' && (
              <div className="space-y-8 max-w-lg">
                <div>
                  <h3 className="text-lg font-medium text-[var(--color-kb-text-heading)] mb-1">{t('appearance')}</h3>
                  <p className="text-xs text-[var(--color-kb-text-muted)] mb-6">{t('appearanceDescription')}</p>
                </div>
                
                <div className="space-y-4">
                  <div className="text-sm font-medium text-[var(--color-kb-text)] mb-3">{t('theme')}</div>
                  <div className="grid grid-cols-3 gap-6">
                    <ThemeCard active={theme === 'light'} onClick={() => setTheme('light')} type="light" label={t('light')} />
                    <ThemeCard active={theme === 'dark'} onClick={() => setTheme('dark')} type="dark" label={t('dark')} />
                    <ThemeCard active={theme === 'system'} onClick={() => setTheme('system')} type="system" label={t('system')} />
                  </div>
                </div>

                <div className="pt-8 border-t border-[var(--color-kb-panel-border)]">
                   <div className="flex items-center justify-between">
                     <div>
                       <div className="text-sm font-medium text-[var(--color-kb-text)] mb-1">{t('accentColor')}</div>
                       <div className="text-xs text-[var(--color-kb-text-muted)]">{t('accentColorDescription')}</div>
                     </div>
                     <div className="flex items-center space-x-3">
                       <ColorDot color="#07c160" active={activeColor === '#07c160'} onClick={() => setActiveColor('#07c160')} />
                       <ColorDot color="#3b82f6" active={activeColor === '#3b82f6'} onClick={() => setActiveColor('#3b82f6')} />
                       <ColorDot color="#8b5cf6" active={activeColor === '#8b5cf6'} onClick={() => setActiveColor('#8b5cf6')} />
                       <ColorDot color="#ef4444" active={activeColor === '#ef4444'} onClick={() => setActiveColor('#ef4444')} />
                     </div>
                   </div>
                </div>
                
                <div className="pt-8 border-t border-[var(--color-kb-panel-border)]">
                   <div className="flex items-center justify-between">
                     <div>
                       <div className="text-sm font-medium text-[var(--color-kb-text)] mb-1">{t('language')}</div>
                       <div className="text-xs text-[var(--color-kb-text-muted)]">{t('languageDescription')}</div>
                     </div>
                     <LanguageSwitch />
                   </div>
                </div>
                
                <div className="pt-8 border-t border-[var(--color-kb-panel-border)]">
                   <div className="flex items-center justify-between">
                     <div>
                       <div className="text-sm font-medium text-[var(--color-kb-text)] mb-1">{t('fontSize')}</div>
                       <div className="text-xs text-[var(--color-kb-text-muted)]">{t('fontSizeDescription')}</div>
                     </div>
                     <div className="flex items-center space-x-1 border border-[var(--color-kb-panel-border)] rounded-lg p-1 bg-[var(--color-kb-panel)]">
                       <button onClick={() => setFontSize('small')} className={`px-3 py-1 rounded text-xs transition-colors ${fontSize === 'small' ? 'text-[var(--color-kb-accent)] bg-[var(--color-kb-panel-active)] font-medium shadow-sm' : 'text-[var(--color-kb-text)] hover:bg-[var(--color-kb-panel-hover)]'}`}>{t('small')}</button>
                       <button onClick={() => setFontSize('normal')} className={`px-3 py-1 rounded text-xs transition-colors ${fontSize === 'normal' ? 'text-[var(--color-kb-accent)] bg-[var(--color-kb-panel-active)] font-medium shadow-sm' : 'text-[var(--color-kb-text)] hover:bg-[var(--color-kb-panel-hover)]'}`}>{t('normal')}</button>
                       <button onClick={() => setFontSize('large')} className={`px-3 py-1 rounded text-xs transition-colors ${fontSize === 'large' ? 'text-[var(--color-kb-accent)] bg-[var(--color-kb-panel-active)] font-medium shadow-sm' : 'text-[var(--color-kb-text)] hover:bg-[var(--color-kb-panel-hover)]'}`}>{t('large')}</button>
                     </div>
                   </div>
                </div>
              </div>
            )}

            {activeTab === 'general' && (
              <div className="space-y-8 max-w-lg">
                <div>
                  <h3 className="text-lg font-medium text-[var(--color-kb-text-heading)] mb-1">{t('general')}</h3>
                  <p className="text-xs text-[var(--color-kb-text-muted)] mb-6">{t('generalDescription')}</p>
                </div>
                
                <div className="space-y-8">
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="text-sm font-medium text-[var(--color-kb-text)] mb-1">{t('autoStart')}</div>
                      <div className="text-xs text-[var(--color-kb-text-muted)]">{t('autoStartDescription')}</div>
                    </div>
                    <ToggleSwitch defaultActive={false} />
                  </div>
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="text-sm font-medium text-[var(--color-kb-text)] mb-1">{t('hideToTray')}</div>
                      <div className="text-xs text-[var(--color-kb-text-muted)]">{t('hideToTrayDescription')}</div>
                    </div>
                    <ToggleSwitch defaultActive={true} />
                  </div>
                </div>
              </div>
            )}
            
            {(activeTab === 'shortcuts' || activeTab === 'about') && (
               <div className="h-[300px] flex items-center justify-center text-[var(--color-kb-text-muted)] text-sm">
                 {t('underDevelopment')}
               </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

function LanguageSwitch() {
  const { t, i18n } = useTranslation('shell');
  return (
    <div className="flex items-center space-x-1 border border-[var(--color-kb-panel-border)] rounded-lg p-1 bg-[var(--color-kb-panel)]">
      <button onClick={() => i18n.changeLanguage('zh')} className={`px-3 py-1 rounded text-xs transition-colors ${i18n.language === 'zh' ? 'text-[var(--color-kb-accent)] bg-[var(--color-kb-panel-active)] font-medium shadow-sm' : 'text-[var(--color-kb-text)] hover:bg-[var(--color-kb-panel-hover)]'}`}>{t('zh')}</button>
      <button onClick={() => i18n.changeLanguage('en')} className={`px-3 py-1 rounded text-xs transition-colors ${i18n.language === 'en' ? 'text-[var(--color-kb-accent)] bg-[var(--color-kb-panel-active)] font-medium shadow-sm' : 'text-[var(--color-kb-text)] hover:bg-[var(--color-kb-panel-hover)]'}`}>{t('en')}</button>
    </div>
  );
}


function AccountInfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-start justify-between gap-4 text-sm">
      <span className="text-[var(--color-kb-text-muted)]">{label}</span>
      <span className="text-right font-medium text-[var(--color-kb-text-heading)] break-all">{value}</span>
    </div>
  );
}

function TabButton({ active, icon, label, onClick }: { active: boolean, icon: React.ReactNode, label: string, onClick: () => void }) {
  return (
    <button 
      onClick={onClick}
      className={`w-full flex items-center px-3 py-2.5 rounded-md text-sm transition-all ${active ? 'bg-[var(--color-kb-panel-active)] text-[var(--color-kb-accent)] font-medium shadow-sm border border-[var(--color-kb-panel-border)]/50' : 'text-[var(--color-kb-panel-text)] hover:bg-[var(--color-kb-panel-hover)] border border-transparent font-normal'}`}
    >
      <span className={`mr-3 ${active ? 'text-[var(--color-kb-accent)]' : 'text-[var(--color-kb-text-muted)]'}`}>{icon}</span>
      {label}
    </button>
  );
}

function ThemeCard({ active, type, label, onClick }: { active: boolean, type: 'light' | 'dark' | 'system', label: string, onClick: () => void }) {
  return (
    <div onClick={onClick} className="cursor-pointer group flex flex-col items-center">
      <div className={`w-full aspect-[4/3] rounded-xl border-2 mb-3 p-1.5 flex flex-col justify-between transition-all ${active ? 'border-[var(--color-kb-accent)] shadow-md scale-100 ring-4 ring-[var(--color-kb-accent)]/10' : 'border-[var(--color-kb-panel-border)] group-hover:border-[var(--color-kb-text-muted)] scale-[0.98]'}`}>
        {type === 'light' && (
          <div className="w-full h-full bg-[#f8f9fa] rounded-md overflow-hidden flex flex-col shadow-inner">
            <div className="w-full h-3 bg-white border-b border-gray-200"></div>
            <div className="flex-1 flex">
              <div className="w-1/3 border-r border-gray-200 bg-[#f3f4f6]"></div>
              <div className="flex-1 bg-white p-1">
                <div className="w-3/4 h-1 bg-gray-200 rounded-sm mb-1"></div>
                <div className="w-1/2 h-1 bg-gray-200 rounded-sm"></div>
              </div>
            </div>
          </div>
        )}
        {type === 'dark' && (
          <div className="w-full h-full bg-[#1b1d22] rounded-md overflow-hidden flex flex-col shadow-inner">
            <div className="w-full h-3 bg-[#15171a] border-b border-[#26282e]"></div>
            <div className="flex-1 flex">
              <div className="w-1/3 border-r border-[#26282e] bg-[#1a1c20]"></div>
              <div className="flex-1 bg-[#1b1d22] p-1">
                <div className="w-3/4 h-1 bg-[#2a2c32] rounded-sm mb-1"></div>
                <div className="w-1/2 h-1 bg-[#2a2c32] rounded-sm"></div>
              </div>
            </div>
          </div>
        )}
        {type === 'system' && (
          <div className="w-full h-full rounded-md overflow-hidden flex relative shadow-inner">
            <div className="absolute inset-0 bg-[#f8f9fa] flex flex-col z-0">
               <div className="w-full h-3 bg-white border-b border-gray-200"></div>
               <div className="flex-1 flex">
                 <div className="w-1/3 border-r border-gray-200 bg-[#f3f4f6]"></div>
                 <div className="flex-1 bg-white p-1"></div>
               </div>
            </div>
            
            <div className="absolute inset-0 bg-[#1b1d22] flex flex-col z-10" style={{ clipPath: 'polygon(100% 0, 100% 100%, 0 100%)' }}>
               <div className="w-full h-3 bg-[#15171a] border-b border-[#26282e]"></div>
               <div className="flex-1 flex">
                 <div className="w-1/3 border-r border-[#26282e] bg-[#1a1c20]"></div>
                 <div className="flex-1 bg-[#1b1d22] p-1"></div>
               </div>
            </div>
          </div>
        )}
      </div>
      <span className={`text-sm ${active ? 'text-[var(--color-kb-text-heading)] font-medium' : 'text-[var(--color-kb-text)]'}`}>{label}</span>
      {active && <div className="w-4 h-4 bg-[var(--color-kb-accent)] rounded-full mt-2 flex items-center justify-center">
        <svg className="w-2.5 h-2.5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={3}><path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" /></svg>
      </div>}
      {!active && <div className="w-4 h-4 rounded-full border border-[var(--color-kb-panel-border)] mt-2 bg-[var(--color-kb-editor)]"></div>}
    </div>
  );
}

function ColorDot({ color, active, onClick }: { color: string, active: boolean, onClick: () => void }) {
  return (
    <div onClick={onClick} className={`w-6 h-6 rounded-full cursor-pointer flex items-center justify-center transition-all ${active ? 'ring-2 ring-offset-2 ring-offset-[var(--color-kb-editor)] ring-[var(--color-kb-accent)] scale-110' : 'ring-0 hover:scale-110'}`} style={{ backgroundColor: color }}>
    </div>
  );
}

function ToggleSwitch({ defaultActive = false }: { defaultActive?: boolean }) {
  const [active, setActive] = useState(defaultActive);
  return (
    <div 
      onClick={() => setActive(!active)}
      className={`w-11 h-6 rounded-full p-1 cursor-pointer transition-colors ${active ? 'bg-[var(--color-kb-accent)]' : 'bg-[#e5e7eb] dark:bg-[#374151]'}`}
    >
      <div className={`w-4 h-4 bg-white rounded-full shadow-sm transition-transform ${active ? 'translate-x-5' : 'translate-x-0'}`}></div>
    </div>
  );
}
