<#
.SYNOPSIS
Cria as pastas base do Aureon PDV no Windows.
#>

$Dirs = @(
    "C:\Aureon\data",
    "C:\Aureon\backups",
    "C:\Aureon\logs",
    "C:\Aureon\print-sim",
    "C:\Aureon\diagnostics"
)

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "   CRIANDO ESTRUTURA DE DIRETÓRIOS" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host ""

foreach ($dir in $Dirs) {
    if (-not (Test-Path $dir)) {
        try {
            New-Item -ItemType Directory -Force -Path $dir | Out-Null
            Write-Host "[CRIADO] $dir" -ForegroundColor Green
        } catch {
            Write-Host "[ERRO] Não foi possível criar: $dir" -ForegroundColor Red
        }
    } else {
        Write-Host "[EXISTE] $dir" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "Processo finalizado." -ForegroundColor Cyan
