export const KB_NAV_INTENT_KEY = 'app-kb-nav-intent';

export interface KbNavIntent {
  kbId: string;
  kbTitle?: string;
  docId?: string;
  docTitle?: string;
  docType?: string;
  parentId?: string | null;
  author?: string;
  updatedAt?: string;
  highlight?: boolean;
}

export function setKbNavIntent(intent: KbNavIntent) {
  sessionStorage.setItem(KB_NAV_INTENT_KEY, JSON.stringify(intent));
}

export function readKbNavIntent(): KbNavIntent | null {
  try {
    const raw = sessionStorage.getItem(KB_NAV_INTENT_KEY);
    if (!raw) return null;
    return JSON.parse(raw) as KbNavIntent;
  } catch {
    return null;
  }
}

export function clearKbNavIntent() {
  sessionStorage.removeItem(KB_NAV_INTENT_KEY);
}

export interface KbLocateFileDetail {
  docId: string;
  parentId?: string | null;
}

export function dispatchLocateKbFile(detail: KbLocateFileDetail) {
  window.dispatchEvent(new CustomEvent('kb-locate-file', { detail }));
}

export const APP_OPEN_BROWSER_EVENT = 'app-open-in-app-browser';

export interface InAppBrowserOpenDetail {
  url: string;
  title?: string;
}

export function dispatchOpenInAppBrowser(detail: InAppBrowserOpenDetail) {
  window.dispatchEvent(new CustomEvent(APP_OPEN_BROWSER_EVENT, { detail }));
}
