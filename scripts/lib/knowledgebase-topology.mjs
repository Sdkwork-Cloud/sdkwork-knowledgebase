import path from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  buildProfileId,
  createTopologyRuntime,
  isTcpPortReachable,
  loadTopologySpec,
  normalizeText,
  waitForHttpHealthy,
} from '@sdkwork/app-topology';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export const REPO_ROOT = path.resolve(__dirname, '..', '..');
export const SPEC_PATH = path.join(REPO_ROOT, 'specs/topology.spec.json');
export const API_GATEWAY_REPO = path.resolve(REPO_ROOT, '..', 'sdkwork-api-gateway');

const spec = loadTopologySpec(SPEC_PATH);
const runtime = createTopologyRuntime(spec, REPO_ROOT);

export const DEFAULT_DEV_PROFILE_ID = runtime.defaults.developmentProfileId;
export const DEFAULT_PRODUCTION_PROFILE_ID = runtime.defaults.productionProfileId;

export function resolveDevProfileId(hosting, serviceLayout = 'split-services') {
  runtime.assertHosting(hosting);
  runtime.assertServiceLayout(serviceLayout);
  return buildProfileId(hosting, serviceLayout, 'development');
}

export function resolveDefaultAppSdkBaseUrl(profileEnv = {}) {
  return (
    profileEnv.SDKWORK_KNOWLEDGEBASE_APPLICATION_PUBLIC_HTTP_URL
    ?? profileEnv.VITE_SDKWORK_KNOWLEDGEBASE_APPLICATION_PUBLIC_HTTP_URL
    ?? 'http://127.0.0.1:18081'
  );
}

export function resolveDefaultBackendSdkBaseUrl(profileEnv = {}) {
  return (
    profileEnv.SDKWORK_KNOWLEDGEBASE_APPLICATION_BACKEND_HTTP_URL
    ?? profileEnv.VITE_SDKWORK_KNOWLEDGEBASE_APPLICATION_BACKEND_HTTP_URL
    ?? 'http://127.0.0.1:18082'
  );
}

export function resolveDefaultOpenSdkBaseUrl(profileEnv = {}) {
  return (
    profileEnv.SDKWORK_KNOWLEDGEBASE_APPLICATION_OPEN_HTTP_URL
    ?? profileEnv.VITE_SDKWORK_KNOWLEDGEBASE_APPLICATION_OPEN_HTTP_URL
    ?? 'http://127.0.0.1:18083'
  );
}

export const loadProfile = runtime.loadProfile;
export const applyProfileEnv = runtime.applyProfileEnv;
export const mergeRuntimeEnv = runtime.mergeRuntimeEnv;
export const loadEnvFile = runtime.loadEnvFile;
export const assertHosting = runtime.assertHosting;
export const assertServiceLayout = runtime.assertServiceLayout;
export const resolveSurfaceHttpUrl = runtime.resolveSurfaceHttpUrl.bind(runtime);
export const resolveSurfaceBind = runtime.resolveSurfaceBind.bind(runtime);
export const shouldAutostartGateway = runtime.shouldAutostartGateway;
export const resolveGatewayBind = runtime.resolveGatewayBind;
export const resolveGatewayBaseUrl = runtime.resolveGatewayBaseUrl;
export const resolveCloudGatewayConfigPath = runtime.resolveCloudGatewayConfigPath;
export const resolveIamDevEnv = runtime.resolveIamDevEnv;
export const listOrchestrationProcesses = runtime.listOrchestrationProcesses;
export const listHealthSurfaces = runtime.listHealthSurfaces;

export { buildProfileId, normalizeText, isTcpPortReachable, waitForHttpHealthy, spec, runtime };
