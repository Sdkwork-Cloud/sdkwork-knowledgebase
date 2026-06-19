import React from 'react';
import { Cpu, Activity, Play, Terminal } from 'lucide-react';
import { useTranslation } from 'react-i18next';

export interface McpConsolePanelProps {
  isOpen: boolean;
  onToggle: () => void;
  isTyping: boolean;
  onTriggerQuickTool: (comm: string) => void;
}

export function McpConsolePanel({ isOpen, onToggle, isTyping, onTriggerQuickTool }: McpConsolePanelProps) {
  const { t } = useTranslation('mcp');

  return (
    <div className="bg-gradient-to-br from-indigo-500/5 via-teal-500/5 to-emerald-500/5 border border-[var(--color-kb-panel-border)] rounded-2xl overflow-hidden shadow-xs">
      <div 
        onClick={onToggle}
        className="flex items-center justify-between px-3.5 py-3 bg-[var(--color-kb-panel-hover)] border-b border-[var(--color-kb-panel-border)] cursor-pointer hover:bg-[var(--color-kb-panel)] transition-colors select-none"
      >
        <div className="flex items-center gap-2">
          <span className="relative flex h-2 w-2">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
            <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
          </span>
          <span className="text-[12px] font-bold text-[var(--color-kb-text-heading)] flex items-center gap-1">
            <Cpu size={13} className="text-emerald-500" />
            {t('mcpConsoleTitle')}
          </span>
        </div>
        <span className="text-xs text-[var(--color-kb-text-muted)] bg-emerald-500/10 text-emerald-600 px-1.5 py-0.2 rounded-md font-bold text-[9.5px]">
          {t('activeStatus')}
        </span>
      </div>

      {isOpen && (
        <div className="p-3.5 space-y-3">
          <div className="text-[11px] text-[var(--color-kb-text-muted)] leading-relaxed font-semibold">
            {t('mcpConsoleDesc')}
          </div>

          <div className="space-y-1.5">
            <span className="text-[9.5px] font-bold text-[var(--color-kb-text-muted)] uppercase tracking-wider flex items-center gap-1.5">
              <Activity size={10} className="text-indigo-500 animate-pulse" />
              {t('coreSkills')}
            </span>

            <div className="grid grid-cols-1 gap-2">
              {/* Skill 1: generate_headlines */}
              <div className="flex items-start justify-between bg-[var(--color-kb-editor)] p-2.5 rounded-xl border border-[var(--color-kb-panel-border)] shadow-xs hover:border-indigo-500/20 transition-all">
                <div className="space-y-0.5">
                  <div className="text-[11.5px] font-bold text-[var(--color-kb-text-heading)] flex items-center gap-1">
                    <code className="text-indigo-600 dark:text-indigo-400 font-mono text-[10px]">generate_headlines</code>
                    <span className="text-[9px] font-medium text-slate-400 font-sans">{t('headlineOpt')}</span>
                  </div>
                  <p className="text-[10px] text-[var(--color-kb-text-muted)]">{t('headlineOptDesc')}</p>
                </div>
                <button 
                  type="button"
                  onClick={() => onTriggerQuickTool('帮我优化当前正文标题')}
                  disabled={isTyping}
                  className="px-2 py-1 text-[10px] font-bold text-white bg-indigo-500 hover:bg-indigo-600 rounded-lg shadow-sm transition-all flex items-center gap-0.5 disabled:opacity-50 cursor-pointer"
                >
                  <Play size={8} fill="currentColor" />
                  <span>{t('run')}</span>
                </button>
              </div>

              {/* Skill 2: insert_article_block */}
              <div className="flex items-start justify-between bg-[var(--color-kb-editor)] p-2.5 rounded-xl border border-[var(--color-kb-panel-border)] shadow-xs hover:border-emerald-500/20 transition-all">
                <div className="space-y-0.5">
                  <div className="text-[11.5px] font-bold text-[var(--color-kb-text-heading)] flex items-center gap-1">
                    <code className="text-emerald-600 dark:text-emerald-400 font-mono text-[10px]">insert_article_block</code>
                    <span className="text-[9px] font-medium text-slate-400 font-sans">{t('blockInject')}</span>
                  </div>
                  <p className="text-[10px] text-[var(--color-kb-text-muted)]">{t('blockInjectDesc')}</p>
                </div>
                <button 
                  type="button"
                  onClick={() => onTriggerQuickTool('在中间插入金句：“科技是第一生产力，它重塑了我们对星空的全部想象。”')}
                  disabled={isTyping}
                  className="px-2 py-1 text-[10px] font-bold text-white bg-[#07c160] hover:bg-emerald-600 rounded-lg shadow-sm transition-all flex items-center gap-0.5 disabled:opacity-50 cursor-pointer"
                >
                  <Play size={8} fill="currentColor" />
                  <span>{t('try')}</span>
                </button>
              </div>

              {/* Skill 3: run_editorial_diagnostic */}
              <div className="flex items-start justify-between bg-[var(--color-kb-editor)] p-2.5 rounded-xl border border-[var(--color-kb-panel-border)] shadow-xs hover:border-amber-500/20 transition-all">
                <div className="space-y-0.5">
                  <div className="text-[11.5px] font-bold text-[var(--color-kb-text-heading)] flex items-center gap-1">
                    <code className="text-amber-600 dark:text-amber-400 font-mono text-[10px]">run_editorial_diagnostic</code>
                    <span className="text-[9px] font-medium text-slate-400 font-sans">{t('qualityDiagnostic')}</span>
                  </div>
                  <p className="text-[10px] text-[var(--color-kb-text-muted)]">{t('qualityDiagnosticDesc')}</p>
                </div>
                <button 
                  type="button"
                  onClick={() => onTriggerQuickTool('帮我进行内容 and 排版诊断检查')}
                  disabled={isTyping}
                  className="px-2 py-1 text-[10px] font-bold text-white bg-amber-500 hover:bg-amber-600 rounded-lg shadow-sm transition-all flex items-center gap-0.5 disabled:opacity-50 cursor-pointer"
                >
                  <Play size={8} fill="currentColor" />
                  <span>{t('diagnose')}</span>
                </button>
              </div>

              {/* Skill 4: format_layout_styling */}
              <div className="flex items-start justify-between bg-[var(--color-kb-editor)] p-2.5 rounded-xl border border-[var(--color-kb-panel-border)] shadow-xs hover:border-blue-500/20 transition-all">
                <div className="space-y-0.5">
                  <div className="text-[11.5px] font-bold text-[var(--color-kb-text-heading)] flex items-center gap-1">
                    <code className="text-blue-600 dark:text-blue-400 font-mono text-[10px]">format_layout_styling</code>
                    <span className="text-[9px] font-medium text-slate-400 font-sans">{t('formatLayout')}</span>
                  </div>
                  <p className="text-[10px] text-[var(--color-kb-text-muted)]">{t('formatLayoutDesc')}</p>
                </div>
                <button 
                  type="button"
                  onClick={() => onTriggerQuickTool('一键排版经典绿')}
                  disabled={isTyping}
                  className="px-2 py-1 text-[10px] font-bold text-white bg-blue-500 hover:bg-blue-600 rounded-lg shadow-sm transition-all flex items-center gap-0.5 disabled:opacity-50 cursor-pointer"
                >
                  <Play size={8} fill="currentColor" />
                  <span>{t('restyle')}</span>
                </button>
              </div>
            </div>
          </div>

          <div className="bg-[var(--color-kb-editor)] rounded-xl p-2.5 border border-[var(--color-kb-panel-border)] font-mono text-[10px] text-[var(--color-kb-text-muted)] leading-relaxed space-y-1 select-none shadow-sm">
            <div className="flex justify-between text-[9px] border-b border-[var(--color-kb-panel-border)] pb-1 text-[var(--color-kb-text-muted)] font-bold">
              <span>{t('mcpLocalLogs')}</span>
              <span className="text-emerald-400 animate-pulse">{t('connectionEstablished')}</span>
            </div>
            <div>{t('mcpConnected')}</div>
            <div>{t('activeSelection')}</div>
            <div>{t('listeningTriggers')}</div>
          </div>
        </div>
      )}
    </div>
  );
}
