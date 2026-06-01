$ErrorActionPreference = "Stop"

$specPaths = @(
    "sdks/sdkwork-knowledgebase-app-api/openapi/knowledgebase-app-api.openapi.json",
    "sdks/sdkwork-knowledgebase-backend-api/openapi/knowledgebase-backend-api.openapi.json"
)

$required = @(
    "spaces.create",
    "spaces.retrieve",
    "driveImports.create",
    "ingests.create",
    "ingests.retrieve",
    "sources.list",
    "sources.create",
    "documents.list",
    "documents.create",
    "documents.retrieve",
    "documents.versions.create",
    "documents.versions.list",
    "wiki.index.retrieve",
    "wiki.log.retrieve",
    "wiki.log.entries.create",
    "wiki.schema.retrieve",
    "wiki.schema.profiles.create",
    "wiki.queries.fileAnswer"
)

$operationIds = New-Object System.Collections.Generic.List[string]

foreach ($specPath in $specPaths) {
    if (!(Test-Path $specPath)) {
        throw "Missing OpenAPI spec: $specPath"
    }

    $spec = Get-Content -Raw $specPath | ConvertFrom-Json
    foreach ($pathProperty in $spec.paths.PSObject.Properties) {
        foreach ($methodProperty in $pathProperty.Value.PSObject.Properties) {
            $operationId = $methodProperty.Value.operationId
            if ($operationId) {
                [void]$operationIds.Add([string]$operationId)
            }
        }
    }
}

foreach ($operationId in $operationIds) {
    if ($operationId.Contains("_")) {
        throw "operationId contains underscore: $operationId"
    }

    if ($operationId -match "^(wikiIndex|wikiLog|wikiSchema|wikiPages)") {
        throw "operationId uses flattened wiki resource name: $operationId"
    }
}

foreach ($requiredId in $required) {
    if (!$operationIds.Contains($requiredId)) {
        throw "Missing required operationId: $requiredId"
    }
}

Write-Host "Verified $($operationIds.Count) OpenAPI operationIds."
