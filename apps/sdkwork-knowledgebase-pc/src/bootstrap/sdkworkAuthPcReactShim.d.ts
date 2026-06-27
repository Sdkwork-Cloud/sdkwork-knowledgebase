import type { ReactElement, ReactNode } from 'react';

export interface SdkworkIamAuthRoutesProps {
  basePath?: string;
  getRuntime: () => unknown;
  homePath?: string;
  viewportMode?: 'fixed' | 'page';
}

export function SdkworkIamAuthRoutes(
  props: SdkworkIamAuthRoutesProps,
): ReactElement | null;

export interface SdkworkSessionAuthBrowserRootProps {
  children: ReactNode;
}

export function SdkworkSessionAuthBrowserRoot(
  props: SdkworkSessionAuthBrowserRootProps,
): ReactElement | null;
