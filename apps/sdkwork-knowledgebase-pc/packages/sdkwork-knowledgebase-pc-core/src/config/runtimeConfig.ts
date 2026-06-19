export type SdkworkEnvironment = 'development' | 'test' | 'staging' | 'production';
export type SdkworkConfigProfile = 'dev' | 'test' | 'staging' | 'prod';
export type SdkworkBuildMode = 'development' | 'test' | 'staging' | 'production';
export type SdkworkDeploymentMode =
  | 'web'
  | 'desktop'
  | 'tablet-ipados'
  | 'tablet-android'
  | 'server'
  | 'container'
  | 'saas'
  | 'private'
  | 'local'
  | 'test';
export type SdkworkRuntimeTarget =
  | 'browser'
  | 'desktop'
  | 'tablet-ipados'
  | 'tablet-android'
  | 'server'
  | 'container'
  | 'test-runner';

export type KnowledgebaseHosting = 'self-hosted' | 'cloud-hosted';

export interface SdkworkDependencySdkBaseUrls {
  openApiBaseUrl?: string;
  appApiBaseUrl?: string;
  backendApiBaseUrl?: string;
}

export interface SdkworkSdkBaseUrlConfig {
  defaultApiBaseUrl?: string;
  openApiBaseUrl?: string;
  appApiBaseUrl: string;
  backendApiBaseUrl?: string;
  dependencySdkBaseUrls: Record<string, SdkworkDependencySdkBaseUrls>;
}

export interface SdkworkAuthRuntimeConfig {
  tokenManagerMode: 'appbase-global' | 'service-context' | 'test';
  tokenStorage:
    | 'memory'
    | 'browser-session'
    | 'browser-local'
    | 'os-secure-storage'
    | 'server-context';
  accessTokenHeader: 'Access-Token';
  authTokenHeader: 'Authorization';
  refreshEnabled: boolean;
}

export interface KnowledgebaseRuntimeConfig {
  hosting: KnowledgebaseHosting;
  environment: SdkworkEnvironment;
  configProfile: SdkworkConfigProfile;
  buildMode: SdkworkBuildMode;
  deploymentMode: SdkworkDeploymentMode;
  runtimeTarget: SdkworkRuntimeTarget;
  appKey: 'sdkwork-knowledgebase-pc';
  appApiBaseUrl: string;
  backendApiBaseUrl: string;
  openApiBaseUrl: string;
  platformApiGatewayBaseUrl: string;
  sdkBaseUrls: SdkworkSdkBaseUrlConfig;
  auth: SdkworkAuthRuntimeConfig;
}

export interface RuntimeEnv {
  VITE_SDKWORK_KNOWLEDGEBASE_HOSTING?: string;
  VITE_SDKWORK_KNOWLEDGEBASE_ENVIRONMENT?: string;
  VITE_SDKWORK_KNOWLEDGEBASE_CONFIG_PROFILE?: string;
  VITE_SDKWORK_KNOWLEDGEBASE_BUILD_MODE?: string;
  VITE_SDKWORK_KNOWLEDGEBASE_DEPLOYMENT_MODE?: string;
  VITE_SDKWORK_KNOWLEDGEBASE_RUNTIME_TARGET?: string;
  VITE_SDKWORK_KNOWLEDGEBASE_APPLICATION_PUBLIC_HTTP_URL?: string;
  VITE_SDKWORK_KNOWLEDGEBASE_APPLICATION_BACKEND_HTTP_URL?: string;
  VITE_SDKWORK_KNOWLEDGEBASE_APPLICATION_OPEN_HTTP_URL?: string;
  VITE_SDKWORK_KNOWLEDGEBASE_PLATFORM_API_GATEWAY_HTTP_URL?: string;
  VITE_SDKWORK_APPBASE_APP_API_BASE_URL?: string;
  VITE_SDKWORK_KNOWLEDGEBASE_DEV_SAME_ORIGIN_API?: string;
  VITE_SDKWORK_KNOWLEDGEBASE_TOKEN_MANAGER_MODE?: string;
  VITE_SDKWORK_KNOWLEDGEBASE_TOKEN_STORAGE?: string;
  DEV?: boolean;
  MODE?: string;
  PROD?: boolean;
}

