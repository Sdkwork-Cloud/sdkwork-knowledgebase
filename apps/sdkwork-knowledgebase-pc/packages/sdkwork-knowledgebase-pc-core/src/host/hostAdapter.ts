import { throwKnowledgebaseError } from '../errors/knowledgebaseAppError';
import { KnowledgebaseErrorCodes } from '../errors/knowledgebaseErrorCodes';

export interface BinaryResourcePayload {
  dataBase64: string;
  mimeType?: string | null;
  byteLength: number;
}

export interface HostAdapter {
  isNativeHost: boolean;
  windowControl(action: WindowControlAction): Promise<void>;
  openExternal(url: string): Promise<void>;
  writeTextToClipboard(text: string): Promise<void>;
  fetchBinaryResource(url: string): Promise<BinaryResourcePayload>;
  readLocalResource(path: string): Promise<BinaryResourcePayload>;
  saveBinaryResource(suggestedName: string, bytes: Uint8Array): Promise<boolean>;
  saveExportFile(options: {
    suggestedName: string;
    bytes: Uint8Array;
    mode: 'downloads' | 'saveAs';
  }): Promise<NativeSaveExportFileResult>;
  revealExportFile(path: string): Promise<void>;
  openExportFile(path: string): Promise<void>;
  locateExportFile(fileName: string): Promise<string | null>;
}

export interface NativeSaveExportFileResult {
  saved: boolean;
  cancelled: boolean;
  path?: string | null;
  mode: 'downloads' | 'saveAs';
}

export type WindowControlAction = 'minimize' | 'maximize' | 'unmaximize' | 'close' | 'show';

interface TauriGlobal {
  core?: {
    invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
  };
  shell?: {
    open(url: string): Promise<void>;
  };
  clipboard?: {
    writeText(text: string): Promise<void>;
  };
}

function getTauriGlobal(): TauriGlobal | undefined {
  return (globalThis as typeof globalThis & { __TAURI__?: TauriGlobal }).__TAURI__;
}

function assertSafeExternalUrl(url: string): void {
  const parsed = new URL(url);
  if (parsed.protocol !== 'https:' && parsed.protocol !== 'http:') {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.URL_INVALID_SCHEME);
  }
}

function encodeBytesBase64(bytes: Uint8Array): string {
  let binary = '';
  const chunkSize = 0x8000;
  for (let index = 0; index < bytes.length; index += chunkSize) {
    const chunk = bytes.subarray(index, index + chunkSize);
    binary += String.fromCharCode(...chunk);
  }
  return btoa(binary);
}

export function decodeBinaryResourcePayload(payload: BinaryResourcePayload): Uint8Array {
  const binary = atob(payload.dataBase64);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return bytes;
}

function basenameFromPath(path: string): string {
  const normalized = path.replace(/\\/g, '/');
  const segments = normalized.split('/');
  return segments[segments.length - 1] || path;
}

export function createHostAdapter(): HostAdapter {
  return {
    get isNativeHost() {
      return Boolean(getTauriGlobal());
    },
    async windowControl(action) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        return;
      }
      await tauri.core.invoke('window_control', { request: { action } });
    },
    async openExternal(url) {
      assertSafeExternalUrl(url);
      const tauri = getTauriGlobal();
      if (tauri?.core?.invoke) {
        await tauri.core.invoke('open_external_url', { request: { url } });
        return;
      }
      if (tauri?.shell?.open) {
        await tauri.shell.open(url);
        return;
      }
      globalThis.open?.(url, '_blank', 'noopener,noreferrer');
    },
    async writeTextToClipboard(text) {
      const tauri = getTauriGlobal();
      if (tauri?.clipboard?.writeText) {
        await tauri.clipboard.writeText(text);
        return;
      }
      await navigator.clipboard?.writeText(text);
    },
    async fetchBinaryResource(url) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        throwKnowledgebaseError(KnowledgebaseErrorCodes.DESKTOP_ONLY);
      }
      return tauri.core.invoke<BinaryResourcePayload>('fetch_binary_resource', {
        request: { url },
      });
    },
    async readLocalResource(path) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        throwKnowledgebaseError(KnowledgebaseErrorCodes.DESKTOP_ONLY);
      }
      return tauri.core.invoke<BinaryResourcePayload>('read_local_resource', {
        request: { path },
      });
    },
    async saveBinaryResource(suggestedName, bytes) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        const blob = new Blob([bytes], { type: 'application/pdf' });
        const url = URL.createObjectURL(blob);
        const anchor = document.createElement('a');
        anchor.href = url;
        anchor.download = suggestedName;
        anchor.click();
        URL.revokeObjectURL(url);
        return true;
      }
      return tauri.core.invoke<boolean>('save_binary_resource', {
        request: {
          suggestedName,
          dataBase64: encodeBytesBase64(bytes),
        },
      });
    },
    async saveExportFile({ suggestedName, bytes, mode }) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        throwKnowledgebaseError(KnowledgebaseErrorCodes.DESKTOP_ONLY);
      }
      const response = await tauri.core.invoke<NativeSaveExportFileResult>('save_export_file', {
        request: {
          suggestedName,
          dataBase64: encodeBytesBase64(bytes),
          mode,
        },
      });
      return response;
    },
    async revealExportFile(path) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        return;
      }
      await tauri.core.invoke('reveal_export_file', {
        request: { path },
      });
    },
    async openExportFile(path) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        return;
      }
      await tauri.core.invoke('open_export_file', {
        request: { path },
      });
    },
    async locateExportFile(fileName) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        return null;
      }
      try {
        return await tauri.core.invoke<string>('locate_export_file', {
          request: { fileName },
        });
      } catch {
        return null;
      }
    },
  };
}
