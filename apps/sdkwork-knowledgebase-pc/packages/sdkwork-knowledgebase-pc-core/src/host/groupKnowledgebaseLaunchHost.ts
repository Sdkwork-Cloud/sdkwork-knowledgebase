import { isValidGroupKnowledgebaseLaunchTicket } from '../runtime/groupKnowledgebaseLaunchTicket';
import {
  invokeDesktopCommand,
  isTauriDesktopRuntime,
  listenDesktopEvent,
} from './tauriBridge';

export const GROUP_KNOWLEDGEBASE_LAUNCH_EVENT = 'sdkwork://knowledgebase/group-launch';

const GROUP_KNOWLEDGEBASE_ROUTE_PATTERN = /^\/group-launch#ticket=([^#]+)$/u;

export interface GroupKnowledgebaseDesktopLaunchEvent {
  route: string;
}

export interface GroupKnowledgebaseDesktopLaunchHost {
  subscribe(listener: (route: string) => void): () => void;
  takePending(): Promise<string | null>;
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
      if (!isTauriDesktopRuntime()) {
        return () => undefined;
      }

      let disposed = false;
      let unlisten: (() => void) | undefined;
      void listenDesktopEvent<unknown>(GROUP_KNOWLEDGEBASE_LAUNCH_EVENT, (payload) => {
        const route = readRoute(payload);
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
      if (!isTauriDesktopRuntime()) {
        return null;
      }
      try {
        const event = await invokeDesktopCommand<GroupKnowledgebaseDesktopLaunchEvent | null>(
          'take_pending_group_knowledgebase_launch',
        );
        return readRoute(event);
      } catch {
        return null;
      }
    },
  };
}
