$ErrorActionPreference = "Stop"

$packages = @(
    "sdkwork-knowledgebase-contract",
    "sdkwork-knowledgebase-core",
    "sdkwork-knowledgebase-drive",
    "sdkwork-knowledgebase-product",
    "sdkwork-knowledgebase-storage-sqlx",
    "sdkwork-knowledgebase-test-support"
)

foreach ($package in $packages) {
    cargo fmt -p $package --check
}

cargo test --workspace
powershell -ExecutionPolicy Bypass -File tools/verify_openapi_operation_ids.ps1

Write-Host "SDKWork Knowledgebase backend foundation verification passed."
