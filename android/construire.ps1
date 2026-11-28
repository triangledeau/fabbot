# Construit l'APK d'Arsenal sous Windows (docs/android.md, section 8.2).
# Même travail que android/construire.sh : lire les explications là-bas.
#
#   powershell -ExecutionPolicy Bypass -File android\construire.ps1
#   → android\app\build\outputs\apk\release\app-release.apk
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

$cibles = if ($env:CIBLES) { $env:CIBLES -split " " } else { @("arm64-v8a", "x86_64") }
$triplets = @{ "arm64-v8a" = "aarch64-linux-android"; "x86_64" = "x86_64-linux-android"; "armeabi-v7a" = "armv7-linux-androideabi" }
$args_ndk = @()
foreach ($c in $cibles) {
    if (-not $triplets.ContainsKey($c)) { throw "Processeur inconnu : $c" }
    rustup target add $triplets[$c] | Out-Null
    $args_ndk += @("-t", $c)
}

Write-Host "== 1. Bibliothèque Rust ($($cibles -join ' '))"
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_RUSTFLAGS = "-C link-arg=-Wl,-z,max-page-size=16384"
$env:CARGO_TARGET_X86_64_LINUX_ANDROID_RUSTFLAGS = "-C link-arg=-Wl,-z,max-page-size=16384"
$env:CARGO_PROFILE_RELEASE_STRIP = "symbols"
Remove-Item -Recurse -Force android\app\src\main\jniLibs -ErrorAction SilentlyContinue
cargo ndk @args_ndk -o android/app/src/main/jniLibs build --release -p fab-android
if ($LASTEXITCODE -ne 0) { throw "La compilation Rust a échoué." }

Write-Host "== 2. Textes officiels des cartes"
$csv = "data\cards\sources\public_card_data.csv"
if (-not (Test-Path $csv)) {
    New-Item -ItemType Directory -Force (Split-Path $csv) | Out-Null
    try { Invoke-WebRequest -Uri "https://d1pn9y8e99aays.cloudfront.net/public_card_data.csv" -OutFile $csv }
    catch { Write-Host "Téléchargement impossible : l'APK sera fait sans les textes des cartes." }
}
$assets = "android\app\src\main\assets\donnees"
Remove-Item -Recurse -Force $assets -ErrorAction SilentlyContinue
if (Test-Path $csv) {
    New-Item -ItemType Directory -Force $assets | Out-Null
    Copy-Item $csv $assets
    Write-Host "Textes rangés dans l'APK."
}

Write-Host "== 3. APK"
Push-Location android
try {
    .\gradlew.bat --no-daemon assembleRelease
    if ($LASTEXITCODE -ne 0) { throw "Gradle a échoué." }
} finally { Pop-Location }
Write-Host "APK prêt : android\app\build\outputs\apk\release\app-release.apk"
