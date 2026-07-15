import { isValidGroupKnowledgebaseLaunchTicket } from './groupKnowledgebaseLaunchTicket';

let pendingGroupKnowledgebaseLaunchTicket: string | null = null;

/** This path is relative to the BrowserRouter basename. */
export const GROUP_KNOWLEDGEBASE_LAUNCH_PATH = '/group-launch';

export interface GroupKnowledgebaseLaunchLocation {
  hash?: string;
  pathname: string;
  search?: string;
}

/**
 * Group launch tickets are capabilities, so no fragment from the launch route
 * may become part of an authentication return URL. This preserves ordinary
 * route state while keeping all launch fragments out of auth navigation.
 */
export function sanitizeGroupKnowledgebaseLaunchAuthLocation(
  location: GroupKnowledgebaseLaunchLocation,
): GroupKnowledgebaseLaunchLocation {
  if (location.pathname !== GROUP_KNOWLEDGEBASE_LAUNCH_PATH || !location.hash) {
    return location;
  }

  return { ...location, hash: '' };
}

/**
 * Moves a valid group launch ticket out of a browser fragment before auth
 * routing can copy the return location into a query parameter. The value is
 * deliberately memory-only and is consumed exactly once by the group route.
 */
export function captureGroupKnowledgebaseLaunchTicket(
  location: GroupKnowledgebaseLaunchLocation,
): boolean {
  if (location.pathname !== GROUP_KNOWLEDGEBASE_LAUNCH_PATH || !location.hash?.startsWith('#')) {
    return false;
  }

  const params = new URLSearchParams(location.hash.slice(1));
  const ticket = params.get('ticket');
  const isCanonicalTicket = params.size === 1 && isValidGroupKnowledgebaseLaunchTicket(ticket);
  if (isCanonicalTicket) {
    pendingGroupKnowledgebaseLaunchTicket = ticket;
  }
  if (typeof window !== 'undefined') {
    // React Router exposes paths relative to its basename. Use the actual
    // browser location so fragment removal preserves a non-root deployment.
    // This runs for malformed fragments too: an ambiguous value could still
    // contain a syntactically valid capability and must never reach auth.
    window.history.replaceState(
      window.history.state,
      '',
      window.location.pathname + window.location.search,
    );
  }
  return isCanonicalTicket;
}

export function takePendingGroupKnowledgebaseLaunchTicket(): string | null {
  const ticket = pendingGroupKnowledgebaseLaunchTicket;
  pendingGroupKnowledgebaseLaunchTicket = null;
  return ticket;
}
