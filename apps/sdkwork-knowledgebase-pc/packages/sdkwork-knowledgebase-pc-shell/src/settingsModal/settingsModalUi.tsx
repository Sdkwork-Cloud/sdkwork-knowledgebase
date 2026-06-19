import React, { useEffect, useState } from 'react';
import { Check, Copy } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { toast } from '@packages/sdkwork-knowledgebase-pc-knowledgebase/src';

import { SETTINGS_APP_DISPLAY_NAME } from '../settingsModalConstants';

export function SidebarSection({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div>
      <div className="mb-2 px-3 text-[10px] font-bold uppercase tracking-widest text-zinc-400 dark:text-[var(--color-kb-text-muted)]">
        {title}
      </div>
      <div className="space-y-1">{children}</div>
    </div>
  );
}

export function SettingsCard({
  title,
  children,
  variant = 'default',
}: {
  title?: string;
  children: React.ReactNode;
  variant?: 'default' | 'danger';
}) {
  const borderClass =
    variant === 'danger'
      ? 'border-rose-200/80 dark:border-rose-900/40'
      : 'border-zinc-200/80 dark:border-[var(--color-kb-panel-border)]';

  return (
    <section className={`rounded-2xl border ${borderClass} bg-white p-5 shadow-sm dark:bg-[var(--color-kb-panel)]/35`}>
      {title ? (
        <h3 className="mb-4 text-[13px] font-bold text-zinc-800 dark:text-[var(--color-kb-text-heading)]">
          {title}
        </h3>
      ) : null}
      {children}
    </section>
  );
}

export function SettingsRow({
  label,
  description,
  control,
  inline = false,
}: {
  label: string;
  description?: string;
  control: React.ReactNode;
  inline?: boolean;
}) {
  if (inline) {
    return (
      <div className="flex items-center justify-between gap-6 py-3 first:pt-0 last:pb-0">
        <div className="min-w-0">
          <div className="text-sm font-semibold text-zinc-800 dark:text-[var(--color-kb-text)]">{label}</div>
          {description ? (
            <div className="mt-0.5 text-xs text-zinc-500 dark:text-[var(--color-kb-text-muted)]">{description}</div>
          ) : null}
        </div>
        {control}
      </div>
    );
  }

  return (
    <div className="py-1">
      <div className="mb-3 flex items-center justify-between gap-4">
        <div className="text-sm font-semibold text-zinc-800 dark:text-[var(--color-kb-text)]">{label}</div>
        {control}
      </div>
      {description ? (
        <p className="text-xs text-zinc-500 dark:text-[var(--color-kb-text-muted)]">{description}</p>
      ) : null}
    </div>
  );
}

export function InfoRow({
  label,
  value,
  mono = false,
  copyable = false,
}: {
  label: string;
  value: string;
  mono?: boolean;
  copyable?: boolean;
}) {
  const { t } = useTranslation('shell');
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    if (!value || value === '—') return;
    try {
      await navigator.clipboard.writeText(value);
      setCopied(true);
      toast.success(t('copiedToClipboard'));
      window.setTimeout(() => setCopied(false), 1500);
    } catch {
      toast.error(t('diagnosticsCopyFailed'));
    }
  };

  return (
    <div className="flex items-start justify-between gap-4 py-3 first:pt-0 last:pb-0 text-sm">
      <span className="text-zinc-500 dark:text-[var(--color-kb-text-muted)]">{label}</span>
      <div className="flex max-w-[58%] items-start justify-end gap-2">
        <span
          className={`text-right font-medium text-zinc-900 dark:text-[var(--color-kb-text-heading)] break-all ${mono ? 'font-mono text-xs' : ''}`}
        >
          {value}
        </span>
        {copyable && value !== '—' ? (
          <button
            type="button"
            onClick={() => void handleCopy()}
            title={t('copyToClipboard')}
            className="shrink-0 rounded-md p-1 text-zinc-400 transition-colors hover:bg-zinc-100 hover:text-zinc-700 dark:hover:bg-[var(--color-kb-panel-hover)] dark:hover:text-[var(--color-kb-text)]"
          >
            {copied ? <Check size={13} className="text-emerald-500" /> : <Copy size={13} />}
          </button>
        ) : null}
      </div>
    </div>
  );
}

