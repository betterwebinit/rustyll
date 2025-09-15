#!/bin/bash

# Script de instalação local do Rustyll
# Instala o Rustyll no sistema local

set -e

# Cores
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "======================================"
echo "   Instalação Local do Rustyll"
echo "======================================"
echo ""

# Verificar se cargo está instalado
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Erro: Cargo não está instalado${NC}"
    echo "Instale Rust em: https://rustup.rs/"
    exit 1
fi

# Compilar em modo release
echo -e "${YELLOW}Compilando Rustyll...${NC}"
cargo build --release

# Verificar se a compilação foi bem sucedida
if [ ! -f "target/release/rustyll" ]; then
    echo -e "${RED}Erro: Falha na compilação${NC}"
    exit 1
fi

# Copiar para /usr/local/bin
echo -e "${YELLOW}Instalando Rustyll...${NC}"
sudo cp target/release/rustyll /usr/local/bin/rustyll
sudo chmod +x /usr/local/bin/rustyll

# Verificar instalação
if command -v rustyll &> /dev/null; then
    echo -e "${GREEN}✓ Rustyll instalado com sucesso!${NC}"
    echo ""
    rustyll --version
    echo ""
    echo "Use 'rustyll --help' para ver os comandos disponíveis"
else
    echo -e "${RED}Erro: Instalação falhou${NC}"
    exit 1
fi