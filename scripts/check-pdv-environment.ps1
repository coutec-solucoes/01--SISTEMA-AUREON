<#
.SYNOPSIS
Verifica se as pastas e requisitos para o Aureon PDV estão configurados.
#>

$BaseDir = "C:\Aureon"
$Dirs = @(
    "C:\Aureon\data",
    "C:\Aureon\backups",
    "C:\Aureon\logs",
    "C:\Aureon\print-sim",
    "C:\Aureon\diagnostics"
)

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "   DIAGNÓSTICO BÁSICO AUREON PDV" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host ""

if (-not (Test-Path $BaseDir)) {
    Write-Host "[ALERTA] Diretório base C:\Aureon não existe." -ForegroundColor Red
} else {
    Write-Host "[OK] Diretório base C:\Aureon encontrado." -ForegroundColor Green
}

foreach ($dir in $Dirs) {
    if (-not (Test-Path $dir)) {
        Write-Host "[ALERTA] Subdiretório não encontrado: $dir" -ForegroundColor Yellow
    } else {
        Write-Host "[OK] Subdiretório encontrado: $dir" -ForegroundColor Green
    }
}

$DbPath = "$BaseDir\data\aureon-local.db"
if (Test-Path $DbPath) {
    Write-Host "[OK] Banco de dados encontrado em: $DbPath" -ForegroundColor Green
} else {
    Write-Host "[INFO] Banco de dados não existe ou será criado no primeiro acesso." -ForegroundColor Cyan
}

try {
    $testFile = "$BaseDir\data\.test_write"
    Set-Content -Path $testFile -Value "test"
    Remove-Item $testFile
    Write-Host "[OK] Permissão de escrita validada." -ForegroundColor Green
} catch {
    Write-Host "[ERRO] Sem permissão de escrita em $BaseDir\data. Execute como Administrador ou conceda permissões." -ForegroundColor Red
}

Write-Host ""
Write-Host "Diagnóstico finalizado." -ForegroundColor Cyan
