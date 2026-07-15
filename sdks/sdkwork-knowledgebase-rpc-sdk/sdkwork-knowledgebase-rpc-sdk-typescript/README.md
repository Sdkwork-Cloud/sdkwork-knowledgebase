# SdkworkKnowledgebaseRpc

SdkworkKnowledgebaseRpc is an SDKWork RPC SDK scaffold generated from proto packages and an SDKWork RPC manifest.

## Proto packages

- sdkwork.intelligence.internal.v1

## Service catalog

- sdkwork.intelligence.internal.v1.GroupKnowledgeSpaceLifecycleService (internal)
  - EnsureGroupKnowledgeSpace: internal.groupKnowledgeSpaces.ensure, unary, auth=service-mtls, idempotency=required
  - SynchronizeGroupKnowledgeSpaceMembers: internal.groupKnowledgeSpaceMembers.synchronize, unary, auth=service-mtls, idempotency=required
  - ArchiveGroupKnowledgeSpace: internal.groupKnowledgeSpaces.archive, unary, auth=service-mtls, idempotency=required

## Endpoint and TLS/mTLS

Configure the endpoint through application SDK bootstrap. Use TLS for protected remote endpoints and mTLS when the deployment policy requires client certificates.

## Metadata auth

Use metadata providers for authorization, access-token, traceparent, idempotency-key, and x-request-hash. Application code should inject providers through SDK bootstrap instead of assembling raw metadata in business modules.

## Deadline and cancellation

Set a deadline for each RPC call through the generated deadline helpers or the language transport options. Callers should pass cancellation through the platform-native signal when available.

## Unary call example

```ts
import { createRpcIdempotencyMetadata, createStaticMetadataProvider, resolveRpcDeadlineMs } from './src/index.js';

const metadataProvider = createStaticMetadataProvider({
  authorization: 'Bearer <auth-token>',
  'access-token': '<access-token>',
  'idempotency-key': 'create-message-001',
});
const deadlineMs = resolveRpcDeadlineMs({ timeoutMs: 5000 });
const idempotencyMetadata = createRpcIdempotencyMetadata({ idempotencyKey: 'create-message-001' });
// Call GroupKnowledgeSpaceLifecycleService.EnsureGroupKnowledgeSpace with metadataProvider, idempotencyMetadata, and deadlineMs using the generated protobuf client.
```

## Regeneration evidence

RPC generation defaults to convention-first source output and does not write persisted generator evidence in normal generated language workspaces.

Use `sdkgen inspect --protocol rpc` to verify the RPC SDK family name, language workspace name, RPC manifest, proto source reference, generated client files, and native package manifest. Add `--emit-control-plane` only when release, CI, audit, or migration workflows need persisted generator evidence; the evidence paths are derived by generator convention.

## Verification commands

- buf lint
- buf breaking
- sdkgen generate --protocol rpc --dry-run
- run the generated client compile command for this language
