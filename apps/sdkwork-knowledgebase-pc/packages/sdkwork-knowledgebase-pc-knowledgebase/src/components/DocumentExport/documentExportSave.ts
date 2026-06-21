import { toast } from '../ui/toast-manager';
import { isBlank, trim } from '@sdkwork/sdkwork-knowledgebase-pc-commons/stringUtils';
import {
  detectOperatingSystem,
  encodeBytesBase64,
  getExportRuntimeEnvironment,
  invokeTauriCommand,
  isDesktopExportHost,
  isSaveFilePickerAvailable,
} from './exportRuntime';
import type { ExportSaveMode, SaveExportFileResult } from './types';

interface NativeSaveExportFileResult {
  saved: boolean;
  cancelled: boolean;
  path?: string | null;
  mode: ExportSaveMode;
}

interface HostExportAdapter {
  isNativeHost?: boolean;
  saveExportFile?: (options: {
    suggestedName: string;
    bytes: Uint8Array;
    mode: ExportSaveMode;
  }) => Promise<{
    saved: boolean;
    cancelled: boolean;
    path?: string | null;
    mode: ExportSaveMode;
  }>;
  revealExportFile?: (path: string) => Promise<void>;
  openExportFile?: (path: string) => Promise<void>;
  locateExportFile?: (fileName: string) => Promise<string | null | undefined>;
}

async function loadHostAdapter(): Promise<HostExportAdapter | null> {
  try {
    const core = await import('@packages/sdkwork-knowledgebase-pc-core/src');
    if (typeof core.createHostAdapter === 'function') {
      return core.createHostAdapter() as HostExportAdapter;
    }
  } catch {
    // pc-core may be absent in partial workspaces; Tauri fallback handles desktop saves.
  }
  return null;
}

function normalizeSaveResponse(
  response: NativeSaveExportFileResult,
  suggestedName: string,
): SaveExportFileResult {
  return {
    saved: response.saved,
    cancelled: response.cancelled,
    path: response.path ?? undefined,
    pathLabel: response.path
      ? response.path.split(/[/\\]/).pop() ?? suggestedName
      : suggestedName,
    mode: response.mode,
  };
}

async function saveViaNativeHost(options: {
  bytes: Uint8Array;
  suggestedName: string;
  mode: ExportSaveMode;
}): Promise<SaveExportFileResult | null> {
  if (!isDesktopExportHost()) {
    return null;
  }

  const host = await loadHostAdapter();
  if (host?.isNativeHost && typeof host.saveExportFile === 'function') {
    try {
      const response = await host.saveExportFile({
        suggestedName: options.suggestedName,
        bytes: options.bytes,
        mode: options.mode,
      });
      return normalizeSaveResponse(response, options.suggestedName);
    } catch (error) {
      console.warn('[DocumentExport] host saveExportFile failed, trying Tauri fallback', error);
    }
  }

  const response = await invokeTauriCommand<NativeSaveExportFileResult>('save_export_file', {
    suggestedName: options.suggestedName,
    dataBase64: encodeBytesBase64(options.bytes),
    mode: options.mode,
  });
  if (!response) {
    return null;
  }

  return normalizeSaveResponse(response, options.suggestedName);
}

