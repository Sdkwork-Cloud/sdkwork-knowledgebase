import type { ReactElement } from 'react';

export interface SdkworkIamAuthRoutesProps {
  basePath?: string;
  getRuntime: () => unknown;
  homePath?: string;
  viewportMode?: 'fixed' | 'page';
}

export function SdkworkIamAuthRoutes(
  props: SdkworkIamAuthRoutesProps,
): ReactElement | null;
