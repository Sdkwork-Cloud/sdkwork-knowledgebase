import React from 'react';
import { Bold, Italic, Strikethrough, List, ListOrdered, Undo, Redo, Code, Columns } from 'lucide-react';
import { InsertToolsMenu, InsertToolItem } from './InsertToolsMenu';
import { DocumentExportMenu } from './DocumentExport';
import type { DocumentExportContentProvider, DocumentExportFormat } from './DocumentExport';

export interface UniversalToolbarGroup {
  type: 'typography' | 'format' | 'list' | 'insert' | 'history' | 'view' | 'export';
  tools?: InsertToolItem[];
}

export interface UniversalToolbarProps {
  editor: any;
  t: (key: string) => string;
  config: UniversalToolbarGroup[];
  height?: number | string;

  handleToggleSourceMode?: () => void;
  isSourceMode?: boolean;
  isSplitMode?: boolean;
  setIsSplitMode?: (val: boolean) => void;
  getExportContent?: DocumentExportContentProvider;
  exportFormats?: DocumentExportFormat[];
}

export function UniversalToolbar({
  editor,
  t,
  config = [],
  height = 40,
  handleToggleSourceMode,
  isSourceMode,
  isSplitMode,
  setIsSplitMode,
  getExportContent,
  exportFormats,
}: UniversalToolbarProps) {
  if (!editor) return null;

  const hasExportGroup = config.some((group) => group.type === 'export');
  const mainGroups = config.filter((group) => group.type !== 'export');

  const renderGroupContent = (group: UniversalToolbarGroup): React.ReactNode => {
    switch (group.type) {
      case 'typography':
        return (
          <>
            <button
              type="button"
              onClick={() => editor.chain().focus().toggleBold().run()}
              className={`p-1 md:p-1.5 rounded-md transition-all ${
                editor.isActive('bold')
                  ? 'bg-[var(--color-kb-panel-active)] text-[var(--color-kb-accent)]'
                  : 'hover:bg-[var(--color-kb-panel-hover)] hover:text-[var(--color-kb-text-heading)]'
              }`}
              title={t('bold')}
            >
              <Bold size={14} />
            </button>
            <button
              type="button"
              onClick={() => editor.chain().focus().toggleItalic().run()}
              className={`p-1 md:p-1.5 rounded-md transition-all ${
                editor.isActive('italic')
                  ? 'bg-[var(--color-kb-panel-active)] text-[var(--color-kb-accent)]'
                  : 'hover:bg-[var(--color-kb-panel-hover)] hover:text-[var(--color-kb-text-heading)]'
              }`}
              title={t('italic')}
            >
              <Italic size={14} />
            </button>
            <button
              type="button"
              onClick={() => editor.chain().focus().toggleStrike().run()}
              className={`p-1 md:p-1.5 rounded-md transition-all ${
                editor.isActive('strike')
                  ? 'bg-[var(--color-kb-panel-active)] text-[var(--color-kb-accent)]'
                  : 'hover:bg-[var(--color-kb-panel-hover)] hover:text-[var(--color-kb-text-heading)]'
              }`}
              title={t('strikethrough')}
            >
              <Strikethrough size={14} />
            </button>
          </>
        );
      case 'format':
        return null;
      case 'list':
        return (
          <>
            <button
              type="button"
              onClick={() => editor.chain().focus().toggleBulletList().run()}
              className={`p-1 md:p-1.5 rounded-md transition-all ${
                editor.isActive('bulletList')
                  ? 'bg-[var(--color-kb-panel-active)] text-[var(--color-kb-accent)]'
                  : 'hover:bg-[var(--color-kb-panel-hover)] hover:text-[var(--color-kb-text-heading)]'
              }`}
              title={t('bulletList')}
            >
              <List size={14} />
            </button>
            <button
              type="button"
              onClick={() => editor.chain().focus().toggleOrderedList().run()}
              className={`p-1 md:p-1.5 rounded-md transition-all ${
                editor.isActive('orderedList')
                  ? 'bg-[var(--color-kb-panel-active)] text-[var(--color-kb-accent)]'
                  : 'hover:bg-[var(--color-kb-panel-hover)] hover:text-[var(--color-kb-text-heading)]'
              }`}
              title={t('orderedList')}
            >
              <ListOrdered size={14} />
            </button>
          </>
        );
      case 'insert':
        return group.tools ? <InsertToolsMenu tools={group.tools} /> : null;
      case 'history':
        return (
          <>
            <button
              type="button"
              onClick={() => editor.chain().focus().undo().run()}
              disabled={!editor.can().undo()}
              className="p-1 md:p-1.5 rounded-md transition-all hover:bg-[var(--color-kb-panel-hover)] hover:text-[var(--color-kb-text-heading)] disabled:opacity-30"
              title={t('undo')}
            >
              <Undo size={14} />
            </button>
            <button
              type="button"
              onClick={() => editor.chain().focus().redo().run()}
              disabled={!editor.can().redo()}
              className="p-1 md:p-1.5 rounded-md transition-all hover:bg-[var(--color-kb-panel-hover)] hover:text-[var(--color-kb-text-heading)] disabled:opacity-30"
              title={t('redo')}
            >
              <Redo size={14} />
            </button>
          </>
        );
      case 'view':
        return (
          <>
            <button
              type="button"
              onClick={handleToggleSourceMode}
              className={`p-1 md:p-1.5 px-2 py-1 rounded-md transition-all flex items-center gap-1 text-[11.5px] font-semibold border border-zinc-200/80 dark:border-zinc-800/80 ${
                isSourceMode
                  ? 'bg-[var(--color-kb-panel-active)] text-[var(--color-kb-accent)]'
                  : 'bg-white dark:bg-[var(--color-kb-panel)] hover:bg-[var(--color-kb-panel-hover)]'
              }`}
              title={t('sourceMode')}
            >
              <Code size={13} />
              <span className="hidden md:inline whitespace-nowrap">{t('sourceMode')}</span>
            </button>
            <button
              type="button"
              onClick={() => setIsSplitMode?.(!isSplitMode)}
              className={`p-1 md:p-1.5 px-2 py-1 rounded-md transition-all flex items-center gap-1 text-[11.5px] font-semibold border border-zinc-200/80 dark:border-zinc-800/80 ${
                isSplitMode
                  ? 'bg-[var(--color-kb-panel-active)] text-[var(--color-kb-accent)]'
                  : 'bg-white dark:bg-[var(--color-kb-panel)] hover:bg-[var(--color-kb-panel-hover)]'
              }`}
              title={t('splitMode')}
            >
              <Columns size={13} />
              <span className="hidden md:inline whitespace-nowrap">{t('splitMode')}</span>
            </button>
          </>
        );
      default:
        return null;
    }
  };

  const renderGroup = (group: UniversalToolbarGroup, groupIdx: number, groupCount: number) => {
    const content = renderGroupContent(group);
    if (!content) return null;

    return (
      <div key={`${group.type}-${groupIdx}`} className="flex items-center gap-0.5 md:gap-1 shrink-0">
        {content}
        {groupIdx < groupCount - 1 && (
          <div className="w-px h-4 bg-zinc-200/80 dark:bg-zinc-800/80 mx-0.5 md:mx-1 shrink-0" />
        )}
      </div>
    );
  };

  return (
    <div
      className="flex items-center flex-nowrap gap-1 md:gap-1 text-[var(--color-kb-text-muted)] w-full select-none overflow-visible shrink-0"
      style={{
        minHeight: typeof height === 'number' ? `${height}px` : height,
        height: typeof height === 'number' ? `${height}px` : height,
      }}
    >
      <div className="flex items-center flex-nowrap gap-1 md:gap-1 min-w-0 flex-1 overflow-visible">
        {mainGroups.map((group, groupIdx) => renderGroup(group, groupIdx, mainGroups.length))}
      </div>

      {hasExportGroup && getExportContent ? (
        <DocumentExportMenu
          getContent={getExportContent}
          formats={exportFormats}
          className="relative ml-auto shrink-0 export-dropdown-container z-30"
        />
      ) : null}
    </div>
  );
}