const APP_KEY = 'sdkwork-knowledgebase-pc';
const LOCAL_APP_API_BASE_URL = 'http://127.0.0.1:18081';
const LOCAL_BACKEND_API_BASE_URL = 'http://127.0.0.1:18082';
const LOCAL_OPEN_API_BASE_URL = 'http://127.0.0.1:18083';
const LOCAL_PLATFORM_API_GATEWAY_BASE_URL = 'http://127.0.0.1:3900';

const CLOUD_APP_API_BASE_URL = 'https://knowledgebase.sdkwork.com/app/v3/api';
const CLOUD_BACKEND_API_BASE_URL = 'https://knowledgebase-admin.sdkwork.com/backend/v3/api';
const CLOUD_OPEN_API_BASE_URL = 'https://knowledge.sdkwork.com/knowledge/v3/api';
const CLOUD_PLATFORM_API_GATEWAY_BASE_URL = 'https://api.sdkwork.com';

const VALID_ENVIRONMENTS: SdkworkEnvironment[] = [
  'development',
  'test',
  'staging',
  'production',
];
const VALID_CONFIG_PROFILES: SdkworkConfigProfile[] = ['dev', 'test', 'staging', 'prod'];
const VALID_BUILD_MODES: SdkworkBuildMode[] = [
  'development',
  'test',
  'staging',
  'production',
];
const VALID_DEPLOYMENT_MODES: SdkworkDeploymentMode[] = [
  'web',
  'desktop',
  'tablet-ipados',
  'tablet-android',
  'server',
  'container',
  'saas',
  'private',
  'local',
  'test',
];
const VALID_RUNTIME_TARGETS: SdkworkRuntimeTarget[] = [
  'browser',
  'desktop',
  'tablet-ipados',
  'tablet-android',
  'server',
  'container',
  'test-runner',
];

function normalized(value: string | undefined): string | undefined {
  return value?.trim().toLowerCase() || undefined;
}

function parseOneOf<T extends string>(
  value: string | undefined,
  validValues: readonly T[],
  fallback: T,
): T {
  const nextValue = normalized(value);
  if (nextValue && validValues.includes(nextValue as T)) {
    return nextValue as T;
  }
  return fallback;
}

function normalizeEnvironment(value: string | undefined, env: RuntimeEnv): SdkworkEnvironment {
  const nextValue = normalized(value);
  if (nextValue === 'dev') {
    return 'development';
  }
  if (nextValue === 'prod') {
    return 'production';
  }
  if (nextValue && VALID_ENVIRONMENTS.includes(nextValue as SdkworkEnvironment)) {
    return nextValue as SdkworkEnvironment;
  }
  if (env.PROD) {
    return 'production';
  }
  return 'development';
}

function normalizeProfile(
  value: string | undefined,
  environment: SdkworkEnvironment,
): SdkworkConfigProfile {
  const nextValue = normalized(value);
  if (nextValue && VALID_CONFIG_PROFILES.includes(nextValue as SdkworkConfigProfile)) {
    return nextValue as SdkworkConfigProfile;
  }
  if (environment === 'production') {
    return 'prod';
  }
  if (environment === 'development') {
    return 'dev';
  }
  return environment;
}

function normalizeBuildMode(
  value: string | undefined,
  env: RuntimeEnv,
  environment: SdkworkEnvironment,
): SdkworkBuildMode {
  const nextValue = normalized(value ?? env.MODE);
  if (nextValue === 'dev') {
    return 'development';
  }
  if (nextValue === 'prod') {
    return 'production';
  }
  return parseOneOf(nextValue, VALID_BUILD_MODES, environment);
}

function normalizeHosting(
  value: string | undefined,
  deploymentMode: SdkworkDeploymentMode,
): KnowledgebaseHosting {
  const explicit = normalized(value);
  if (explicit === 'cloud-hosted' || explicit === 'self-hosted') {
    return explicit;
  }
  if (explicit === 'cloud') {
    return 'cloud-hosted';
  }
  if (explicit === 'standalone') {
    return 'self-hosted';
  }
  if (deploymentMode === 'local' || deploymentMode === 'test') {
    return 'self-hosted';
  }
  return 'cloud-hosted';
}

function defaultPlatformApiGatewayBaseUrl(
  hosting: KnowledgebaseHosting,
  deploymentMode: SdkworkDeploymentMode,
): string {
  if (hosting === 'self-hosted') {
    return LOCAL_PLATFORM_API_GATEWAY_BASE_URL;
  }
  return deploymentMode === 'local' || deploymentMode === 'test'
    ? LOCAL_PLATFORM_API_GATEWAY_BASE_URL
    : CLOUD_PLATFORM_API_GATEWAY_BASE_URL;
}

