import type { SearchSession } from './types';

export const MAX_SEARCH_SESSIONS = 50;
export const MAX_MESSAGES_PER_SESSION = 100;

export function trimSearchSessions(sessions: SearchSession[]): SearchSession[] {
  const trimmedSessions = sessions
    .slice(0, MAX_SEARCH_SESSIONS)
    .map((session) => ({
      ...session,
      messages: session.messages.slice(-MAX_MESSAGES_PER_SESSION),
    }));

  return trimmedSessions;
}

export function persistSearchSessions(
  storageKey: string,
  sessions: SearchSession[],
): SearchSession[] {
  const trimmed = trimSearchSessions(sessions);
  try {
    localStorage.setItem(storageKey, JSON.stringify(trimmed));
  } catch (error) {
    console.error('Failed to persist search sessions to localStorage', error);
  }
  return trimmed;
}

export function loadSearchSessions(storageKey: string): SearchSession[] | null {
  try {
    const stored = localStorage.getItem(storageKey);
    if (!stored) {
      return null;
    }
    const parsed = JSON.parse(stored) as SearchSession[];
    if (!Array.isArray(parsed) || parsed.length === 0) {
      return null;
    }
    return trimSearchSessions(parsed);
  } catch (error) {
    console.error('Failed to parse search sessions from localStorage', error);
    return null;
  }
}
