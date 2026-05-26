<#
.SYNOPSIS
Script para compilar o Aureon PDV para Windows.

.DESCRIPTION
Realiza o build do frontend Blazor e do backend Rust (Tauri). 
Este script não assina digitalmente o código nem configura o auto-update.
Ele cria o artefato base para instalação local e empacotamento MSI/NSIS futuro.

.NOTES
Requisitos: dotnet 8.0 SDK, Cargo/Rust, Node (se necessário para tooling do Tauri).
#>

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "  BUILD AUREON PDV - WINDOWS (LOCAL)     " -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host ""

$BasePath = Split-Path -Parent $MyInvocation.MyCommand.Path | Split-Path -Parent
Set-Location $BasePath

Write-Host "[1/3] Compilando Frontend Blazor..." -ForegroundColor Yellow
dotnet build apps/aureon-pdv/ui-blazor
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERRO: Build do frontend falhou!" -ForegroundColor Red
    exit 1
}
Write-Host "Frontend compilado com sucesso." -ForegroundColor Green

Write-Host ""
Write-Host "[2/3] Checando Backend Tauri..." -ForegroundColor Yellow
cargo check -p aureon-pdv
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERRO: Check do backend falhou!" -ForegroundColor Red
    exit 1
}
Write-Host "Backend validado com sucesso." -ForegroundColor Green

Write-Host ""
Write-Host "[3/3] Compilando Binário Release (Tauri)..." -ForegroundColor Yellow
# Aqui normalmente executamos 'cargo tauri build' ou 'cargo build --release -p aureon-pdv'
# Como estamos apenas preparando o ambiente (Fase 20 Bloco 10), rodamos o cargo build local.
cargo build --release -p aureon-pdv
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERRO: Build do backend falhou!" -ForegroundColor Red
    exit 1
}
Write-Host "Backend compilado com sucesso." -ForegroundColor Green

Write-Host ""
Write-Host "Build finalizado. O executável está em: target/release/aureon-pdv.exe" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