function defaultAppApiBaseUrl(
  hosting: KnowledgebaseHosting,
  deploymentMode: SdkworkDeploymentMode,
): string {
  if (hosting === 'self-hosted') {
    return LOCAL_APP_API_BASE_URL;
  }
  return deploymentMode === 'local' || deploymentMode === 'test'
    ? LOCAL_APP_API_BASE_URL
    : CLOUD_APP_API_BASE_URL;
}

function defaultBackendApiBaseUrl(
  hosting: KnowledgebaseHosting,
  deploymentMode: SdkworkDeploymentMode,
): string {
  if (hosting === 'self-hosted') {
    return LOCAL_BACKEND_API_BASE_URL;
  }
  return deploymentMode === 'local' || deploymentMode === 'test'
    ? LOCAL_BACKEND_API_BASE_URL
    : CLOUD_BACKEND_API_BASE_URL;
}

function defaultOpenApiBaseUrl(
  hosting: KnowledgebaseHosting,
  deploymentMode: SdkworkDeploymentMode,
): string {
  if (hosting === 'self-hosted') {
    return LOCAL_OPEN_API_BASE_URL;
  }
  return deploymentMode === 'local' || deploymentMode === 'test'
    ? LOCAL_OPEN_API_BASE_URL
    : CLOUD_OPEN_API_BASE_URL;
}

function normalizeTokenManagerMode(
  value: string | undefined,
  environment: SdkworkEnvironment,
): SdkworkAuthRuntimeConfig['tokenManagerMode'] {
  if (value === 'service-context' || value === 'test') {
    return value;
  }
  return environment === 'test' ? 'test' : 'appbase-global';
}

function normalizeTokenStorage(
  value: string | undefined,
  runtimeTarget: SdkworkRuntimeTarget,
  environment: SdkworkEnvironment,
): SdkworkAuthRuntimeConfig['tokenStorage'] {
  if (
    value === 'memory'
    || value === 'browser-session'
    || value === 'browser-local'
    || value === 'os-secure-storage'
    || value === 'server-context'
  ) {
    return value;
  }
  if (environment === 'test') {
    return 'memory';
  }
  return runtimeTarget === 'desktop' ? 'os-secure-storage' : 'browser-session';
}

function shouldUseDevSameOriginApi(
  env: RuntimeEnv,
  deploymentMode: SdkworkDeploymentMode,
): boolean {
  const explicit = normalized(env.VITE_SDKWORK_KNOWLEDGEBASE_DEV_SAME_ORIGIN_API);
  if (explicit === 'true' || explicit === '1') {
    return true;
  }
  if (explicit === 'false' || explicit === '0') {
    return false;
  }
  return Boolean(env.DEV)
    && normalized(env.MODE) === 'development'
    && (deploymentMode === 'local' || deploymentMode === 'test' || deploymentMode === 'desktop');
}

function applyDevSameOriginApiBaseUrl(
  env: RuntimeEnv,
  deploymentMode: SdkworkDeploymentMode,
  baseUrl: string,
): string {
  return shouldUseDevSameOriginApi(env, deploymentMode) ? '' : baseUrl;
}

export function detectRuntimeTargetFromEnv(env: RuntimeEnv = import.meta.env): SdkworkRuntimeTarget {
  const explicit = normalized(env.VITE_SDKWORK_KNOWLEDGEBASE_RUNTIME_TARGET);
  if (explicit && VALID_RUNTIME_TARGETS.includes(explicit as SdkworkRuntimeTarget)) {
    return explicit as SdkworkRuntimeTarget;
  }

  const tauri = (globalThis as typeof globalThis & { __TAURI__?: unknown }).__TAURI__;
  if (tauri) {
    return 'desktop';
  }

  return 'browser';
}

function defaultDeploymentMode(
  env: RuntimeEnv,
  runtimeTarget: SdkworkRuntimeTarget,
  environment: SdkworkEnvironment,
  hosting: KnowledgebaseHosting,
): SdkworkDeploymentMode {
  if (runtimeTarget === 'desktop') {
    return 'desktop';
  }
  if (environment === 'test') {
    return 'test';
  }
  if (hosting === 'self-hosted') {
    return 'local';
  }
  return 'saas';
}

