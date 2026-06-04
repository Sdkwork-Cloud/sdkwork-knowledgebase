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
    "spaces.browser.list",
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
    if (!$spec.components -or !$spec.components.schemas -or $spec.components.schemas.PSObject.Properties.Count -eq 0) {
        throw "OpenAPI spec has no component schemas: $specPath"
    }

    foreach ($pathProperty in $spec.paths.PSObject.Properties) {
        foreach ($methodProperty in $pathProperty.Value.PSObject.Properties) {
            $operationId = $methodProperty.Value.operationId
            if ($operationId) {
                if (!$methodProperty.Value.responses) {
                    throw "OpenAPI operation has no responses: $operationId"
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

    if ($operationId -match "^(wikiIndex|wikiLog|wikiSchema|wikiPages)") {
        throw "operationId uses flattened wiki resource name: $operationId"
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
    "KnowledgeWikiFileEntry",
    "ProblemDetails"
)

foreach ($requiredSchema in $requiredSchemas) {
    if (!$schemaNames.Contains($requiredSchema)) {
        throw "Missing required OpenAPI schema: $requiredSchema"
    }
}

Write-Host "Verified $($operationIds.Count) OpenAPI operationIds and $($schemaNames.Count) schemas."
