import { isValidGroupKnowledgebaseLaunchTicket } from '../runtime/groupKnowledgebaseLaunchTicket';

export const GROUP_KNOWLEDGEBASE_LAUNCH_EVENT = 'sdkwork://knowledgebase/group-launch';

const GROUP_KNOWLEDGEBASE_ROUTE_PATTERN = /^\/group-launch#ticket=([^#]+)$/u;

export interface GroupKnowledgebaseDesktopLaunchEvent {
  route: string;
}

export interface GroupKnowledgebaseDesktopLaunchHost {
  subscribe(listener: (route: string) => void): () => void;
  takePending(): Promise<string | null>;
}

interface TauriEventPayload {
  payload: unknown;
}

interface TauriGlobal {
  core?: {
    invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
  };
  event?: {
    listen(
      event: string,
      listener: (event: TauriEventPayload) => void,
    ): Promise<() => void>;
  };
}

function getTauriGlobal(): TauriGlobal | undefined {
  return (globalThis as typeof globalThis & { __TAURI__?: TauriGlobal }).__TAURI__;
}

export function parseGroupKnowledgebaseLaunchRoute(value: unknown): string | null {
  if (typeof value !== 'string') {
    return null;
  }
  const ticket = GROUP_KNOWLEDGEBASE_ROUTE_PATTERN.exec(value)?.[1];
  return isValidGroupKnowledgebaseLaunchTicket(ticket) ? value : null;
}

function readRoute(payload: unknown): string | null {
  if (!payload || typeof payload !== 'object' || Array.isArray(payload)) {
    return null;
  }
  return parseGroupKnowledgebaseLaunchRoute(
    (payload as GroupKnowledgebaseDesktopLaunchEvent).route,
  );
}

export function createGroupKnowledgebaseDesktopLaunchHost(): GroupKnowledgebaseDesktopLaunchHost {
  return {
    subscribe(listener) {
      const listen = getTauriGlobal()?.event?.listen;
      if (!listen) {
        return () => undefined;
      }

      let disposed = false;
      let unlisten: (() => void) | undefined;
      void listen(GROUP_KNOWLEDGEBASE_LAUNCH_EVENT, (event) => {
        const route = readRoute(event.payload);
        if (route) {
          listener(route);
        }
      })
        .then((release) => {
          if (disposed) {
            release();
          } else {
            unlisten = release;
          }
        })
        .catch(() => undefined);

      return () => {
        disposed = true;
        unlisten?.();
      };
    },
    async takePending() {
      const invoke = getTauriGlobal()?.core?.invoke;
      if (!invoke) {
        return null;
      }
      try {
        const event = await invoke<GroupKnowledgebaseDesktopLaunchEvent | null>(
          'take_pending_group_knowledgebase_launch',
        );
        return readRoute(event);
      } catch {
        return null;
      }
    },
  };
}
