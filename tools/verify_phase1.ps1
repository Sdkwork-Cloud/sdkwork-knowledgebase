$ErrorActionPreference = "Stop"

function Invoke-Checked {
    param(
        [Parameter(Mandatory = $true)]
        [string]$FilePath,
        [Parameter(ValueFromRemainingArguments = $true)]
        [string[]]$Arguments
    )

    & $FilePath @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Command failed with exit code ${LASTEXITCODE}: $FilePath $($Arguments -join ' ')"
    }
}

$packages = @(
    "sdkwork-knowledgebase-contract",
    "sdkwork-knowledgebase-core",
    "sdkwork-knowledgebase-drive",
    "sdkwork-knowledgebase-app-api",
    "sdkwork-knowledgebase-product",
    "sdkwork-knowledgebase-storage-sqlx",
    "sdkwork-knowledgebase-test-support"
)

foreach ($package in $packages) {
    Invoke-Checked cargo fmt -p $package --check
}

Invoke-Checked cargo test --workspace
Invoke-Checked powershell -ExecutionPolicy Bypass -File tools/verify_openapi_operation_ids.ps1

$llmWiki = Get-Content -Raw docs/llm-wiki.md
# Detect common mojibake/replacement characters without embedding them directly in this script.
if ($llmWiki.Contains([char]0x9225) -or $llmWiki.Contains([char]0x922B) -or $llmWiki.Contains([char]0xFFFD)) {
    throw "docs/llm-wiki.md contains mojibake replacement text"
}

if (!$llmWiki.Contains("Database objects created by SDKWork Knowledgebase use the") -or !$llmWiki.Contains('`kb_` prefix')) {
    throw "docs/llm-wiki.md must document the SDKWork Knowledgebase kb_ database object naming standard"
}

$databaseObjectSearchRoots = @(
    "services/sdkwork-knowledgebase-storage-sqlx/migrations",
    "services/sdkwork-knowledgebase-storage-sqlx/src",
    "services/sdkwork-knowledgebase-storage-sqlx/tests"
)

$oldDatabaseObjectPatterns = @(
    "CREATE TABLE IF NOT EXISTS knowledge_",
    "CREATE INDEX IF NOT EXISTS idx_knowledge_",
    "CREATE UNIQUE INDEX IF NOT EXISTS uk_knowledge_",
    "FROM knowledge_",
    "INSERT INTO knowledge_",
    "UPDATE knowledge_",
    "JOIN knowledge_",
    "pragma_table_info('knowledge_"
)

function Get-DefinedDatabaseObjects {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Content,
        [Parameter(Mandatory = $true)]
        [string]$Prefix
    )

    $objects = New-Object System.Collections.Generic.List[string]
    foreach ($line in $Content -split "`n") {
        $trimmed = $line.Trim()
        if ($trimmed.StartsWith($Prefix)) {
            $tail = $trimmed.Substring($Prefix.Length).Trim()
            $name = ($tail -split "\s+")[0].Trim('"')
            if (![string]::IsNullOrWhiteSpace($name)) {
                $objects.Add($name)
            }
        }
    }
    return $objects
}

foreach ($root in $databaseObjectSearchRoots) {
    foreach ($path in Get-ChildItem -Path $root -Recurse -File) {
        if ($path.Name -eq "migration_manifest.rs") {
            continue
        }

        $content = Get-Content -Raw $path.FullName
        foreach ($pattern in $oldDatabaseObjectPatterns) {
            if ($content.Contains($pattern)) {
                throw "Old knowledge_ database object usage remains in $($path.FullName): $pattern"
            }
        }

        foreach ($table in Get-DefinedDatabaseObjects -Content $content -Prefix "CREATE TABLE IF NOT EXISTS ") {
            if (!$table.StartsWith("kb_")) {
                throw "Knowledgebase database table must use kb_ prefix in $($path.FullName): $table"
            }
        }

        foreach ($index in Get-DefinedDatabaseObjects -Content $content -Prefix "CREATE INDEX IF NOT EXISTS ") {
            if (!$index.StartsWith("idx_kb_")) {
                throw "Knowledgebase database index must use idx_kb_ prefix in $($path.FullName): $index"
            }
        }

        foreach ($index in Get-DefinedDatabaseObjects -Content $content -Prefix "CREATE UNIQUE INDEX IF NOT EXISTS ") {
            if (!$index.StartsWith("uk_kb_")) {
                throw "Knowledgebase database unique index must use uk_kb_ prefix in $($path.FullName): $index"
            }
        }
    }
}

Write-Host "SDKWork Knowledgebase backend foundation verification passed."
