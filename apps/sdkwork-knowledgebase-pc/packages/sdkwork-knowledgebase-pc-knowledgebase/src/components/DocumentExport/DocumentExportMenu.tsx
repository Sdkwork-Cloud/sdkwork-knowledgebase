import React, { useEffect, useMemo, useState } from 'react';
import { ChevronDown, FileDown } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { DropdownItem } from '../InsertToolsMenu';
import { getDocumentExportCapabilities } from './documentExportCapabilities';
import { hasExportableContent } from './exportContentUtils';
import { useDocumentExport } from './useDocumentExport';
import type { DocumentExportContentProvider, DocumentExportFormat, ExportSaveMode } from './types';

const DEFAULT_FORMATS: DocumentExportFormat[] = ['pdf', 'markdown', 'word', 'image'];

const FORMAT_OPTIONS: Record<
  DocumentExportFormat,
  { labelKey: string; saveAsLabelKey: string; badge: string; badgeClass: string }
> = {
  pdf: {
    labelKey: 'exportAsPdf',
    saveAsLabelKey: 'saveAsPdf',
    badge: 'PDF',
    badgeClass:
      'bg-rose-50 text-rose-600 dark:bg-rose-950/40 dark:text-rose-400 border-rose-100 dark:border-rose-900/40',
  },
  markdown: {
    labelKey: 'exportAsMarkdown',
    saveAsLabelKey: 'saveAsMarkdown',
    badge: 'MD',
    badgeClass:
      'bg-emerald-50 text-emerald-600 dark:bg-emerald-950/40 dark:text-emerald-400 border-emerald-100 dark:border-emerald-900/40',
  },
  word: {
    labelKey: 'exportAsWord',
    saveAsLabelKey: 'saveAsWord',
    badge: 'DOC',
    badgeClass:
      'bg-blue-50 text-blue-600 dark:bg-blue-950/40 dark:text-blue-400 border-blue-100 dark:border-blue-900/40',
  },
  image: {
    labelKey: 'exportAsImage',
    saveAsLabelKey: 'saveAsImage',
    badge: 'PNG',
    badgeClass:
      'bg-orange-50 text-orange-600 dark:bg-orange-950/40 dark:text-orange-400 border-orange-100 dark:border-orange-900/40',
  },
};

export interface DocumentExportMenuProps {
  getContent: DocumentExportContentProvider;
  formats?: DocumentExportFormat[];
  className?: string;
  disabled?: boolean;
}

export function DocumentExportMenu({
  getContent,
  formats = DEFAULT_FORMATS,
  className = 'relative ml-auto shrink-0 export-dropdown-container z-30',
  disabled = false,
}: DocumentExportMenuProps) {
  const { t } = useTranslation('editor');
  const [isOpen, setIsOpen] = useState(false);
  const { exportByFormat, isExporting } = useDocumentExport({ getContent });

  const visibleFormats = useMemo(() => {
    const content = getContent();
    if (content?.sourceKind === 'richtext') {
      return formats.filter((format) => format !== 'markdown');
    }
    return formats;
  }, [formats, getContent, isOpen]);

  const capabilities = useMemo(
    () => getDocumentExportCapabilities(getContent()?.sourceKind),
    [getContent, isOpen],
  );

  const contentReady = useMemo(() => {
    const content = getContent();
    if (!content) {
      return false;
    }
    return hasExportableContent({
      title: content.title,
      html: content.html,
      markdown: content.markdown,
      sourceKind: content.sourceKind,
    });
  }, [getContent, isOpen]);

  const pdfEngineDescription = capabilities.pdfEngineDescription;

  useEffect(() => {
    const handleOutsideClick = (event: MouseEvent) => {
      const target = event.target as HTMLElement;
      if (!target.closest('.export-dropdown-container')) {
        setIsOpen(false);
      }
    };
    document.addEventListener('click', handleOutsideClick);
    return () => document.removeEventListener('click', handleOutsideClick);
  }, []);

  useEffect(() => {
    if (!isOpen) {
      return;
    }
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        setIsOpen(false);
      }
    };
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [isOpen]);

  const handleSelect = async (format: DocumentExportFormat, mode: ExportSaveMode) => {
    setIsOpen(false);
    await exportByFormat(format, mode);
  };

  const renderFormatItem = (
    format: DocumentExportFormat,
    mode: ExportSaveMode,
    label: string,
  ) => {
    const option = FORMAT_OPTIONS[format];
    return (
      <DropdownItem
        text={label}
        disabled={isExporting}
        icon={
          <span
            className={`w-9 h-5 shrink-0 text-[10px] font-bold rounded flex items-center justify-center border ${option.badgeClass}`}
          >
            {option.badge}
          </span>
        }
        onClick={() => {
          void handleSelect(format, mode);
        }}
      />
    );
  };

  return (
    <div className={className}>
      <button
        type="button"
        disabled={disabled || isExporting || !contentReady}
        onClick={(event) => {
          event.stopPropagation();
          if (!contentReady) {
            return;
          }
          setIsOpen(!isOpen);
        }}
        className={`p-1 md:p-1.5 px-2.5 py-1 md:px-2.5 md:py-1 rounded-md transition-all flex items-center gap-1 font-semibold text-[11.5px] border border-zinc-200/80 dark:border-zinc-800/80 shadow-[0_1px_2px_rgba(0,0,0,0.02)] disabled:opacity-50 disabled:cursor-not-allowed ${
          isOpen
            ? 'bg-[var(--color-kb-panel-active)] text-[var(--color-kb-accent)]'
            : 'bg-white dark:bg-[var(--color-kb-panel)] text-[var(--color-kb-text)] hover:bg-[var(--color-kb-panel-hover)] hover:text-[var(--color-kb-text-heading)]'
        }`}
        title={
          contentReady
            ? `${capabilities.platformLabel} · ${pdfEngineDescription}`
            : t('noExportableContent')
        }
      >
        <FileDown size={13} className="opacity-80 shrink-0" />
        <span className="whitespace-nowrap">{isExporting ? t('exporting') : t('export')}</span>
        <ChevronDown
          size={11}
          className={`shrink-0 opacity-60 transition-transform duration-200 ${isOpen ? 'rotate-180' : ''}`}
        />
      </button>

      {isOpen && (
        <div className="absolute right-0 top-[28px] md:top-[32px] w-max whitespace-nowrap bg-white dark:bg-[var(--color-kb-editor)] border border-zinc-200/80 dark:border-[var(--color-kb-panel-border)] shadow-[0_4px_16px_rgba(0,0,0,0.08)] dark:shadow-[0_4px_24px_rgba(0,0,0,0.4)] rounded-xl py-1 z-[310] animate-in fade-in slide-in-from-top-2 duration-150">
          {visibleFormats.map((format) => (
            <React.Fragment key={`dl-${format}`}>
              {renderFormatItem(format, 'downloads', t(FORMAT_OPTIONS[format].labelKey))}
            </React.Fragment>
          ))}

          {capabilities.saveAsAvailable && (
            <>
              <div className="my-1 mx-2 border-t border-zinc-100 dark:border-zinc-800/80" />

              <div className="px-3 py-1 text-[10px] font-semibold uppercase tracking-wide text-zinc-400 dark:text-zinc-500 whitespace-nowrap">
                {t('saveAsSection')}
              </div>
              {visibleFormats.map((format) => (
                <React.Fragment key={`sa-${format}`}>
                  {renderFormatItem(format, 'saveAs', t(FORMAT_OPTIONS[format].saveAsLabelKey))}
                </React.Fragment>
              ))}
            </>
          )}
        </div>
      )}
    </div>
  );
}
