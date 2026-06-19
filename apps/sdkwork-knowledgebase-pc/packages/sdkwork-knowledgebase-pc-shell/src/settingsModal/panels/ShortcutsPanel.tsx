import React from 'react';
import { useTranslation } from 'react-i18next';

import { fieldMatchesSettingsQuery } from '../../settingsPreferences';
import { SettingsCard, SettingsEmptyFilterState } from '../settingsModalUi';

export function ShortcutsPanel({ filterQuery }: { filterQuery: string }) {
  const { t } = useTranslation('shell');
  const mod = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform) ? '⌘' : 'Ctrl';

  const showGlobal = fieldMatchesSettingsQuery(
    filterQuery,
    'shortcutsGlobal',
    ['global', 'settings', 'esc', '全局', '设置'],
    t,
  );
  const showSearch = fieldMatchesSettingsQuery(
    filterQuery,
    'shortcutsSearch',
    ['search', 'message', 'enter', '搜索', '发送'],
    t,
  );

  const itemMatchesQuery = (action: string, keys: string[]) => {
    if (!filterQuery) {
      return true;
    }
    const normalized = filterQuery.toLowerCase();
    return (
      action.toLowerCase().includes(normalized)
      || keys.some((key) => key.toLowerCase().includes(normalized))
    );
  };

  const groups = [
    {
      id: 'global' as const,
      title: t('shortcutsGlobal'),
      visible: showGlobal,
      items: [
        { action: t('shortcutOpenSettings'), keys: [mod, ','] },
        { action: t('shortcutCloseDialog'), keys: ['Esc'] },
      ],
    },
    {
      id: 'search' as const,
      title: t('shortcutsSearch'),
      visible: showSearch,
      items: [
        { action: t('shortcutNewSearchSession'), keys: [mod, 'N'] },
        { action: t('shortcutSendMessage'), keys: ['Enter'] },
        { action: t('shortcutConfirmRename'), keys: ['Enter'] },
        { action: t('shortcutCancelRename'), keys: ['Esc'] },
      ],
    },
  ];

  const visibleGroups = groups
    .filter((group) => group.visible)
    .map((group) => ({
      ...group,
      items: group.items.filter((item) => itemMatchesQuery(item.action, item.keys)),
    }))
    .filter((group) => group.items.length > 0);

  return (
    <div className="mx-auto max-w-2xl space-y-6">
      {visibleGroups.map((group) => (
        <SettingsCard key={group.id} title={group.title}>
          <div className="divide-y divide-zinc-100 dark:divide-[var(--color-kb-panel-border)]">
            {group.items.map((item) => (
              <div key={item.action} className="flex items-center justify-between gap-4 py-3 first:pt-0 last:pb-0">
                <span className="text-sm font-medium text-zinc-800 dark:text-[var(--color-kb-text)]">
                  {item.action}
                </span>
                <div className="flex shrink-0 items-center gap-1.5">
                  {item.keys.map((key, index) => (
                    <React.Fragment key={`${item.action}-${key}-${index}`}>
                      {index > 0 ? <span className="text-xs text-zinc-400">+</span> : null}
                      <kbd className="rounded-md border border-zinc-200 bg-zinc-50 px-2 py-1 text-[11px] font-semibold text-zinc-600 shadow-sm dark:border-[var(--color-kb-panel-border)] dark:bg-[var(--color-kb-panel)] dark:text-[var(--color-kb-text)]">
                        {key}
                      </kbd>
                    </React.Fragment>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </SettingsCard>
      ))}

      {visibleGroups.length === 0 && filterQuery ? <SettingsEmptyFilterState /> : null}
    </div>
  );
}
