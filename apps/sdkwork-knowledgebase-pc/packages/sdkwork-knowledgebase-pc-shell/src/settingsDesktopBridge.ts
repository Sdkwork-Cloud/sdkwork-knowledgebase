import { SETTINGS_STORAGE_KEYS } from './settingsModalConstants';

export interface DesktopPreferencesSnapshot {
  autoStart: boolean;
  hideToTray: boolean;
}

export type DesktopPlatform = 'windows' | 'macos' | 'linux' | 'unknown';

export interface DesktopHostStatus {
  enabled: boolean;
  supported: boolean;
  platform: DesktopPlatform;
  hideToTraySupported: boolean;
}

export interface DesktopPreferenceSyncResult {
  ok: boolean;
  errorMessage?: string;
}

export interface TrayLocaleSnapshot {
  showLabel: string;
  settingsLabel: string;
  quitLabel: string;
  tooltip: string;
}

interface TauriInvokeGlobal {
  core?: {
    invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
  };
  event?: {
    listen(event: string, handler: () => void): Promise<() => void>;
  };
}

const OPEN_SETTINGS_EVENT = 'open-settings';

function getTauriGlobal(): TauriInvokeGlobal | undefined {
  return (globalThis as typeof globalThis & { __TAURI__?: TauriInvokeGlobal }).__TAURI__;
}

function getTauriInvoke(): TauriInvokeGlobal['core'] | undefined {
  return getTauriGlobal()?.core;
}

export function isDesktopHostAvailable(): boolean {
  return Boolean(getTauriInvoke()?.invoke);
}

function normalizeDesktopPlatform(value: string | undefined): DesktopPlatform {
  if (value === 'windows' || value === 'macos' || value === 'linux') {
    return value;
  }
  return 'unknown';
}

export function readStoredDesktopPreferences(): DesktopPreferencesSnapshot {
  if (typeof window === 'undefined') {
    return { autoStart: false, hideToTray: true };
  }

  try {
    const autoStartRaw = window.localStorage.getItem(SETTINGS_STORAGE_KEYS.autoStart);
    const hideToTrayRaw = window.localStorage.getItem(SETTINGS_STORAGE_KEYS.hideToTray);
    return {
      autoStart: autoStartRaw ? JSON.parse(autoStartRaw) === true : false,
      hideToTray: hideToTrayRaw ? JSON.parse(hideToTrayRaw) !== false : true,
    };
  } catch {
    return { autoStart: false, hideToTray: true };
  }
}

export async function syncDesktopPreferences(
  preferences: DesktopPreferencesSnapshot,
): Promise<DesktopPreferenceSyncResult> {
  const invoke = getTauriInvoke()?.invoke;
  if (!invoke) {
    return { ok: true };
  }

  try {
    await invoke('sync_desktop_preferences', {
      request: {
        hideToTray: preferences.hideToTray,
        autoStart: preferences.autoStart,
      },
    });
    return { ok: true };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    return { ok: false, errorMessage };
  }
}

export async function readDesktopHostStatus(): Promise<DesktopHostStatus | null> {
  const invoke = getTauriInvoke()?.invoke;
  if (!invoke) {
    return null;
  }

  try {
    const response = await invoke<{
      enabled: boolean;
      supported: boolean;
      platform: string;
      hideToTraySupported: boolean;
    }>('get_desktop_host_status');

    return {
      enabled: response.enabled,
      supported: response.supported,
      platform: normalizeDesktopPlatform(response.platform),
      hideToTraySupported: response.hideToTraySupported,
    };
  } catch {
    return null;
  }
}

export async function syncTrayLocale(locale: TrayLocaleSnapshot): Promise<void> {
  const invoke = getTauriInvoke()?.invoke;
  if (!invoke) {
    return;
  }

  try {
    await invoke('sync_tray_locale', {
      request: {
        showLabel: locale.showLabel,
        settingsLabel: locale.settingsLabel,
        quitLabel: locale.quitLabel,
        tooltip: locale.tooltip,
      },
    });
  } catch {
    // Desktop bridge failures should not block UI preference persistence.
  }
}

export async function listenDesktopOpenSettings(handler: () => void): Promise<() => void> {
  const listen = getTauriGlobal()?.event?.listen;
  if (!listen) {
    return () => {};
  }

  try {
    return await listen(OPEN_SETTINGS_EVENT, handler);
  } catch {
    return () => {};
  }
}
