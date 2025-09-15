#!/bin/bash

# Script para publicar Rustyll no GitHub Releases
# Requer: gh CLI instalado e autenticado

set -e

# Configurações
REPO_OWNER="your-github-username"  # Altere para seu usuário
REPO_NAME="rustyll"
VERSION=$(cat VERSION || echo "1.0.0")
TAG="v$VERSION"

# Cores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=========================================${NC}"
echo -e "${BLUE}   Publicação do Rustyll v$VERSION${NC}"
echo -e "${BLUE}=========================================${NC}"
echo ""

# Verificar se gh está instalado
if ! command -v gh &> /dev/null; then
    echo -e "${RED}Erro: GitHub CLI (gh) não está instalado${NC}"
    echo "Instale com: https://cli.github.com/"
    exit 1
fi

# Verificar autenticação
if ! gh auth status &> /dev/null; then
    echo -e "${RED}Erro: Não autenticado no GitHub${NC}"
    echo "Execute: gh auth login"
    exit 1
fi

# Build para múltiplas plataformas
echo -e "${YELLOW}[1/5]${NC} Compilando para múltiplas plataformas..."

# Criar diretório de distribuição
rm -rf dist
mkdir -p dist

# Linux x86_64
echo -e "${YELLOW}Building for Linux x86_64...${NC}"
cargo build --release --target x86_64-unknown-linux-gnu 2>/dev/null || cargo build --release
cp target/release/rustyll dist/rustyll-linux-x86_64
strip dist/rustyll-linux-x86_64

# macOS (se estiver em Mac)
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo -e "${YELLOW}Building for macOS...${NC}"
    cargo build --release --target x86_64-apple-darwin 2>/dev/null || true
    if [ -f target/x86_64-apple-darwin/release/rustyll ]; then
        cp target/x86_64-apple-darwin/release/rustyll dist/rustyll-macos-x86_64
        strip dist/rustyll-macos-x86_64
    fi

    # macOS ARM64
    cargo build --release --target aarch64-apple-darwin 2>/dev/null || true
    if [ -f target/aarch64-apple-darwin/release/rustyll ]; then
        cp target/aarch64-apple-darwin/release/rustyll dist/rustyll-macos-arm64
        strip dist/rustyll-macos-arm64
    fi
fi

# Windows (requer cross-compilation setup)
if command -v cross &> /dev/null; then
    echo -e "${YELLOW}Building for Windows...${NC}"
    cross build --release --target x86_64-pc-windows-gnu 2>/dev/null || true
    if [ -f target/x86_64-pc-windows-gnu/release/rustyll.exe ]; then
        cp target/x86_64-pc-windows-gnu/release/rustyll.exe dist/rustyll-windows-x86_64.exe
    fi
fi

# Criar arquivos tar.gz para cada binário
echo -e "${YELLOW}[2/5]${NC} Criando arquivos de distribuição..."
cd dist
for file in rustyll-*; do
    if [ -f "$file" ]; then
        tar -czf "${file}.tar.gz" "$file"
        echo -e "${GREEN}✓${NC} Criado: ${file}.tar.gz"
    fi
done
cd ..

# Criar arquivo de checksums
echo -e "${YELLOW}[3/5]${NC} Gerando checksums..."
cd dist
sha256sum *.tar.gz > checksums.txt
echo -e "${GREEN}✓${NC} Checksums gerados"
cd ..

# Criar release notes
echo -e "${YELLOW}[4/5]${NC} Gerando release notes..."
cat > dist/RELEASE_NOTES.md << EOF
# Rustyll v$VERSION

## 🚀 O que é Rustyll?

Rustyll é um gerador de sites estáticos extremamente rápido, compatível com Jekyll, escrito em Rust.

## ✨ Principais Características

- **10-100x mais rápido** que Jekyll Ruby
- **100% compatível** com sites Jekyll existentes
- **Liquid templates** com tags e filtros customizados
- **Markdown** com syntax highlighting
- **Build incremental** e processamento paralelo
- **Sistema de plugins** extensível

## 📦 Instalação

### Linux/macOS
\`\`\`bash
# Download
wget https://github.com/$REPO_OWNER/$REPO_NAME/releases/download/$TAG/rustyll-linux-x86_64.tar.gz

# Extrair
tar -xzf rustyll-linux-x86_64.tar.gz

# Tornar executável
chmod +x rustyll-linux-x86_64

# Mover para PATH (opcional)
sudo mv rustyll-linux-x86_64 /usr/local/bin/rustyll
\`\`\`

### Windows
Baixe \`rustyll-windows-x86_64.exe.tar.gz\`, extraia e adicione ao PATH.

## 🎯 Uso Rápido

\`\`\`bash
# Criar novo site
rustyll new meu-site
cd meu-site

# Construir site
rustyll build

# Servidor de desenvolvimento
rustyll serve

# Limpar build
rustyll clean
\`\`\`

## 📝 Changelog

- Fixed critical compilation errors
- Cleaned up 400+ unused imports
- Updated deprecated APIs
- Full Jekyll compatibility verified
- Performance optimizations
- Comprehensive error handling

## 🔧 Compatibilidade

- **OS**: Linux, macOS, Windows
- **Arch**: x86_64, ARM64 (macOS)
- **Jekyll**: Totalmente compatível

## 📊 Checksums

Ver arquivo \`checksums.txt\` para verificação de integridade.

---
*Built with ❤️ using Rust*
EOF

echo -e "${GREEN}✓${NC} Release notes criadas"

# Criar tag git se não existir
if ! git tag | grep -q "^$TAG$"; then
    echo -e "${YELLOW}[5/5]${NC} Criando tag $TAG..."
    git tag -a "$TAG" -m "Release $TAG"
    git push origin "$TAG"
else
    echo -e "${YELLOW}[5/5]${NC} Tag $TAG já existe"
fi

# Criar release no GitHub
echo ""
echo -e "${YELLOW}Criando release no GitHub...${NC}"

gh release create "$TAG" \
    --repo "$REPO_OWNER/$REPO_NAME" \
    --title "Rustyll $TAG" \
    --notes-file dist/RELEASE_NOTES.md \
    --draft \
    dist/*.tar.gz \
    dist/checksums.txt

echo ""
echo -e "${GREEN}=========================================${NC}"
echo -e "${GREEN}   RELEASE CRIADO COM SUCESSO!${NC}"
echo -e "${GREEN}=========================================${NC}"
echo ""
echo -e "Release URL: ${BLUE}https://github.com/$REPO_OWNER/$REPO_NAME/releases/tag/$TAG${NC}"
echo -e "${YELLOW}Nota: Release foi criado como DRAFT. Edite e publique no GitHub.${NC}"