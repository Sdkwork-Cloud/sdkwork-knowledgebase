export type DocumentExportFormat = 'pdf' | 'markdown' | 'word' | 'image';

export type DocumentExportSourceKind = 'markdown' | 'richtext';

export type DocumentPdfExportEngine = 'native-markdown' | 'native-webview' | 'canvas';

export type ExportSaveMode = 'downloads' | 'saveAs';

export interface SaveExportFileResult {
  saved: boolean;
  cancelled?: boolean;
  path?: string;
  pathLabel?: string;
  mode: ExportSaveMode;
}

export interface DocumentExportResult {
  save: SaveExportFileResult;
  pdfEngine?: DocumentPdfExportEngine;
  /** True only when a native engine was expected but canvas was used instead. */
  usedCanvasFallback?: boolean;
  imageLoadFailures?: number;
  usedTiledRender?: boolean;
  canvasMayBeClipped?: boolean;
}

export interface DocumentExportContent {
  title: string;
  html: string;
  markdown?: string;
  sourceKind?: DocumentExportSourceKind;
}

export type DocumentExportContentProvider = () => DocumentExportContent | null;
