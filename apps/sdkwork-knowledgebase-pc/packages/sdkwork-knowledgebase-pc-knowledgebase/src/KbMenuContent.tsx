import type { TFunction } from 'i18next';
import { Cloud, Edit2, GitBranch, Settings, Share, Trash2 } from 'lucide-react';
import { ContextMenuItem, ContextMenuSeparator } from './components/ui/context-menu';
import { DropdownMenuItem, DropdownMenuSeparator } from './components/ui/dropdown-menu';

export interface KbMenuItemsProps {
  onRename: (event: Event) => void;
  onDelete: (event: Event) => void;
  onOpenSettings: (event: Event) => void;
  onImportGit?: (event: Event) => void;
  onSyncGit?: (event: Event) => void;
  onImportCloudDrive?: (event: Event) => void;
  t: TFunction<'kb' | 'common', undefined>;
}

export function KbDropdownItems({
  onRename,
  onDelete,
  onOpenSettings,
  onImportGit,
  onSyncGit,
  onImportCloudDrive,
  t,
}: KbMenuItemsProps) {
  return (
    <>
      <DropdownMenuItem onSelect={onOpenSettings}>
        <Settings size={14} className="mr-2 text-emerald-600 dark:text-emerald-500" />
        {t('settings', { defaultValue: '知识库设置' })}
      </DropdownMenuItem>
      <DropdownMenuItem onSelect={onRename}>
        <Edit2 size={14} className="mr-2 text-gray-500" />
        {t('rename', { ns: 'common' })}
      </DropdownMenuItem>

      <DropdownMenuSeparator className="bg-[var(--color-kb-panel-border)]/60 my-1" />

      <DropdownMenuItem onSelect={onImportCloudDrive}>
        <Cloud size={14} className="mr-2 text-amber-600 dark:text-amber-400" />
        {t('importFromCloudDrive', { defaultValue: '从网盘中导入...' })}
      </DropdownMenuItem>
      <DropdownMenuItem onSelect={onImportGit}>
        <GitBranch size={14} className="mr-2 text-teal-600 dark:text-teal-400" />
        {t('importFromGit', { defaultValue: '从 Git 仓库导入...' })}
      </DropdownMenuItem>
      <DropdownMenuItem onSelect={onSyncGit}>
        <Share size={14} className="mr-2 text-blue-500" />
        {t('syncToGit', { defaultValue: '同步仓库到 Git...' })}
      </DropdownMenuItem>

      <DropdownMenuSeparator className="bg-[var(--color-kb-panel-border)]/60 my-1" />

      <DropdownMenuItem onSelect={onDelete} className="text-red-500 focus:text-red-500">
        <Trash2 size={14} className="mr-2" />
        {t('delete', { ns: 'common' })}
      </DropdownMenuItem>
    </>
  );
}

export function KbContextItems({
  onRename,
  onDelete,
  onOpenSettings,
  onImportGit,
  onSyncGit,
  onImportCloudDrive,
  t,
}: KbMenuItemsProps) {
  return (
    <>
      <ContextMenuItem onSelect={onOpenSettings}>
        <Settings size={14} className="mr-2 text-emerald-600 dark:text-emerald-500" />
        {t('settings', { defaultValue: '知识库设置' })}
      </ContextMenuItem>
      <ContextMenuItem onSelect={onRename}>
        <Edit2 size={14} className="mr-2 text-gray-500" />
        {t('rename', { ns: 'common' })}
      </ContextMenuItem>

      <ContextMenuSeparator className="bg-[var(--color-kb-panel-border)]/60 my-1" />

      <ContextMenuItem onSelect={onImportCloudDrive}>
        <Cloud size={14} className="mr-2 text-amber-600 dark:text-amber-400" />
        {t('importFromCloudDrive', { defaultValue: '从网盘中导入...' })}
      </ContextMenuItem>
      <ContextMenuItem onSelect={onImportGit}>
        <GitBranch size={14} className="mr-2 text-teal-600 dark:text-teal-400" />
        {t('importFromGit', { defaultValue: '从 Git 仓库导入...' })}
      </ContextMenuItem>
      <ContextMenuItem onSelect={onSyncGit}>
        <Share size={14} className="mr-2 text-blue-500" />
        {t('syncToGit', { defaultValue: '同步仓库到 Git...' })}
      </ContextMenuItem>

      <ContextMenuSeparator className="bg-[var(--color-kb-panel-border)]/60 my-1" />

      <ContextMenuItem onSelect={onDelete} className="text-red-500 focus:text-red-500">
        <Trash2 size={14} className="mr-2" />
        {t('delete', { ns: 'common' })}
      </ContextMenuItem>
    </>
  );
}
