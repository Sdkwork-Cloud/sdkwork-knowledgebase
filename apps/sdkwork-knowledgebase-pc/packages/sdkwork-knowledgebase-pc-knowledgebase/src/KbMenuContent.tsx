import React from 'react';
import { Edit2, Trash2, Globe, Settings, GitBranch, Share, Cloud } from 'lucide-react';
import { DropdownMenuItem, DropdownMenuSeparator } from './components/ui/dropdown-menu';
import { ContextMenuItem, ContextMenuSeparator } from './components/ui/context-menu';

export interface KbMenuItemsProps {
  onRename: (e: React.MouseEvent) => void;
  onDelete: (e: React.MouseEvent) => void;
  onDeploy: (e: React.MouseEvent) => void;
  onOpenSettings: (e: React.MouseEvent) => void;
  onImportGit?: (e: React.MouseEvent) => void;
  onSyncGit?: (e: React.MouseEvent) => void;
  onImportCloudDrive?: (e: React.MouseEvent) => void;
  t: (key: string, options?: any) => string;
}

export function KbDropdownItems({ onRename, onDelete, onDeploy, onOpenSettings, onImportGit, onSyncGit, onImportCloudDrive, t }: KbMenuItemsProps) {
  return (
    <>
      <DropdownMenuItem onSelect={onOpenSettings as any}>
        <Settings size={14} className="mr-2 text-emerald-600 dark:text-emerald-500" />
        {t('settings', { defaultValue: '知识库设置' })}
      </DropdownMenuItem>
      <DropdownMenuItem onSelect={onRename as any}>
        <Edit2 size={14} className="mr-2 text-gray-500" />
        {t('rename', { ns: 'common' })}
      </DropdownMenuItem>
      <DropdownMenuItem onSelect={onDeploy as any}>
        <Globe size={14} className="mr-2 text-indigo-500" />
        {t('deployAsWebsite', { defaultValue: '部署为网站' })}
      </DropdownMenuItem>
      
      <DropdownMenuSeparator className="bg-[var(--color-kb-panel-border)]/60 my-1" />
      
      <DropdownMenuItem onSelect={onImportCloudDrive as any}>
        <Cloud size={14} className="mr-2 text-amber-600 dark:text-amber-400" />
        {t('importFromCloudDrive', { defaultValue: '从网盘中导入...' })}
      </DropdownMenuItem>
      <DropdownMenuItem onSelect={onImportGit as any}>
        <GitBranch size={14} className="mr-2 text-teal-600 dark:text-teal-400" />
        {t('importFromGit', { defaultValue: '从 Git 仓库导入...' })}
      </DropdownMenuItem>
      <DropdownMenuItem onSelect={onSyncGit as any}>
        <Share size={14} className="mr-2 text-blue-500" />
        {t('syncToGit', { defaultValue: '同步仓库到 Git...' })}
      </DropdownMenuItem>
      
      <DropdownMenuSeparator className="bg-[var(--color-kb-panel-border)]/60 my-1" />

      <DropdownMenuItem onSelect={onDelete as any} className="text-red-500 focus:text-red-500">
        <Trash2 size={14} className="mr-2" />
        {t('delete', { ns: 'common' })}
      </DropdownMenuItem>
    </>
  );
}

export function KbContextItems({ onRename, onDelete, onDeploy, onOpenSettings, onImportGit, onSyncGit, onImportCloudDrive, t }: KbMenuItemsProps) {
  return (
    <>
      <ContextMenuItem onSelect={onOpenSettings as any}>
        <Settings size={14} className="mr-2 text-emerald-600 dark:text-emerald-500" />
        {t('settings', { defaultValue: '知识库设置' })}
      </ContextMenuItem>
      <ContextMenuItem onSelect={onRename as any}>
        <Edit2 size={14} className="mr-2 text-gray-500" />
        {t('rename', { ns: 'common' })}
      </ContextMenuItem>
      <ContextMenuItem onSelect={onDeploy as any}>
        <Globe size={14} className="mr-2 text-indigo-500" />
        {t('deployAsWebsite', { defaultValue: '部署为网站' })}
      </ContextMenuItem>

      <ContextMenuSeparator className="bg-[var(--color-kb-panel-border)]/60 my-1" />

      <ContextMenuItem onSelect={onImportCloudDrive as any}>
        <Cloud size={14} className="mr-2 text-amber-600 dark:text-amber-400" />
        {t('importFromCloudDrive', { defaultValue: '从网盘中导入...' })}
      </ContextMenuItem>
      <ContextMenuItem onSelect={onImportGit as any}>
        <GitBranch size={14} className="mr-2 text-teal-600 dark:text-teal-400" />
        {t('importFromGit', { defaultValue: '从 Git 仓库导入...' })}
      </ContextMenuItem>
      <ContextMenuItem onSelect={onSyncGit as any}>
        <Share size={14} className="mr-2 text-blue-500" />
        {t('syncToGit', { defaultValue: '同步仓库到 Git...' })}
      </ContextMenuItem>

      <ContextMenuSeparator className="bg-[var(--color-kb-panel-border)]/60 my-1" />

      <ContextMenuItem onSelect={onDelete as any} className="text-red-500 focus:text-red-500">
        <Trash2 size={14} className="mr-2" />
        {t('delete', { ns: 'common' })}
      </ContextMenuItem>
    </>
  );
}
