import { isKnowledgebaseApiAvailable } from './knowledgebaseApiRegistry';
import { KnowledgebaseErrorCodes } from '../errors/knowledgebaseErrorCodes';
import { throwKnowledgebaseError } from '../errors/knowledgebaseAppError';

let networkOnline = typeof navigator === 'undefined' ? true : navigator.onLine;

/** Shell/bootstrap updates browser connectivity for fail-closed API usage. */
export function setKnowledgebaseNetworkOnline(online: boolean): void {
  networkOnline = online;
}

export function isKnowledgebaseNetworkOnline(): boolean {
  return networkOnline;
}

/** True when live API calls are expected to succeed (network + SDK client). */
export function isKnowledgebaseWritableSurface(): boolean {
  return networkOnline && isKnowledgebaseApiAvailable();
}

export function requireKnowledgebaseNetworkOnline(
  operation?: string,
): void {
  if (!networkOnline) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.NETWORK_OFFLINE, {
      cause: operation,
    });
  }
}

export function requireKnowledgebaseWritableMutation(operation?: string): void {
  requireKnowledgebaseNetworkOnline(operation);
  if (!isKnowledgebaseApiAvailable()) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.API_UNAVAILABLE, {
      cause: operation,
    });
  }
}
