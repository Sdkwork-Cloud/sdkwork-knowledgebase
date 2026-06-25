import React, { useEffect, useMemo, useState } from 'react';
import { isBlank, trim } from '@sdkwork/sdkwork-knowledgebase-pc-commons/stringUtils';
import type { SessionSnapshot, SessionStore } from '../session/sessionStore';

export interface KnowledgebaseAuthLocationLike {
  hash?: string;
  pathname: string;
  search?: string;
}

export type KnowledgebaseAuthGateDecision =
  | { kind: 'product-route' }
  | { kind: 'auth-route' }
  | { kind: 'redirect'; replace: true; to: string };

export type KnowledgebaseAuthIntegrationMode = 'standalone' | 'host-managed';

export interface KnowledgebaseAuthGateProps {
  authRoutes?: React.ReactNode;
  children: React.ReactNode;
  homePath?: string;
  integrationMode?: KnowledgebaseAuthIntegrationMode;
  location?: KnowledgebaseAuthLocationLike;
  navigate?: (to: string, options: { replace: true }) => void;
  session: SessionStore;
}

const DEFAULT_HOME_PATH = '/';
const AUTH_BASE_PATH = '/auth';
const AUTH_LOGIN_PATH = '/auth/login';

/** Set when redirecting away from auth routes after login; AppShell resets landing tab to knowledge base. */
export const KNOWLEDGEBASE_POST_AUTH_LANDING_FLAG = 'knowledgebase-post-auth-landing';

export function hasKnowledgebaseIamSession(session: SessionSnapshot): boolean {
  return Boolean(session.authToken && session.accessToken && session.context?.tenantId);
}

export function buildKnowledgebaseAuthLoginRedirect(
  location: KnowledgebaseAuthLocationLike,
): string {
  const returnPath = `${normalizePathname(location.pathname)}${location.search ?? ''}${
    location.hash ?? ''
  }`;
  return `${AUTH_LOGIN_PATH}?redirect=${encodeURIComponent(returnPath)}`;
}

export function sanitizeKnowledgebaseAuthRedirect(value: string | null | undefined): string {
  if (!value) {
    return DEFAULT_HOME_PATH;
  }

  let decoded = value;
  try {
    decoded = decodeURIComponent(value);
  } catch {
    return DEFAULT_HOME_PATH;
  }

  if (!decoded.startsWith('/') || decoded.startsWith('//')) {
    return DEFAULT_HOME_PATH;
  }

  const redirectUrl = new URL(decoded, 'http://sdkwork-knowledgebase.local');
  if (isKnowledgebaseAuthRoute(redirectUrl.pathname)) {
    return DEFAULT_HOME_PATH;
  }

  return `${redirectUrl.pathname}${redirectUrl.search}${redirectUrl.hash}`;
}

export function resolveKnowledgebaseAuthGateDecision({
  hasSession,
  homePath = DEFAULT_HOME_PATH,
  integrationMode = 'standalone',
  location,
}: {
  hasSession: boolean;
  homePath?: string;
  integrationMode?: KnowledgebaseAuthIntegrationMode;
  location: KnowledgebaseAuthLocationLike;
}): KnowledgebaseAuthGateDecision {
  if (integrationMode === 'host-managed') {
    return { kind: 'product-route' };
  }

  const pathname = normalizePathname(location.pathname);
  if (isKnowledgebaseAuthRoute(pathname)) {
    if (!hasSession) {
      return { kind: 'auth-route' };
    }

    const redirect = new URLSearchParams((location.search ?? '').replace(/^\?/, '')).get(
      'redirect',
    );
    return {
      kind: 'redirect',
      replace: true,
      to: sanitizeKnowledgebaseAuthRedirect(redirect) || normalizePathname(homePath),
    };
  }

  if (!hasSession) {
    return {
      kind: 'redirect',
      replace: true,
      to: buildKnowledgebaseAuthLoginRedirect(location),
    };
  }

  return { kind: 'product-route' };
}

export function KnowledgebaseAuthGate({
  authRoutes,
  children,
  homePath = DEFAULT_HOME_PATH,
  integrationMode = 'standalone',
  location,
  navigate,
  session,
}: KnowledgebaseAuthGateProps) {
  const [snapshot, setSnapshot] = useState(() => session.getSnapshot());
  const currentLocation = useBrowserLocation(location);

  useEffect(() => {
    setSnapshot(session.getSnapshot());
    return session.subscribe(setSnapshot);
  }, [session, currentLocation.pathname, currentLocation.search, currentLocation.hash]);

  const decision = useMemo(
    () =>
      resolveKnowledgebaseAuthGateDecision({
        hasSession: hasKnowledgebaseIamSession(snapshot),
        homePath,
        integrationMode,
        location: currentLocation,
      }),
    [currentLocation, homePath, integrationMode, snapshot],
  );

  useEffect(() => {
    if (decision.kind !== 'redirect') {
      return;
    }
    if (
      typeof window !== 'undefined' &&
      isKnowledgebaseAuthRoute(normalizePathname(currentLocation.pathname))
    ) {
      try {
        window.sessionStorage.setItem(KNOWLEDGEBASE_POST_AUTH_LANDING_FLAG, '1');
      } catch {
        // Ignore storage errors; landing tab reset is best-effort.
      }
    }
    if (navigate) {
      navigate(decision.to, { replace: true });
      return;
    }
    if (typeof window !== 'undefined') {
      window.location.replace(decision.to);
    }
  }, [decision, currentLocation.pathname, navigate]);

  if (decision.kind === 'redirect') {
    return null;
  }

  if (decision.kind === 'auth-route') {
    return React.createElement(React.Fragment, null, authRoutes);
  }

  return React.createElement(React.Fragment, null, children);
}

export function isKnowledgebaseAuthRoute(pathname: string): boolean {
  return pathname === AUTH_BASE_PATH || pathname.startsWith(`${AUTH_BASE_PATH}/`);
}

function normalizePathname(pathname: string): string {
  const normalized = pathname.trim();
  if (!normalized) {
    return DEFAULT_HOME_PATH;
  }
  return normalized.startsWith('/') ? normalized : `/${normalized}`;
}

function useBrowserLocation(
  location: KnowledgebaseAuthLocationLike | undefined,
): KnowledgebaseAuthLocationLike {
  const [browserLocation, setBrowserLocation] = useState<KnowledgebaseAuthLocationLike>(() =>
    location ?? readBrowserLocation(),
  );

  useEffect(() => {
    if (location) {
      setBrowserLocation(location);
      return undefined;
    }
    if (typeof window === 'undefined') {
      return undefined;
    }

    const update = () => setBrowserLocation(readBrowserLocation());
    window.addEventListener('popstate', update);
    window.addEventListener('hashchange', update);
    window.addEventListener('storage', update);
    return () => {
      window.removeEventListener('popstate', update);
      window.removeEventListener('hashchange', update);
      window.removeEventListener('storage', update);
    };
  }, [location]);

  return browserLocation;
}

function readBrowserLocation(): KnowledgebaseAuthLocationLike {
  if (typeof window === 'undefined') {
    return { pathname: DEFAULT_HOME_PATH, search: '', hash: '' };
  }
  return {
    pathname: window.location.pathname,
    search: window.location.search,
    hash: window.location.hash,
  };
}
