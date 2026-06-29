import dependencyComposition from '../../../specs/dependency.composition.json';
import type { SdkworkDependencySdkBaseUrls } from '../config/runtimeConfig.js';

type DependencyCompositionManifest = typeof dependencyComposition;

const APP_SURFACE = dependencyComposition.surfaces.find((surface) => surface.surface === 'app');

export function readDependencyCompositionManifest(): DependencyCompositionManifest {
  return dependencyComposition;
}

export function listDependencySdkWorkspaces(): string[] {
  const workspaces = new Set<string>();
  for (const surface of dependencyComposition.surfaces) {
    for (const client of surface.sdkClients) {
      workspaces.add(client.workspace);
    }
  }
  return [...workspaces].sort();
}

export function buildDependencySdkBaseUrls(input: {
  appApiBaseUrl: string;
  iamAppApiBaseUrl: string;
  driveAppApiBaseUrl: string;
}): Record<string, SdkworkDependencySdkBaseUrls> {
  const result: Record<string, SdkworkDependencySdkBaseUrls> = {};

  for (const client of APP_SURFACE?.sdkClients ?? []) {
    if (client.workspace === 'sdkwork-iam-app-sdk') {
      result[client.workspace] = { appApiBaseUrl: input.iamAppApiBaseUrl };
      continue;
    }
    if (client.workspace === 'sdkwork-drive-app-sdk') {
      result[client.workspace] = { appApiBaseUrl: input.driveAppApiBaseUrl };
      continue;
    }
    result[client.workspace] = { appApiBaseUrl: input.appApiBaseUrl };
  }

  return result;
}
