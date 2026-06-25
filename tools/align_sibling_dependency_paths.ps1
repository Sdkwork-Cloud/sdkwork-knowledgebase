# Align sibling SDKWork dependency paths for local development.
# See sdkwork-specs/DEPENDENCY_MANAGEMENT_SPEC.md §3.

param(
  [string]$WorkspaceRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
)

$ErrorActionPreference = 'Stop'

function Ensure-Junction {
    param(
        [string]$LinkPath,
        [string]$TargetPath
    )

    if (Test-Path $LinkPath) {
        Write-Host "ok: $LinkPath"
        return
    }

    if (-not (Test-Path $TargetPath)) {
        Write-Warning "skip: target missing for $LinkPath -> $TargetPath"
        return
    }

    cmd /c mklink /J "$LinkPath" "$TargetPath" | Out-Null
    Write-Host "linked: $LinkPath -> $TargetPath"
}

$aliases = @(
    @{
        Link = Join-Path $WorkspaceRoot 'sdkwork-claw-router'
        Target = Join-Path $WorkspaceRoot 'sdkwork-clawrouter'
    }
)

foreach ($entry in $aliases) {
    Ensure-Junction -LinkPath $entry.Link -TargetPath $entry.Target
}

Write-Host 'Sibling dependency path alignment complete.'
