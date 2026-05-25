<#
.SYNOPSIS
Script para acionar o diagnóstico de build fiscal real via Docker a partir do Windows.
#>

Write-Host "DIAGNOSTICO FISCAL REAL (WINDOWS -> DOCKER)"

# Verifica se o Docker está rodando
$null = (docker info 2>&1)
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERRO] Docker Desktop nao parece estar rodando."
    exit 1
}

Push-Location "docker\fiscal"

Write-Host "[INFO] Construindo a imagem Docker..."
docker compose build

Write-Host "[INFO] 1. Rodando Cargo Check Padrao..."
docker compose run --rm aureon-fiscal-builder cargo check -p aureon-api-local

Write-Host "[INFO] 2. Rodando Cargo Check com fiscal_real..."
docker compose run --rm aureon-fiscal-builder cargo check -p aureon-api-local --features fiscal_real

Write-Host "[INFO] 3. Rodando Cargo Check com fiscal_real e fiscal_xmldsig_real..."
docker compose run --rm aureon-fiscal-builder cargo check -p aureon-api-local --features fiscal_real,fiscal_xmldsig_real

Pop-Location

Write-Host "FINALIZADO"