export function sanitizeExportBaseName(title: string, fallback = 'document'): string {
  const trimmed = title.trim() || fallback;
  return trimmed.replace(/[\\/:*?"<>|]/g, '_');
}

export function buildExportFileName(title: string, extension: string): string {
  const baseName = sanitizeExportBaseName(title);
  const normalizedExtension = extension.replace(/^\./, '');
  if (baseName.toLowerCase().endsWith(`.${normalizedExtension.toLowerCase()}`)) {
    return baseName;
  }
  return `${baseName}.${normalizedExtension}`;
}

const PICKER_MIME_BY_EXTENSION: Record<string, string> = {
  pdf: 'application/pdf',
  md: 'text/markdown',
  doc: 'application/msword',
  png: 'image/png',
};

async function tryWebShareDownload(
  bytes: Uint8Array,
  fileName: string,
  mimeType: string,
): Promise<boolean> {
  if (typeof navigator.share !== 'function' || typeof File === 'undefined') {
    return false;
  }

  try {
    const file = new File([bytes], fileName, { type: mimeType });
    if (typeof navigator.canShare === 'function' && !navigator.canShare({ files: [file] })) {
      return false;
    }
    await navigator.share({ files: [file], title: fileName });
    return true;
  } catch (error: unknown) {
    if (error instanceof DOMException && error.name === 'AbortError') {
      return false;
    }
    return false;
  }
}

function triggerBrowserDownload(bytes: Uint8Array, fileName: string, mimeType: string): void {
  const blob = new Blob([bytes], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = fileName;
  link.rel = 'noopener';
  link.style.display = 'none';
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  window.setTimeout(() => URL.revokeObjectURL(url), 60_000);
}

async function triggerWebDownload(
  bytes: Uint8Array,
  fileName: string,
  mimeType: string,
): Promise<void> {
  const runtime = getExportRuntimeEnvironment();
  const os = detectOperatingSystem();

  if (runtime.isMobile && (await tryWebShareDownload(bytes, fileName, mimeType))) {
    return;
  }

  triggerBrowserDownload(bytes, fileName, mimeType);

  if (os === 'ios' || (runtime.browser === 'safari' && runtime.isMobile)) {
    toast.info('若未自动下载，请在浏览器下载管理或分享菜单中查看文件');
  }
}

async function saveViaFilePicker(
  bytes: Uint8Array,
  suggestedName: string,
  mimeType: string,
): Promise<SaveExportFileResult> {
  if (!isSaveFilePickerAvailable()) {
    await triggerWebDownload(bytes, suggestedName, mimeType);
    return {
      saved: true,
      cancelled: false,
      mode: 'saveAs',
      pathLabel: suggestedName,
    };
  }

  const picker = (window as typeof window & {
    showSaveFilePicker?: (options: {
      suggestedName: string;
      types: Array<{ description: string; accept: Record<string, string[]> }>;
      excludeAcceptAllOption?: boolean;
    }) => Promise<FileSystemFileHandle>;
  }).showSaveFilePicker;

  const extension = suggestedName.includes('.')
    ? suggestedName.split('.').pop()?.toLowerCase() ?? 'bin'
    : 'bin';
  const pickerMime = PICKER_MIME_BY_EXTENSION[extension] ?? mimeType.split(';')[0] ?? mimeType;
  const pickerTypes = [
    {
      description: 'Export file',
      accept: { [pickerMime]: [`.${extension}`] },
    },
  ];

  try {
    const handle = await picker!({
      suggestedName,
      types: pickerTypes,
      excludeAcceptAllOption: true,
    });
    const writable = await handle.createWritable();
    await writable.write(new Blob([bytes], { type: mimeType }));
    await writable.close();
    return {
      saved: true,
      cancelled: false,
      mode: 'saveAs',
      pathLabel: handle.name || suggestedName,
    };
  } catch (error: unknown) {
    if (error instanceof DOMException && error.name === 'AbortError') {
      return { saved: false, cancelled: true, mode: 'saveAs' };
    }
    if (error instanceof DOMException && error.name === 'SecurityError') {
      await triggerWebDownload(bytes, suggestedName, mimeType);
      return {
        saved: true,
        cancelled: false,
        mode: 'saveAs',
        pathLabel: suggestedName,
      };
    }
    throw error;
  }
}

export async function persistExportFile(options: {
  bytes: Uint8Array;
  suggestedName: string;
  mode: ExportSaveMode;
  mimeType: string;
}): Promise<SaveExportFileResult> {
  if (isDesktopExportHost()) {
    const nativeResult = await saveViaNativeHost({
      bytes: options.bytes,
      suggestedName: options.suggestedName,
      mode: options.mode,
    });
    if (nativeResult) {
      return nativeResult;
    }
    return {
      saved: false,
      cancelled: false,
      mode: options.mode,
      pathLabel: options.suggestedName,
    };
  }

  if (options.mode === 'saveAs') {
    return saveViaFilePicker(options.bytes, options.suggestedName, options.mimeType);
  }

  await triggerWebDownload(options.bytes, options.suggestedName, options.mimeType);
  return {
    saved: true,
    cancelled: false,
    mode: 'downloads',
    pathLabel: options.suggestedName,
  };
}

export function describeExportSaveResult(result: SaveExportFileResult): string | null {
  if (result.cancelled || !result.saved) {
    return null;
  }

  const fileLabel = result.pathLabel ?? '文件';
  if (result.mode === 'downloads') {
    if (result.path) {
      return `已保存到「下载」：${shortenPath(result.path)}`;
    }
    return `已保存到「下载」：${fileLabel}`;
  }

  if (result.path) {
    return `已另存为：${shortenPath(result.path)}`;
  }
  return `已另存为：${fileLabel}`;
}

async function invokeHostOrTauri(
  hostMethod: keyof HostExportAdapter,
  tauriCommand: string,
  request: Record<string, unknown>,
  hostCall: (host: HostExportAdapter) => Promise<void>,
): Promise<boolean> {
  if (!isDesktopExportHost()) {
    return false;
  }

  const host = await loadHostAdapter();
  const hostFn = host?.[hostMethod];
  if (host?.isNativeHost && typeof hostFn === 'function') {
    try {
      await hostCall(host);
      return true;
    } catch {
      // fall through to Tauri invoke
    }
  }

  const result = await invokeTauriCommand<void>(tauriCommand, request);
  return result !== null;
}

export async function revealExportInFolder(path: string): Promise<void> {
  if (!path || !isDesktopExportHost()) {
    return;
  }

  const ok = await invokeHostOrTauri(
    'revealExportFile',
    'reveal_export_file',
    { path },
    async (host) => {
      await host.revealExportFile!(path);
    },
  );
  if (!ok) {
    toast.error('无法打开所在文件夹');
  }
}

export async function openExportFile(path: string): Promise<void> {
  if (!path || !isDesktopExportHost()) {
    return;
  }

  const ok = await invokeHostOrTauri(
    'openExportFile',
    'open_export_file',
    { path },
    async (host) => {
      await host.openExportFile!(path);
    },
  );
  if (!ok) {
    toast.error('无法打开文件');
  }
}

export async function resolveSavedExportPath(
  result: SaveExportFileResult,
): Promise<string | undefined> {
  if (result.path?.trim()) {
    return result.path;
  }

  if (!result.pathLabel?.trim()) {
    return undefined;
  }

  if (!isDesktopExportHost()) {
    return undefined;
  }

  const host = await loadHostAdapter();
  if (host?.isNativeHost && typeof host.locateExportFile === 'function') {
    try {
      const located = await host.locateExportFile(result.pathLabel);
      if (located) {
        return located;
      }
    } catch {
      // fall through to direct Tauri invoke
    }
  }

  if (result.mode === 'saveAs') {
    return undefined;
  }

  const located = await invokeTauriCommand<string>('locate_export_file', {
    fileName: result.pathLabel,
  });
  return located ?? undefined;
}

function shortenPath(path: string, maxLength = 56): string {
  if (path.length <= maxLength) {
    return path;
  }
  const parts = path.split(/[/\\]/);
  const fileName = parts[parts.length - 1] ?? path;
  if (fileName.length >= maxLength - 3) {
    return `...${fileName.slice(-(maxLength - 3))}`;
  }
  const separator = path.includes('\\') ? '\\' : '/';
  return `...${parts.slice(-2).join(separator)}`;
}

export const EXPORT_MIME_TYPES = {
  pdf: 'application/pdf',
  markdown: 'text/markdown;charset=utf-8',
  word: 'application/msword',
  image: 'image/png',
} as const;
