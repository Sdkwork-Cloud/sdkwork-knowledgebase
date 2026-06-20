$ErrorActionPreference = "Stop"

$specPaths = @(
    "sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json",
    "sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json"
)

$required = @(
    "spaces.create",
    "spaces.retrieve",
    "driveImports.create",
    "ingests.create",
    "ingests.retrieve",
    "spaces.browser.list",
    "retrievals.create",
    "retrievals.retrieve",
    "contextPacks.create",
    "agentProfiles.create",
    "agentProfiles.retrieve",
    "agentProfiles.update",
    "agentProfiles.delete",
    "agentProfiles.bindings.list",
    "agentProfiles.bindings.create",
    "agentProfiles.bindings.update",
    "agentProfiles.bindings.delete",
    "agentProfiles.retrievalPreview.create",
    "sources.list",
    "sources.create",
    "documents.list",
    "documents.create",
    "documents.retrieve",
    "documents.versions.create",
    "documents.versions.list",
    "okf.bundle.index.retrieve",
    "okf.bundle.log.retrieve",
    "okf.log.entries.create",
    "okf.bundle.profile.retrieve",
    "okf.profile.create",
    "okf.queries.fileAnswer",
    "indexes.create",
    "indexes.retrieve",
    "indexes.rebuild",
    "retrievalProfiles.create",
    "retrievalProfiles.retrieve",
    "retrievalProfiles.update",
    "retrievalTraces.list",
    "retrievalTraces.retrieve",
    "providerHealth.retrieve"
)

$operationIds = New-Object System.Collections.Generic.List[string]

foreach ($specPath in $specPaths) {
    if (!(Test-Path $specPath)) {
        throw "Missing OpenAPI spec: $specPath"
    }

    $spec = Get-Content -Raw $specPath | ConvertFrom-Json
    if (!$spec.components -or !$spec.components.schemas -or $spec.components.schemas.PSObject.Properties.Count -eq 0) {
        throw "OpenAPI spec has no component schemas: $specPath"
    }

    $securitySchemes = $spec.components.securitySchemes
    if (!$securitySchemes) {
        throw "OpenAPI spec must define SDKWork v3 security schemes: $specPath"
    }

    $authToken = $securitySchemes.AuthToken
    if (!$authToken -or $authToken.type -ne "http" -or $authToken.scheme -ne "bearer") {
        throw "OpenAPI spec must define AuthToken as HTTP bearer security: $specPath"
    }

    $accessToken = $securitySchemes.AccessToken
    if (
        !$accessToken `
        -or $accessToken.type -ne "apiKey" `
        -or $accessToken.in -ne "header" `
        -or $accessToken.name -ne "Access-Token"
    ) {
        throw "OpenAPI spec must define AccessToken as Access-Token header apiKey security: $specPath"
    }

    foreach ($pathProperty in $spec.paths.PSObject.Properties) {
        foreach ($methodProperty in $pathProperty.Value.PSObject.Properties) {
            $operationId = $methodProperty.Value.operationId
            if ($operationId) {
                $operation = $methodProperty.Value

                if (!$methodProperty.Value.responses) {
                    throw "OpenAPI operation has no responses: $operationId"
                }

                $operationSecurity = $operation.security
                if (!$operationSecurity -or $operationSecurity.Count -eq 0) {
                    throw "OpenAPI operation must declare SDKWork v3 security or explicit anonymous security: $operationId"
                }

                $firstSecurity = $operationSecurity[0]
                if (
                    !$firstSecurity.PSObject.Properties["AuthToken"] `
                    -or !$firstSecurity.PSObject.Properties["AccessToken"]
                ) {
                    throw "OpenAPI operation must require both AuthToken and AccessToken: $operationId"
                }

                foreach ($errorStatus in @("400", "404")) {
                    $responseProperty = $operation.responses.PSObject.Properties[$errorStatus]
                    if ($responseProperty) {
                        $content = $responseProperty.Value.content
                        if (!$content -or !$content.PSObject.Properties["application/problem+json"]) {
                            throw "OpenAPI error response $errorStatus must include application/problem+json: $operationId"
                        }
                    }
                }

                if ($pathProperty.Name.Contains("{") -and !$methodProperty.Value.parameters) {
                    throw "OpenAPI operation with path parameters has no parameter definitions: $operationId"
                }

                $methodName = [string]$methodProperty.Name
                if (($methodName -eq "post" -or $methodName -eq "patch") -and !$methodProperty.Value.requestBody) {
                    throw "OpenAPI mutating operation has no requestBody: $operationId"
                }

                [void]$operationIds.Add([string]$operationId)
            }
        }
    }
}

$schemaNames = New-Object System.Collections.Generic.HashSet[string]
foreach ($specPath in $specPaths) {
    $spec = Get-Content -Raw $specPath | ConvertFrom-Json
    foreach ($schemaProperty in $spec.components.schemas.PSObject.Properties) {
        [void]$schemaNames.Add([string]$schemaProperty.Name)
    }
}

foreach ($operationId in $operationIds) {
    if ($operationId.Contains("_")) {
        throw "operationId contains underscore: $operationId"
    }

    if ($operationId -match "^(okfBundleIndex|okfBundleLog|okfBundleProfile|okfConcepts)") {
        throw "operationId uses flattened okf resource name: $operationId"
    }
}

foreach ($requiredId in $required) {
    if (!$operationIds.Contains($requiredId)) {
        throw "Missing required operationId: $requiredId"
    }
}

$requiredSchemas = @(
    "CreateKnowledgeSpaceRequest",
    "KnowledgeSpace",
    "KnowledgeIngestRequest",
    "IngestionJob",
    "KnowledgeDriveImportRequest",
    "ListKnowledgeBrowserRequest",
    "KnowledgeBrowserPage",
    "KnowledgeBrowserNode",
    "KnowledgeBrowserView",
    "KnowledgeBrowserNodeType",
    "KnowledgeBrowserNodePermissions",
    "KnowledgeDocument",
    "KnowledgeDocumentVersion",
    "KnowledgeSource",
    "KnowledgeOkfBundleFile",
    "KnowledgeRetrievalRequest",
    "KnowledgeRetrievalResult",
    "KnowledgeContextPackRequest",
    "KnowledgeContextPack",
    "KnowledgeMemoryContextFragment",
    "KnowledgeAgentProfile",
    "KnowledgeAgentBinding",
    "KnowledgeIndex",
    "KnowledgeRetrievalProfile",
    "KnowledgeRetrievalTrace",
    "KnowledgeProviderHealth",
    "ProblemDetails"
)

foreach ($requiredSchema in $requiredSchemas) {
    if (!$schemaNames.Contains($requiredSchema)) {
        throw "Missing required OpenAPI schema: $requiredSchema"
    }
}

Write-Host "Verified $($operationIds.Count) OpenAPI operationIds and $($schemaNames.Count) schemas."
