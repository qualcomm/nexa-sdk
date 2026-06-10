# geniex_benchmark entry script for QDC Windows (POWERSHELL framework).
#
# QDC extracts the artifact zip to C:\Temp\TestContent\ and runs this via
# PowerShell. Anything under C:\Temp\QDC_Logs\ is auto-uploaded. run_qdc_jobs.py
# fills the here-string below with `name|plugin|csv_devices|url|kind` lines.
#
# We sweep ctx in {512, 1024, 4096} per cell to align with test-llama.cpp's
# PERFORMANCE SESSION; each ctx gets its own pre-trimmed prompt file.

$ErrorActionPreference = "Continue"

$LOG = "C:\Temp\QDC_Logs"
$OUT = "$LOG\results"
$MODELS = "C:\Temp\models"
$TC = "C:\Temp\TestContent"
$BUNDLE = "$TC\pkg-geniex"
$PROMPTS = "$TC\prompts"

New-Item -ItemType Directory -Force -Path $LOG, $OUT, $MODELS | Out-Null
Start-Transcript -Path "$LOG\script.log" -Force | Out-Null

# Trust the self-signed cert the HTP .cat catalogs are signed with, or the
# Hexagon backends fail their code-integrity check at load.
$cert = "$TC\ggml-htp-v1.cer"
if (Test-Path $cert) {
    & certutil.exe -addstore -f Root $cert | Out-Null
    & certutil.exe -addstore -f TrustedPublisher $cert | Out-Null
}

Set-Location $BUNDLE
$env:GENIEX_PLUGIN_PATH = "$BUNDLE\lib"
$env:PATH = "$BUNDLE\lib;$BUNDLE\lib\llama_cpp;$BUNDLE\lib\qairt;$BUNDLE\lib\qairt\htp-files;$env:PATH"

$rows = @'
{MODELS}
'@ -split "`n" | ForEach-Object { $_.Trim() } | Where-Object { $_ }

$IMG = "$TC/test.png" -replace '\\', '/'

$ctxList = @(512, 1024, 4096)
$tsvByCtx = @{}
foreach ($ctx in $ctxList) {
    $tsvByCtx[$ctx] = "C:\Temp\matrix-$ctx.tsv"
    Remove-Item $tsvByCtx[$ctx] -ErrorAction SilentlyContinue
}

foreach ($row in $rows) {
    $name, $plugin, $devs, $url, $kind, $mmproj_url, $vlm, $image = $row -split '\|'
    $dir = "$MODELS\$name"
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    Write-Output "=== fetch $name ($kind) ==="
    if ($kind -eq "bundle") {
        $mpath = "$dir\bundle"
        if (-not (Test-Path $mpath)) {
            try {
                Invoke-WebRequest -Uri $url -OutFile "$dir\b.zip"
                Expand-Archive -Path "$dir\b.zip" -DestinationPath $mpath -Force
                $entries = Get-ChildItem $mpath
                if ($entries.Count -eq 1 -and $entries[0].PSIsContainer) {
                    Move-Item "$($entries[0].FullName)\*" $mpath
                    Remove-Item $entries[0].FullName
                }
            } catch {
                Write-Output "WARNING: $name fetch failed, skipping"
                Remove-Item $mpath -Recurse -ErrorAction SilentlyContinue
                continue
            }
        }
    } else {
        $mpath = "$dir\model.gguf"
        if (-not (Test-Path $mpath)) {
            try {
                Invoke-WebRequest -Uri $url -OutFile $mpath
            } catch {
                Write-Output "WARNING: $name fetch failed, skipping"
                continue
            }
        }
    }
    $mmpathfwd = ""
    if ($mmproj_url) {
        $mmpath = "$dir\mmproj.gguf"
        if (-not (Test-Path $mmpath)) {
            try {
                Invoke-WebRequest -Uri $mmproj_url -OutFile $mmpath
            } catch {
                Write-Output "WARNING: $name mmproj fetch failed, skipping"
                continue
            }
        }
        $mmpathfwd = $mmpath -replace '\\', '/'
    }
    $imgpath = if ($image -eq "1") { $IMG } else { "" }
    $mpathfwd = $mpath -replace '\\', '/'
    foreach ($d in $devs -split ',') {
        foreach ($ctx in $ctxList) {
            "{0}-{1}-{2}-c{3}`t{1}`t{2}`t{4}`t`t{5}`t{6}`t{7}" -f `
                $name, $plugin, $d, $ctx, $mpathfwd, $mmpathfwd, $imgpath, $vlm `
                | Add-Content $tsvByCtx[$ctx]
        }
    }
}

foreach ($ctx in $ctxList) {
    $tsv = $tsvByCtx[$ctx]
    $prompt = "$PROMPTS\sample_prompt_$ctx.txt"
    Write-Output "=== matrix ctx=$ctx ==="
    if (Test-Path $tsv) { Get-Content $tsv }
    & "$BUNDLE\bin\geniex_benchmark.exe" --matrix-file $tsv --output-json-dir "$OUT" -r 3 `
        -c $ctx --prompt-file $prompt --reset-between-runs
    Write-Output "rc=$LASTEXITCODE  ($((Get-ChildItem $OUT).Count) cell json files so far)"
}
Write-Output "=== done ==="
Stop-Transcript | Out-Null
exit 0