export function createRuntimeConfig(env: RuntimeEnv = import.meta.env): KnowledgebaseRuntimeConfig {
  const environment = normalizeEnvironment(env.VITE_SDKWORK_KNOWLEDGEBASE_ENVIRONMENT, env);
  const runtimeTarget = detectRuntimeTargetFromEnv(env);
  const hosting = normalizeHosting(
    env.VITE_SDKWORK_KNOWLEDGEBASE_HOSTING,
    runtimeTarget === 'desktop' ? 'desktop' : environment === 'test' ? 'test' : 'local',
  );
  const deploymentMode = parseOneOf(
    env.VITE_SDKWORK_KNOWLEDGEBASE_DEPLOYMENT_MODE,
    VALID_DEPLOYMENT_MODES,
    defaultDeploymentMode(env, runtimeTarget, environment, hosting),
  );
  const configProfile = normalizeProfile(env.VITE_SDKWORK_KNOWLEDGEBASE_CONFIG_PROFILE, environment);
  const buildMode = normalizeBuildMode(env.VITE_SDKWORK_KNOWLEDGEBASE_BUILD_MODE, env, environment);

  const platformApiGatewayBaseUrl =
    env.VITE_SDKWORK_KNOWLEDGEBASE_PLATFORM_API_GATEWAY_HTTP_URL
    || defaultPlatformApiGatewayBaseUrl(hosting, deploymentMode);

  const appApiBaseUrl =
    env.VITE_SDKWORK_KNOWLEDGEBASE_APPLICATION_PUBLIC_HTTP_URL
    || defaultAppApiBaseUrl(hosting, deploymentMode);
  const backendApiBaseUrl =
    env.VITE_SDKWORK_KNOWLEDGEBASE_APPLICATION_BACKEND_HTTP_URL
    || defaultBackendApiBaseUrl(hosting, deploymentMode);
  const openApiBaseUrl =
    env.VITE_SDKWORK_KNOWLEDGEBASE_APPLICATION_OPEN_HTTP_URL
    || defaultOpenApiBaseUrl(hosting, deploymentMode);
  const appbaseAppApiBaseUrl = applyDevSameOriginApiBaseUrl(
    env,
    deploymentMode,
    env.VITE_SDKWORK_APPBASE_APP_API_BASE_URL
    || (hosting === 'self-hosted' ? appApiBaseUrl : platformApiGatewayBaseUrl),
  );

  const resolvedAppApiBaseUrl = applyDevSameOriginApiBaseUrl(env, deploymentMode, appApiBaseUrl);
  const resolvedBackendApiBaseUrl = applyDevSameOriginApiBaseUrl(
    env,
    deploymentMode,
    backendApiBaseUrl,
  );
  const resolvedOpenApiBaseUrl = applyDevSameOriginApiBaseUrl(env, deploymentMode, openApiBaseUrl);

  return {
    hosting,
    environment,
    configProfile,
    buildMode,
    deploymentMode,
    runtimeTarget,
    appKey: APP_KEY,
    appApiBaseUrl: resolvedAppApiBaseUrl,
    backendApiBaseUrl: resolvedBackendApiBaseUrl,
    openApiBaseUrl: resolvedOpenApiBaseUrl,
    platformApiGatewayBaseUrl,
    sdkBaseUrls: {
      defaultApiBaseUrl: resolvedAppApiBaseUrl,
      appApiBaseUrl: resolvedAppApiBaseUrl,
      backendApiBaseUrl: resolvedBackendApiBaseUrl,
      openApiBaseUrl: resolvedOpenApiBaseUrl,
      dependencySdkBaseUrls: {
        'sdkwork-appbase-app-sdk': {
          appApiBaseUrl: appbaseAppApiBaseUrl,
        },
        'sdkwork-knowledgebase-app-sdk': {
          appApiBaseUrl: resolvedAppApiBaseUrl,
        },
      },
    },
    auth: {
      tokenManagerMode: normalizeTokenManagerMode(
        env.VITE_SDKWORK_KNOWLEDGEBASE_TOKEN_MANAGER_MODE,
        environment,
      ),
      tokenStorage: normalizeTokenStorage(
        env.VITE_SDKWORK_KNOWLEDGEBASE_TOKEN_STORAGE,
        runtimeTarget,
        environment,
      ),
      accessTokenHeader: 'Access-Token',
      authTokenHeader: 'Authorization',
      refreshEnabled: environment !== 'test',
    },
  };
}
