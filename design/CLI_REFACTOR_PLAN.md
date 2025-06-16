# Plano de Refatoração e Expansão da CLI do Rustyll

Este documento apresenta um plano detalhado para aprimorar a interface de linha de comando (CLI) do Rustyll. As propostas a seguir foram pensadas sob a perspectiva de um engenheiro multipapel – contemplando design de UX/CLI, arquitetura de software, desenvolvimento e garantia de qualidade.

## 1. Análise de Usabilidade e Consistência

### 1.1 Revisão de Nomenclatura
- **Padronização dos comandos**: priorizar nomes completos (`build`, `serve`, etc.), mantendo _aliases_ curtos apenas quando realmente úteis e sem conflito.
- **Opções coerentes**: garantir que opções semelhantes mantenham a mesma forma em todos os comandos (ex.: `--verbose` e `--quiet`), evitando sinônimos desnecessários.

### 1.2 Estrutura de Comandos e Subcomandos
- Avaliar a criação de subcomandos agrupadores, como `rustyll config` (subcomandos `get`, `set`, `list`) e `rustyll cache` (`status`, `clear`).
- Organizar opções globais (ex.: `--config`, `--source`) no nível superior da CLI, válidas para qualquer comando.

### 1.3 Feedback ao Usuário
- Mensagens padronizadas de sucesso e erro, utilizando cores e ícones simples para melhor leitura.
- Exibir progresso em operações demoradas (barra ou _spinner_). Em builds longos, mostrar etapas concluídas.

### 1.4 Discoverability
- Tornar `rustyll help` mais completo, trazendo exemplos de uso.
- Gerar scripts de _autocomplete_ para Bash, Zsh e Fish.

### 1.5 Convenções Comuns
- Manter atalhos amplamente reconhecidos como `-h/--help`, `-V/--version`, `--dry-run` e `--force`.

## 2. Refatoração e Design de Opções

### 2.1 Consolidação de Opções
- Agrupar opções de paralelismo em um bloco único (`--parallel [markdown|sass|all]`, `--threads N`).
- Introduzir perfis de performance (`--mode dev|prod|ultra-fast`) que ajustam múltiplos parâmetros de maneira pré-definida.

### 2.2 Opções Globais vs. Locais
- Definir claramente quais opções afetam todos os comandos (`--source`, `--destination`, `--config`, `--verbose`, `--quiet`, `--trace`).
- Manter específicas de cada comando apenas as realmente necessárias, evitando duplicação desnecessária.

### 2.3 Configuração via CLI e Arquivo
- Estabelecer ordem de precedência: **CLI > _config.yml > padrão**.
- Documentar no help como as opções da CLI podem sobrescrever o arquivo de configuração.

### 2.4 Remodelagem das Opções de Performance
- Criar subcomandos ou _flags_ dedicados para cache (`rustyll cache clear`, `rustyll build --cache [markdown|sass|liquid]`).
- Melhorar a opção de _benchmark/profile_, permitindo `build --profile --output report.json`.

## 3. Expansão para Novas Funcionalidades

### 3.1 Migradores
- Novo comando: `rustyll migrate <PLATAFORMA> <ORIGEM> <DESTINO>`.
    - Opções: `--force-overwrite`, `--keep-original-assets`, `--dry-run`, `--report-file`.
    - Subcomando auxiliar: `rustyll migrate list-platforms`.

### 3.2 Gerenciamento de Temas
- Extensão do `new-theme` para `rustyll theme` com subcomandos:
    - `install <URL|NOME>`
    - `list`
    - `apply <NOME>`

### 3.3 Suporte a Plugins
- Prever `rustyll plugin` com `install`, `list`, `enable` e `disable`.
- Permitir repositórios de plugins em configuração.

### 3.4 Aprimoramentos do `doctor`
- Validar configurações e sugerir correções automáticas ou links para documentação.

## 4. Garantia de Qualidade e Testabilidade

### 4.1 Cobertura de Testes para CLI
- Implementar testes automatizados que executem comandos em ambiente isolado, verificando combinações de opções e saídas esperadas.
- Utilizar crates como `assert_cmd` e `escargot` para orquestrar execucoes de binarios durante os testes.

### 4.2 Testes de Regressão
- Garantir paridade com comportamentos já existentes e com o Jekyll sempre que aplicável.
- Manter suite de testes que previna quebra de compatibilidade.

### 4.3 Testes de Borda e Robustez
- Casos com caminhos inexistentes, permissões insuficientes e entradas inválidas.
- Uso de `--dry-run` para simular operações sem alterações reais.

### 4.4 Saídas Consistentes
- Mensagens de erro e sucesso padronizadas em todas as ferramentas.
- Estrutura de logs configurável (níveis: error, warn, info, debug, trace).

## 5. Documentação e Exemplos

### 5.1 Estrutura da Documentação
- Criar seção dedicada à CLI no manual do projeto, listando cada comando, opções, exemplos e notas de compatibilidade Jekyll.

### 5.2 Receitas (Cookbooks)
- Exemplos prontos demonstrando combinações de opções, como "Build de produção otimizado" ou "Servidor de desenvolvimento com live reload".

### 5.3 Fluxos de Trabalho Típicos
- Descrever passo a passo desde a criação de um novo site até a publicação, usando os novos comandos.

### 5.4 Atualização do TODO.md
- Registrar as tarefas propostas para acompanhamento da equipe.

## Conclusão

O plano acima visa tornar o Rustyll mais acessível, previsível e poderoso para usuários de todos os níveis, mantendo a compatibilidade com o ecossistema Jekyll sempre que benéfico. A refatoração estruturada da CLI, aliada a novas funcionalidades e ampla cobertura de testes, garantirá uma experiência robusta e moderna para a comunidade.

