$summaries = @{
    # app-api
    "spaces.create" = "Create a knowledge space"
    "spaces.retrieve" = "Retrieve a knowledge space"
    "driveImports.create" = "Import a drive object into knowledgebase"
    "ingests.create" = "Create an ingestion job"
    "ingests.retrieve" = "Retrieve an ingestion job"
    "documents.list" = "List knowledge documents"
    "documents.create" = "Create a knowledge document"
    "documents.retrieve" = "Retrieve a knowledge document"
    "documents.update" = "Update a knowledge document"
    "documents.delete" = "Delete a knowledge document"
    "documents.versions.list" = "List document versions"
    "documents.versions.create" = "Create a document version"
    "wiki.pages.list" = "List wiki pages"
    "wiki.pages.retrieve" = "Retrieve a wiki page"
    "wiki.pages.revisions.list" = "List wiki page revisions"
    "wiki.index.retrieve" = "Retrieve the wiki index"
    "wiki.log.retrieve" = "Retrieve the wiki log"
    "wiki.schema.retrieve" = "Retrieve the wiki schema"
    "wiki.queries.create" = "Create a wiki query"
    "wiki.queries.fileAnswer" = "File an answer for a wiki query"
    "wiki.contextPacks.create" = "Create a wiki context pack"
    "spaces.browser.list" = "List browser view of a knowledge space"
    "retrievals.create" = "Create a knowledge retrieval"
    "retrievals.retrieve" = "Retrieve a knowledge retrieval result"
    "contextPacks.create" = "Create a knowledge context pack"
    "agentProfiles.create" = "Create a knowledge agent profile"
    "agentProfiles.retrieve" = "Retrieve a knowledge agent profile"
    "agentProfiles.update" = "Update a knowledge agent profile"
    "agentProfiles.delete" = "Delete a knowledge agent profile"
    "agentProfiles.bindings.list" = "List agent profile bindings"
    "agentProfiles.bindings.create" = "Create an agent profile binding"
    "agentProfiles.bindings.update" = "Update an agent profile binding"
    "agentProfiles.bindings.delete" = "Delete an agent profile binding"
    "agentProfiles.retrievalPreview.create" = "Preview retrieval for an agent profile"
    # backend-api
    "sources.list" = "List knowledge sources"
    "sources.create" = "Create a knowledge source"
    "wiki.compileJobs.create" = "Create a wiki compile job"
    "wiki.candidates.list" = "List wiki candidates"
    "wiki.candidates.approve" = "Approve a wiki candidate"
    "wiki.candidates.reject" = "Reject a wiki candidate"
    "wiki.pages.publish" = "Publish a wiki page"
    "wiki.schema.profiles.create" = "Create a wiki schema profile"
    "wiki.schema.profiles.update" = "Update a wiki schema profile"
    "wiki.index.rebuild" = "Rebuild the wiki index"
    "wiki.log.entries.create" = "Create a wiki log entry"
    "wiki.exports.create" = "Create a wiki export"
    "wiki.exports.retrieve" = "Retrieve a wiki export"
    "wiki.fileEntries.list" = "List wiki file entries"
    "wiki.lintRuns.create" = "Create a wiki lint run"
    "wiki.evalRuns.create" = "Create a wiki eval run"
    "indexes.create" = "Create a knowledge index"
    "indexes.retrieve" = "Retrieve a knowledge index"
    "indexes.rebuild" = "Rebuild a knowledge index"
    "retrievalProfiles.create" = "Create a retrieval profile"
    "retrievalProfiles.retrieve" = "Retrieve a retrieval profile"
    "retrievalProfiles.update" = "Update a retrieval profile"
    "retrievalTraces.list" = "List retrieval traces"
    "retrievalTraces.retrieve" = "Retrieve a retrieval trace"
    "providerHealth.retrieve" = "Retrieve provider health status"
}

$files = @(
    "sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json",
    "sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json",
    "sdks/sdkwork-knowledgebase-sdk/openapi/knowledgebase-open-api.openapi.json"
)

foreach ($file in $files) {
    $fullPath = Join-Path $PSScriptRoot $file
    if (-not (Test-Path $fullPath)) {
        $fullPath = $file
    }
    
    $content = Get-Content $fullPath -Raw
    $json = $content | ConvertFrom-Json
    
    $modified = $false
    foreach ($path in $json.paths.PSObject.Properties) {
        foreach ($method in @("get", "post", "put", "patch", "delete")) {
            if ($path.Value.PSObject.Properties[$method]) {
                $op = $path.Value.$method
                if ($op.operationId -and -not $op.summary) {
                    $opId = $op.operationId
                    if ($summaries.ContainsKey($opId)) {
                        $op | Add-Member -NotePropertyName "summary" -NotePropertyValue $summaries[$opId] -Force
                        $modified = $true
                        Write-Host "Added summary to $opId : $($summaries[$opId])"
                    } else {
                        Write-Host "WARNING: No summary mapping for $opId"
                    }
                }
            }
        }
    }
    
    if ($modified) {
        $json | ConvertTo-Json -Depth 100 | Set-Content $fullPath -Encoding UTF8NoBOM
        Write-Host "Updated $file"
    }
}
