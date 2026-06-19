export type ExportOperatingSystem =
  | 'windows'
  | 'macos'
  | 'linux'
  | 'ios'
  | 'android'
  | 'unknown';

export type ExportRuntimeKind = 'tauri-desktop' | 'web-browser';

export type ExportBrowserFamily = 'chrome' | 'edge' | 'firefox' | 'safari' | 'unknown';

export interface ExportRuntimeEnvironment {
  kind: ExportRuntimeKind;
  os: ExportOperatingSystem;
  browser: ExportBrowserFamily;
  isSecureContext: boolean;
  isMobile: boolean;
}

function readUserAgent(): string {
  return navigator.userAgent.toLowerCase();
}

function readPlatform(): string {
  return (navigator.platform ?? '').toLowerCase();
}

export function detectOperatingSystem(): ExportOperatingSystem {
  const ua = readUserAgent();
  const platform = readPlatform();

  if (/iphone|ipad|ipod/.test(ua) || (platform === 'macintel' && navigator.maxTouchPoints > 1)) {
    return 'ios';
  }
  if (/android/.test(ua)) {
    return 'android';
  }
  if (platform.includes('win') || ua.includes('windows')) {
    return 'windows';
  }
  if (platform.includes('mac') || ua.includes('macintosh')) {
    return 'macos';
  }
  if (platform.includes('linux') || ua.includes('linux')) {
    return 'linux';
  }

  const uaPlatform = (navigator as Navigator & { userAgentData?: { platform?: string } })
    .userAgentData?.platform;
  if (uaPlatform) {
    const normalized = uaPlatform.toLowerCase();
    if (normalized.includes('win')) return 'windows';
    if (normalized.includes('mac')) return 'macos';
    if (normalized.includes('linux')) return 'linux';
    if (normalized.includes('android')) return 'android';
  }

  return 'unknown';
}

export function detectBrowserFamily(): ExportBrowserFamily {
  const ua = readUserAgent();
  if (ua.includes('edg/')) {
    return 'edge';
  }
  if (ua.includes('firefox/')) {
    return 'firefox';
  }
  if (ua.includes('chrome/') || ua.includes('crios/')) {
    return 'chrome';
  }
  if (ua.includes('safari/') && !ua.includes('chrome/') && !ua.includes('chromium/')) {
    return 'safari';
  }
  return 'unknown';
}

export function isMobileExportEnvironment(): boolean {
  const os = detectOperatingSystem();
  if (os === 'ios' || os === 'android') {
    return true;
  }
  return /mobile|tablet/.test(readUserAgent());
}

export function getExportRuntimeEnvironment(): ExportRuntimeEnvironment {
  return {
    kind: isDesktopExportHost() ? 'tauri-desktop' : 'web-browser',
    os: detectOperatingSystem(),
    browser: detectBrowserFamily(),
    isSecureContext: typeof window !== 'undefined' ? window.isSecureContext : false,
    isMobile: isMobileExportEnvironment(),
  };
}

type TauriInvokeFn = <T>(command: string, payload?: Record<string, unknown>) => Promise<T>;

type TauriGlobal = {
  core?: { invoke?: TauriInvokeFn };
  tauri?: { invoke?: TauriInvokeFn };
  invoke?: TauriInvokeFn;
};

export function getTauriGlobal(): TauriGlobal | undefined {
  return (globalThis as typeof globalThis & { __TAURI__?: TauriGlobal }).__TAURI__;
}

export function getTauriInvoke(): TauriInvokeFn | undefined {
  const tauri = getTauriGlobal();
  return tauri?.core?.invoke ?? tauri?.tauri?.invoke ?? tauri?.invoke;
}

export function isDesktopExportHost(): boolean {
  return Boolean(getTauriInvoke());
}

export function isSaveFilePickerAvailable(): boolean {
  if (isDesktopExportHost()) {
    return true;
  }
  if (typeof window === 'undefined' || !window.isSecureContext) {
    return false;
  }
  const picker = (window as Window & { showSaveFilePicker?: unknown }).showSaveFilePicker;
  if (typeof picker !== 'function') {
    return false;
  }
  // iOS Safari exposes APIs inconsistently; prefer download/share fallbacks.
  const os = detectOperatingSystem();
  if (os === 'ios') {
    return false;
  }
  return true;
}

export function canRevealExportInFolder(): boolean {
  return isDesktopExportHost();
}

export function canOpenExportFile(): boolean {
  return isDesktopExportHost();
}

export function resolveExportCanvasScale(): number {
  const runtime = getExportRuntimeEnvironment();
  if (runtime.isMobile) {
    return 1.25;
  }
  if (runtime.browser === 'firefox') {
    return 1.75;
  }
  return 2;
}

export function encodeBytesBase64(bytes: Uint8Array): string {
  const chunkSize = 0x8000;
  const parts: string[] = [];
  for (let offset = 0; offset < bytes.length; offset += chunkSize) {
    const slice = bytes.subarray(offset, offset + chunkSize);
    let binary = '';
    for (let index = 0; index < slice.length; index += 1) {
      binary += String.fromCharCode(slice[index]);
    }
    parts.push(binary);
  }
  return btoa(parts.join(''));
}

export async function invokeTauriCommand<T>(
  command: string,
  request: Record<string, unknown>,
): Promise<T | null> {
  const invoke = getTauriInvoke();
  if (!invoke) {
    return null;
  }

  const attempts: Record<string, unknown>[] = [{ request }, request];

  for (const payload of attempts) {
    try {
      return await invoke<T>(command, payload);
    } catch (error) {
      if (payload === request) {
        console.warn(`[DocumentExport] ${command} failed`, error);
      }
    }
  }

  return null;
}
