#!/bin/bash

# Script de diagnóstico para o ambiente fiscal real via Docker

echo "=================================================="
echo "    DIAGNÓSTICO FISCAL REAL (LINUX / DOCKER)      "
echo "=================================================="

# Verifica se o docker-compose está disponível
if ! command -v docker compose &> /dev/null; then
    echo "[ERRO] Docker Compose não encontrado."
    exit 1
fi

cd docker/fiscal || { echo "[ERRO] Diretório docker/fiscal não encontrado."; exit 1; }

echo "[INFO] Construindo a imagem Docker com as dependências nativas..."
docker compose build

echo ""
echo "[INFO] 1. Rodando Cargo Check Padrão (Sem features)..."
docker compose run --rm aureon-fiscal-builder cargo check -p aureon-api-local

echo ""
echo "[INFO] 2. Rodando Cargo Check com feature 'fiscal_real'..."
docker compose run --rm aureon-fiscal-builder cargo check -p aureon-api-local --features fiscal_real

echo ""
echo "[INFO] 3. Rodando Cargo Check com 'fiscal_real,fiscal_xmldsig_real'..."
docker compose run --rm aureon-fiscal-builder cargo check -p aureon-api-local --features fiscal_real,fiscal_xmldsig_real

echo ""
echo "=================================================="
echo "                   FINALIZADO                     "
echo "=================================================="