export function SegmentedControl<T extends string>({
  options,
  value,
  onChange,
}: {
  options: Array<{ value: T; label: string }>;
  value: T;
  onChange: (value: T) => void;
}) {
  return (
    <div className="inline-flex items-center rounded-xl border border-zinc-200 bg-zinc-50 p-1 dark:border-[var(--color-kb-panel-border)] dark:bg-[var(--color-kb-panel)]">
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          onClick={() => onChange(option.value)}
          className={`rounded-lg px-3 py-1.5 text-xs font-semibold transition-all ${
            value === option.value
              ? 'bg-white text-[var(--color-kb-accent)] shadow-sm dark:bg-[var(--color-kb-panel-active)]'
              : 'text-zinc-600 hover:text-zinc-900 dark:text-[var(--color-kb-text)] dark:hover:text-[var(--color-kb-text-heading)]'
          }`}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}

export function TabButton({
  active,
  icon,
  label,
  onClick,
}: {
  active: boolean;
  icon: React.ReactNode;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      data-settings-nav-active={active ? 'true' : undefined}
      aria-current={active ? 'page' : undefined}
      className={`flex w-full items-center gap-2.5 rounded-xl px-3 py-2.5 text-[13px] font-semibold transition-all ${
        active
          ? 'bg-zinc-900 text-white shadow-md dark:bg-[var(--color-kb-accent)]'
          : 'text-zinc-500 hover:bg-black/5 hover:text-zinc-900 dark:text-[var(--color-kb-text-muted)] dark:hover:bg-[var(--color-kb-panel-hover)] dark:hover:text-[var(--color-kb-text-heading)]'
      }`}
    >
      {icon}
      <span>{label}</span>
    </button>
  );
}

export function ThemeCard({
  active,
  type,
  label,
  onClick,
}: {
  active: boolean;
  type: 'light' | 'dark' | 'system';
  label: string;
  onClick: () => void;
}) {
  return (
    <button type="button" onClick={onClick} className="group flex flex-col items-center text-left">
      <div
        className={`mb-2.5 w-full rounded-xl border-2 p-1.5 transition-all ${
          active
            ? 'border-[var(--color-kb-accent)] shadow-md ring-4 ring-[var(--color-kb-accent)]/10'
            : 'border-zinc-200 group-hover:border-zinc-300 dark:border-[var(--color-kb-panel-border)]'
        }`}
      >
        {type === 'light' && (
          <div className="flex aspect-[4/3] w-full flex-col overflow-hidden rounded-lg bg-[#f8f9fa] shadow-inner">
            <div className="h-3 border-b border-gray-200 bg-white" />
            <div className="flex flex-1">
              <div className="w-1/3 border-r border-gray-200 bg-[#f3f4f6]" />
              <div className="flex-1 bg-white p-1">
                <div className="mb-1 h-1 w-3/4 rounded-sm bg-gray-200" />
                <div className="h-1 w-1/2 rounded-sm bg-gray-200" />
              </div>
            </div>
          </div>
        )}
        {type === 'dark' && (
          <div className="flex aspect-[4/3] w-full flex-col overflow-hidden rounded-lg bg-[#1b1d22] shadow-inner">
            <div className="h-3 border-b border-[#26282e] bg-[#15171a]" />
            <div className="flex flex-1">
              <div className="w-1/3 border-r border-[#26282e] bg-[#1a1c20]" />
              <div className="flex-1 bg-[#1b1d22] p-1">
                <div className="mb-1 h-1 w-3/4 rounded-sm bg-[#2a2c32]" />
                <div className="h-1 w-1/2 rounded-sm bg-[#2a2c32]" />
              </div>
            </div>
          </div>
        )}
        {type === 'system' && (
          <div className="relative flex aspect-[4/3] w-full overflow-hidden rounded-lg shadow-inner">
            <div className="absolute inset-0 flex flex-col bg-[#f8f9fa]">
              <div className="h-3 border-b border-gray-200 bg-white" />
              <div className="flex flex-1">
                <div className="w-1/3 border-r border-gray-200 bg-[#f3f4f6]" />
                <div className="flex-1 bg-white" />
              </div>
            </div>
            <div
              className="absolute inset-0 flex flex-col bg-[#1b1d22]"
              style={{ clipPath: 'polygon(100% 0, 100% 100%, 0 100%)' }}
            >
              <div className="h-3 border-b border-[#26282e] bg-[#15171a]" />
              <div className="flex flex-1">
                <div className="w-1/3 border-r border-[#26282e] bg-[#1a1c20]" />
                <div className="flex-1 bg-[#1b1d22]" />
              </div>
            </div>
          </div>
        )}
      </div>
      <span
        className={`text-sm ${active ? 'font-semibold text-zinc-900 dark:text-[var(--color-kb-text-heading)]' : 'text-zinc-600 dark:text-[var(--color-kb-text)]'}`}
      >
        {label}
      </span>
    </button>
  );
}

export function ToggleSwitch({
  active,
  disabled = false,
  onChange,
}: {
  active: boolean;
  disabled?: boolean;
  onChange: (value: boolean) => void;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={active}
      disabled={disabled}
      onClick={() => onChange(!active)}
      className={`h-6 w-11 rounded-full p-1 transition-colors ${
        disabled ? 'cursor-not-allowed opacity-50' : 'cursor-pointer'
      } ${active ? 'bg-[var(--color-kb-accent)]' : 'bg-zinc-200 dark:bg-zinc-700'}`}
    >
      <div
        className={`h-4 w-4 rounded-full bg-white shadow-sm transition-transform ${active ? 'translate-x-5' : 'translate-x-0'}`}
      />
    </button>
  );
}

export function SettingsEmptyFilterState() {
  const { t } = useTranslation('shell');
  return (
    <div className="rounded-2xl border border-dashed border-zinc-200 px-6 py-10 text-center text-sm text-zinc-500 dark:border-[var(--color-kb-panel-border)] dark:text-[var(--color-kb-text-muted)]">
      {t('settingsPanelSearchEmpty')}
    </div>
  );
}

export function AppearancePreview({
  theme,
  accentColor,
  fontSize,
}: {
  theme: 'light' | 'dark' | 'system';
  accentColor: string;
  fontSize: 'small' | 'normal' | 'large';
}) {
  const { t } = useTranslation('shell');
  const [systemDark, setSystemDark] = useState(() =>
    typeof window !== 'undefined'
      ? window.matchMedia('(prefers-color-scheme: dark)').matches
      : false,
  );

  useEffect(() => {
    if (theme !== 'system' || typeof window === 'undefined') {
      return undefined;
    }

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const onChange = () => setSystemDark(mediaQuery.matches);
    mediaQuery.addEventListener('change', onChange);
    return () => mediaQuery.removeEventListener('change', onChange);
  }, [theme]);

  const previewSize = fontSize === 'small' ? '13px' : fontSize === 'large' ? '16px' : '14px';
  const isDark = theme === 'dark' || (theme === 'system' && systemDark);

  return (
    <div
      className={`overflow-hidden rounded-xl border ${isDark ? 'border-zinc-700 bg-[#121316]' : 'border-zinc-200 bg-zinc-50'}`}
      style={{ fontSize: previewSize }}
    >
      <div
        className={`flex items-center gap-2 border-b px-3 py-2 ${isDark ? 'border-zinc-800 bg-[#0c0c0e]' : 'border-zinc-200 bg-white'}`}
      >
        <div className="h-2.5 w-2.5 rounded-full" style={{ backgroundColor: accentColor }} />
        <span className={`text-xs font-semibold ${isDark ? 'text-zinc-200' : 'text-zinc-700'}`}>
          {SETTINGS_APP_DISPLAY_NAME}
        </span>
      </div>
      <div className="flex p-3">
        <div
          className={`mr-3 h-16 w-14 rounded-lg ${isDark ? 'bg-zinc-800' : 'bg-white border border-zinc-200'}`}
        />
        <div className="flex-1 space-y-2">
          <div className="h-2 w-2/3 rounded-full" style={{ backgroundColor: accentColor, opacity: 0.85 }} />
          <div className={`h-2 w-full rounded-full ${isDark ? 'bg-zinc-800' : 'bg-zinc-200'}`} />
          <div className={`h-2 w-5/6 rounded-full ${isDark ? 'bg-zinc-800' : 'bg-zinc-200'}`} />
          <div className="inline-flex rounded-md px-2 py-0.5 text-[10px] font-semibold text-white" style={{ backgroundColor: accentColor }}>
            {t('appearancePreviewAction')}
          </div>
        </div>
      </div>
    </div>
  );
}
