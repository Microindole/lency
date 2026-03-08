Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ($args.Count -ne 0) {
    throw "run_lcy_tests.ps1 不接受参数。"
}

Write-Host "Running .lcy integration tests..."
Write-Host "====================================="

$projectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$testDir = Join-Path $projectRoot "tests\\integration"

if (-not (Test-Path $testDir)) {
    Write-Host "No tests/integration directory found."
    exit 0
}

$lcyFiles = Get-ChildItem -Path $testDir -Recurse -Filter *.lcy | Sort-Object FullName
if ($lcyFiles.Count -eq 0) {
    Write-Host "No .lcy files found in tests/integration"
    exit 0
}

$pass = 0
$fail = 0
$expectedFail = 0
$expectedTodo = 0
$expectedFixme = 0
$failedFiles = @()

foreach ($file in $lcyFiles) {
    $rel = $file.FullName.Substring($projectRoot.Length + 1).Replace('\', '/')
    $firstLines = (Get-Content $file.FullName -TotalCount 5) -join "`n"

    $isExpectedFail = $firstLines -match '@expect-error'
    $isTodo = $firstLines -match '@expect-error:.*TODO'
    $isFixme = $firstLines -match '@expect-error:.*FIXME'

    cargo run --bin lencyc --quiet -- check $file.FullName *> $null
    $ok = ($LASTEXITCODE -eq 0)

    if ($ok) {
        if ($isExpectedFail) {
            Write-Host "WARN  $rel (expected to fail but passed)"
        } else {
            Write-Host "PASS  $rel"
        }
        $pass++
        continue
    }

    if ($isExpectedFail) {
        if ($isTodo) {
            Write-Host "TODO  $rel (feature not implemented)"
            $expectedTodo++
        } elseif ($isFixme) {
            Write-Host "FIXME $rel (known compiler bug)"
            $expectedFixme++
        } else {
            Write-Host "XFAIL $rel (expected failure)"
            $expectedFail++
        }
    } else {
        Write-Host "FAIL  $rel"
        $failedFiles += $rel
        $fail++
    }
}

Write-Host ""
Write-Host "====================================="
Write-Host "Results:"
Write-Host "  Passed: $pass"
Write-Host "  Expected errors: $expectedFail"
Write-Host "  TODO: $expectedTodo"
Write-Host "  FIXME: $expectedFixme"
Write-Host "  Unexpected failures: $fail"

if ($env:GITHUB_ACTIONS) {
    if ($expectedTodo -gt 0) {
        Write-Host "::warning::There are $expectedTodo TODO expected-fail tests."
    }
    if ($expectedFixme -gt 0) {
        Write-Host "::warning::There are $expectedFixme FIXME expected-fail tests."
    }
}

if ($fail -gt 0) {
    Write-Host ""
    Write-Host "Unexpected failures:"
    foreach ($f in $failedFiles) {
        Write-Host "  - $f"
    }
    exit 1
}

Write-Host ""
Write-Host "All tests passed."
