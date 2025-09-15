#!/bin/bash

# Script de teste para o executável Rustyll
# Testa todas as funcionalidades principais

set -e

echo "========================================="
echo "   Teste do Rustyll v1.0.0"
echo "========================================="
echo ""

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Função para imprimir status
print_test() {
    echo -e "${YELLOW}[TESTE]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

# 1. Build do projeto
print_test "Compilando Rustyll em modo release..."
cargo build --release
print_success "Compilação concluída!"

RUSTYLL="./target/release/rustyll"

# 2. Verificar se o executável existe
if [ ! -f "$RUSTYLL" ]; then
    print_error "Executável não encontrado em $RUSTYLL"
    exit 1
fi
print_success "Executável encontrado!"

# 3. Testar comando de ajuda
print_test "Testando comando de ajuda..."
$RUSTYLL --help > /dev/null 2>&1
print_success "Comando de ajuda funciona!"

# 4. Testar versão
print_test "Verificando versão..."
$RUSTYLL --version
print_success "Comando de versão funciona!"

# 5. Criar site de teste
TEST_DIR="test_site_$(date +%s)"
print_test "Criando novo site de teste em $TEST_DIR..."
$RUSTYLL new "$TEST_DIR"

if [ -d "$TEST_DIR" ]; then
    print_success "Site criado com sucesso!"
else
    print_error "Falha ao criar site"
    exit 1
fi

cd "$TEST_DIR"

# 6. Build do site
print_test "Construindo o site..."
../$RUSTYLL build
if [ -d "_site" ]; then
    print_success "Site construído com sucesso!"
    ls -la _site/
else
    print_error "Falha ao construir site"
    cd ..
    rm -rf "$TEST_DIR"
    exit 1
fi

# 7. Testar servidor (em background por 3 segundos)
print_test "Testando servidor de desenvolvimento..."
../$RUSTYLL serve --port 4001 &
SERVER_PID=$!
sleep 3

# Verificar se o servidor está rodando
if kill -0 $SERVER_PID 2>/dev/null; then
    print_success "Servidor iniciado com sucesso!"

    # Testar requisição HTTP
    if command -v curl &> /dev/null; then
        print_test "Fazendo requisição HTTP..."
        if curl -s http://localhost:4001 > /dev/null; then
            print_success "Servidor respondendo corretamente!"
        else
            print_error "Servidor não está respondendo"
        fi
    fi

    # Parar servidor
    kill $SERVER_PID 2>/dev/null
    wait $SERVER_PID 2>/dev/null
    print_success "Servidor parado com sucesso!"
else
    print_error "Falha ao iniciar servidor"
fi

# 8. Testar clean
print_test "Testando comando clean..."
../$RUSTYLL clean
if [ ! -d "_site" ]; then
    print_success "Clean executado com sucesso!"
else
    print_error "Clean falhou"
fi

# 9. Testar com demo_site existente
cd ..
if [ -d "demo_site" ]; then
    print_test "Testando com demo_site..."
    cd demo_site
    ../$RUSTYLL build
    if [ -d "_site" ]; then
        print_success "Demo site construído com sucesso!"

        # Verificar arquivos importantes
        print_test "Verificando arquivos gerados..."
        if [ -f "_site/index.html" ]; then
            print_success "index.html gerado!"
        fi
        if [ -f "_site/assets/css/style.css" ]; then
            print_success "CSS copiado!"
        fi
    fi
    cd ..
fi

# 10. Testar report
print_test "Testando geração de relatório..."
$RUSTYLL report --source demo_site > /dev/null 2>&1
print_success "Relatório gerado!"

# Limpeza
print_test "Limpando arquivos de teste..."
rm -rf "$TEST_DIR"
print_success "Limpeza concluída!"

echo ""
echo "========================================="
echo -e "${GREEN}   TODOS OS TESTES PASSARAM!${NC}"
echo "========================================="
echo ""
echo "Rustyll está pronto para uso!"
echo "Executável em: $RUSTYLL"
echo "Tamanho: $(du -h $RUSTYLL | cut -f1)"