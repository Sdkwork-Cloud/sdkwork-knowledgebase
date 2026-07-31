import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export type DesktopCommandName =
  | 'window_control'
  | 'fetch_binary_resource'
  | 'read_local_resource'
  | 'open_external_url'
  | 'save_binary_resource'
  | 'save_export_file'
  | 'reveal_export_file'
  | 'open_export_file'
  | 'locate_export_file'
  | 'export_document_pdf'
  | 'sync_desktop_preferences'
  | 'get_desktop_host_status'
  | 'get_autostart_enabled'
  | 'sync_tray_locale'
  | 'write_secure_session_value'
  | 'remove_secure_session_value'
  | 'clear_secure_session_values'
  | 'read_secure_session_value'
  | 'take_pending_group_knowledgebase_launch';

export type DesktopEventName = 'open-settings';

export function isTauriDesktopRuntime(): boolean {
  return typeof window !== 'undefined' && isTauri();
}

export async function invokeDesktopCommand<T>(
  command: DesktopCommandName,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!isTauriDesktopRuntime()) {
    throw new Error('desktop host is unavailable');
  }
  return invoke<T>(command, args);
}

export async function listenDesktopEvent<T>(
  eventName: DesktopEventName,
  handler: (payload: T) => void,
): Promise<() => void> {
  if (!isTauriDesktopRuntime()) {
    return () => undefined;
  }
  return listen<T>(eventName, (event) => handler(event.payload));
}
