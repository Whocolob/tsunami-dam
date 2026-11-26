$ErrorActionPreference = "Stop"

Write-Host "1/3: Проверка Rust target aarch64-linux-android..." -ForegroundColor Cyan
rustup target add aarch64-linux-android

Write-Host "2/3: Сборка .so через cargo-ndk (arm64-v8a)..." -ForegroundColor Cyan
$jniDir = Join-Path $PSScriptRoot "android\app\src\main\jniLibs"
New-Item -ItemType Directory -Force -Path $jniDir | Out-Null

cargo ndk -t arm64-v8a -o $jniDir build --release --lib

Write-Host "3/3: Сборка APK через Gradle..." -ForegroundColor Cyan
Push-Location (Join-Path $PSScriptRoot "android")
try {
    if (Test-Path ".\gradlew.bat") {
        .\gradlew.bat assembleRelease
    } else {
        gradle assembleRelease
    }
} finally {
    Pop-Location
}

$apk = Join-Path $PSScriptRoot "android\app\build\outputs\apk\release\app-release.apk"
Write-Host "Готово! APK: $apk" -ForegroundColor Green
