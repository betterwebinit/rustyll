#!/bin/bash

# Script para criar pacotes de distribuição do Rustyll
# Cria binários para diferentes plataformas e arquiteturas

set -e

VERSION=$(cat VERSION || echo "1.0.0")
PROJECT_NAME="rustyll"
DIST_DIR="dist"

# Cores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}╔══════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║   Rustyll Distribution Builder v$VERSION    ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════╝${NC}"
echo ""

# Limpar distribuições antigas
echo -e "${YELLOW}[•]${NC} Limpando distribuições antigas..."
rm -rf $DIST_DIR
mkdir -p $DIST_DIR
mkdir -p $DIST_DIR/packages

# Detectar sistema operacional
OS=$(uname -s)
ARCH=$(uname -m)

echo -e "${BLUE}Sistema detectado: $OS $ARCH${NC}"
echo ""

# Função para criar pacote
create_package() {
    local target=$1
    local os_name=$2
    local arch_name=$3
    local extension=$4

    echo -e "${YELLOW}[•]${NC} Construindo para $os_name-$arch_name..."

    # Tentar compilar para o target
    if cargo build --release --target "$target" 2>/dev/null; then
        local binary_name="${PROJECT_NAME}${extension}"
        local binary_path="target/$target/release/$binary_name"

        if [ -f "$binary_path" ]; then
            # Criar diretório do pacote
            local package_dir="$DIST_DIR/${PROJECT_NAME}-${VERSION}-${os_name}-${arch_name}"
            mkdir -p "$package_dir"

            # Copiar binário
            cp "$binary_path" "$package_dir/"

            # Strip symbols (se não for Windows)
            if [[ "$extension" != ".exe" ]] && command -v strip &> /dev/null; then
                strip "$package_dir/$binary_name" 2>/dev/null || true
            fi

            # Adicionar README
            cat > "$package_dir/README.md" << EOF
# Rustyll v$VERSION - $os_name $arch_name

## Instalação

1. Extraia o arquivo
2. Torne o binário executável (Linux/macOS):
   \`chmod +x $binary_name\`
3. Mova para um diretório no PATH:
   \`sudo mv $binary_name /usr/local/bin/rustyll\`

## Uso

\`\`\`bash
rustyll --help
rustyll new meu-site
rustyll build
rustyll serve
\`\`\`

## Documentação

https://github.com/rustyll/rustyll
EOF

            # Adicionar LICENSE se existir
            if [ -f "LICENSE" ]; then
                cp LICENSE "$package_dir/"
            fi

            # Criar arquivo tar.gz
            cd "$DIST_DIR"
            tar -czf "packages/${PROJECT_NAME}-${VERSION}-${os_name}-${arch_name}.tar.gz" \
                "${PROJECT_NAME}-${VERSION}-${os_name}-${arch_name}"
            cd ..

            # Criar arquivo zip também (mais amigável para Windows)
            if command -v zip &> /dev/null; then
                cd "$DIST_DIR"
                zip -qr "packages/${PROJECT_NAME}-${VERSION}-${os_name}-${arch_name}.zip" \
                    "${PROJECT_NAME}-${VERSION}-${os_name}-${arch_name}"
                cd ..
            fi

            echo -e "${GREEN}  ✓${NC} Pacote criado: ${PROJECT_NAME}-${VERSION}-${os_name}-${arch_name}"
        else
            echo -e "${RED}  ✗${NC} Binário não encontrado para $target"
        fi
    else
        # Tentar build normal se o target falhar
        if [[ "$target" == "$ARCH-unknown-linux-gnu" ]] || [[ "$OS" == "Darwin" && "$target" == *"apple-darwin" ]]; then
            echo -e "${YELLOW}  →${NC} Usando build padrão para plataforma atual..."
            cargo build --release

            local binary_name="${PROJECT_NAME}${extension}"
            local binary_path="target/release/$binary_name"

            if [ -f "$binary_path" ]; then
                local package_dir="$DIST_DIR/${PROJECT_NAME}-${VERSION}-${os_name}-${arch_name}"
                mkdir -p "$package_dir"
                cp "$binary_path" "$package_dir/"

                # Strip symbols
                if command -v strip &> /dev/null; then
                    strip "$package_dir/$binary_name" 2>/dev/null || true
                fi

                # Criar tar.gz
                cd "$DIST_DIR"
                tar -czf "packages/${PROJECT_NAME}-${VERSION}-${os_name}-${arch_name}.tar.gz" \
                    "${PROJECT_NAME}-${VERSION}-${os_name}-${arch_name}"
                cd ..

                echo -e "${GREEN}  ✓${NC} Pacote criado: ${PROJECT_NAME}-${VERSION}-${os_name}-${arch_name}"
            fi
        else
            echo -e "${YELLOW}  →${NC} Target $target não disponível (instale com rustup target add $target)"
        fi
    fi
}

# Instalar targets comuns (opcional)
install_targets() {
    echo -e "${YELLOW}[•]${NC} Deseja instalar targets adicionais? (s/n)"
    read -r response
    if [[ "$response" == "s" ]]; then
        # Linux
        rustup target add x86_64-unknown-linux-gnu 2>/dev/null || true
        rustup target add aarch64-unknown-linux-gnu 2>/dev/null || true

        # macOS
        rustup target add x86_64-apple-darwin 2>/dev/null || true
        rustup target add aarch64-apple-darwin 2>/dev/null || true

        # Windows
        rustup target add x86_64-pc-windows-gnu 2>/dev/null || true
        rustup target add x86_64-pc-windows-msvc 2>/dev/null || true
    fi
}

# Compilar para diferentes plataformas
echo -e "${BLUE}═══ Iniciando compilação ═══${NC}"
echo ""

# Linux x86_64
create_package "x86_64-unknown-linux-gnu" "linux" "x86_64" ""

# Linux ARM64
create_package "aarch64-unknown-linux-gnu" "linux" "aarch64" ""

# macOS x86_64
create_package "x86_64-apple-darwin" "macos" "x86_64" ""

# macOS ARM64 (Apple Silicon)
create_package "aarch64-apple-darwin" "macos" "aarch64" ""

# Windows x86_64
create_package "x86_64-pc-windows-gnu" "windows" "x86_64" ".exe"

echo ""
echo -e "${BLUE}═══ Gerando checksums ═══${NC}"

# Gerar checksums
cd "$DIST_DIR/packages"
if ls *.tar.gz 1> /dev/null 2>&1; then
    sha256sum *.tar.gz > SHA256SUMS.txt
    echo -e "${GREEN}✓${NC} SHA256SUMS.txt criado"
fi

if ls *.zip 1> /dev/null 2>&1; then
    sha256sum *.zip >> SHA256SUMS.txt 2>/dev/null || true
fi
cd ../..

# Criar installer script
echo -e "${YELLOW}[•]${NC} Criando script de instalação..."
cat > "$DIST_DIR/install.sh" << 'EOF'
#!/bin/bash

# Rustyll installer script
set -e

VERSION="1.0.0"
REPO="https://github.com/rustyll/rustyll"

# Detectar OS e arquitetura
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
    linux*)
        OS="linux"
        ;;
    darwin*)
        OS="macos"
        ;;
    *)
        echo "Sistema operacional não suportado: $OS"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64)
        ARCH="x86_64"
        ;;
    aarch64|arm64)
        ARCH="aarch64"
        ;;
    *)
        echo "Arquitetura não suportada: $ARCH"
        exit 1
        ;;
esac

PACKAGE="rustyll-${VERSION}-${OS}-${ARCH}.tar.gz"
URL="${REPO}/releases/download/v${VERSION}/${PACKAGE}"

echo "Baixando Rustyll v${VERSION} para ${OS}-${ARCH}..."
curl -L -o "/tmp/${PACKAGE}" "$URL"

echo "Extraindo..."
tar -xzf "/tmp/${PACKAGE}" -C /tmp

echo "Instalando..."
sudo mv "/tmp/rustyll-${VERSION}-${OS}-${ARCH}/rustyll" /usr/local/bin/rustyll
sudo chmod +x /usr/local/bin/rustyll

echo "Limpando..."
rm -rf "/tmp/${PACKAGE}" "/tmp/rustyll-${VERSION}-${OS}-${ARCH}"

echo "Rustyll instalado com sucesso!"
rustyll --version
EOF

chmod +x "$DIST_DIR/install.sh"

# Estatísticas finais
echo ""
echo -e "${CYAN}╔══════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║           Build Completo!                ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════╝${NC}"
echo ""

if [ -d "$DIST_DIR/packages" ]; then
    echo -e "${GREEN}Pacotes criados:${NC}"
    ls -lh "$DIST_DIR/packages/" | grep -E "tar.gz|zip" | awk '{print "  • " $9 " (" $5 ")"}'

    echo ""
    echo -e "${BLUE}Total:${NC} $(ls "$DIST_DIR/packages/"*.tar.gz 2>/dev/null | wc -l) pacotes"
    echo -e "${BLUE}Diretório:${NC} $DIST_DIR/packages/"
fi

echo ""
echo -e "${YELLOW}Próximos passos:${NC}"
echo "  1. Teste os binários: ./test_rustyll.sh"
echo "  2. Publique no GitHub: ./scripts/publish_release.sh"
echo "  3. Ou distribua manualmente os arquivos em $DIST_DIR/packages/"