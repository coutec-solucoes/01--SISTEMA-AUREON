# ============================================================
# setup-dev.ps1 — Configuração do ambiente de desenvolvimento
# Aureon Sistema Inteligente — Fase 0
# ============================================================

Write-Host "=== Aureon — Setup de Desenvolvimento ===" -ForegroundColor Cyan
Write-Host ""

# Verifica Rust
Write-Host "Verificando Rust..." -ForegroundColor Yellow
$rustVersion = rustc --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERRO] Rust nao encontrado. Instale em: https://rustup.rs" -ForegroundColor Red
    exit 1
}
Write-Host "[OK] $rustVersion" -ForegroundColor Green

# Verifica .NET
Write-Host "Verificando .NET..." -ForegroundColor Yellow
$dotnetVersion = dotnet --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERRO] .NET SDK nao encontrado. Instale o .NET 8 em: https://dotnet.microsoft.com" -ForegroundColor Red
    exit 1
}
Write-Host "[OK] .NET $dotnetVersion" -ForegroundColor Green

# Verifica Tauri CLI
Write-Host "Verificando Tauri CLI..." -ForegroundColor Yellow
$tauriVersion = cargo tauri --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "[AVISO] Tauri CLI nao encontrado. Instalando..." -ForegroundColor Yellow
    cargo install tauri-cli
}
else {
    Write-Host "[OK] $tauriVersion" -ForegroundColor Green
}

# Cria .env da API se nao existir
$envPath = "services\aureon-api-local\.env"
if (-not (Test-Path $envPath)) {
    Copy-Item "services\aureon-api-local\.env.example" $envPath
    Write-Host "[INFO] Criado $envPath — edite com suas configuracoes locais" -ForegroundColor Cyan
}

# Compila as crates base
Write-Host ""
Write-Host "Compilando crates base..." -ForegroundColor Yellow
cargo build -p aureon-core -p aureon-shared -p aureon-domain -p aureon-infra -p aureon-sync
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERRO] Falha na compilacao das crates base" -ForegroundColor Red
    exit 1
}
Write-Host "[OK] Crates base compiladas" -ForegroundColor Green

Write-Host ""
Write-Host "=== Setup concluido! ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "Proximos passos:" -ForegroundColor White
Write-Host "  1. Edite services\aureon-api-local\.env com sua DATABASE_URL" -ForegroundColor Gray
Write-Host "  2. cargo run -p aureon-api-local   (sobe a API na porta 7070)" -ForegroundColor Gray
Write-Host "  3. cargo tauri dev --manifest-path apps/aureon-pdv/src-tauri/Cargo.toml" -ForegroundColor Gray
Write-Host ""
