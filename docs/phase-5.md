# Documentação da Fase 5 — Configurações Operacionais

Este documento consolida a arquitetura, estrutura física e parâmetros funcionais implementados na **Fase 5 — Configurações Operacionais: PDV, Impressoras, Periféricos e Produção** do Aureon Sistema Inteligente.

---

## 📋 Resumo da Fase 5
A Fase 5 estabelece a fundação de dados e regras operacionais para as futuras fases de venda ativa (Fase 6 — PDV e Vendas). Foram modelados, integrados e validados todos os componentes estruturais para o PDV offline-first, periféricos físicos, setores de preparação, orçamentos, comandas e mesas.

### Blocos de Trabalho Implementados
1. **Bloco 1**: Migrations PostgreSQL (17 tabelas operacionais, índices e restrições de integridade).
2. **Bloco 2**: Endpoints Axum e rotas em Rust, utilizando autenticação por **Token Opaco UUID** (padrão de segurança estabelecido na Fase 3, sem uso de JWT) e persistência auditada.
3. **Bloco 3**: Interface administrativa Blazor WebAssembly com 15 abas unificadas sob o componente premium de navegação `SubmenuOperacionais.razor`.

### 📂 Registro de Commits
- **Bloco 1**: `1420b91` (Migrations PostgreSQL das Configurações Operacionais)
- **Bloco 2**: `edb51e7` (API Rust das Configurações Operacionais)
- **Bloco 3**: `db41e22` (UI Blazor das Configurações Operacionais)
- **Encerramento da Fase 5**: Commit final realizado após esta consolidação de documentos.

---

## 🗄️ Estrutura Física do Banco de Dados (PostgreSQL)
Implementado no arquivo `database/migrations/postgresql/008_configuracoes_operacionais.sql`:

1. **`configuracoes_pdv`**: Parâmetros de venda sem estoque, offline, descontos, alteração de preço manual e supervisão de cancelamento.
2. **`terminais_pdv`**: Dispositivos autorizados na rede (Fixo, Móvel, Totem).
3. **`registradoras`**: Gavetas/caixas físicas para transação financeira com suporte a multimoeda.
4. **`configuracoes_mesas`**: Limites globais para atendimento gourmet de mesas.
5. **`mesas`**: Mapa e capacidade de mesas individuais por setor físico.
6. **`configuracoes_comandas`**: Faixas de comandas físicas e pendências.
7. **`comandas`**: Cadastro sequencial de comandas físicas para atendimento.
8. **`configuracoes_pre_vendas`**: Prazos de expiração e reservas de estoque para pré-venda.
9. **`configuracoes_orcamentos`**: Prazos de expiração em dias e regras de identificação.
10. **`regras_vendas`**: Limites máximos de itens e valor por venda sem supervisor.
11. **`series_numeracoes`**: Controle sequencial e séries para documentos tributários.
12. **`impressoras`**: Vínculos de conexões (USB, TCP/IP, Rede) e emulação ESC/POS.
13. **`setores_producao`**: Setores de preparo vinculados a impressoras direcionadas.
14. **`balancas`**: Balanças de checkout (COM, USB, IP) e baud rates.
15. **`etiquetas_balancas`**: Máscaras de código de barras EAN-13 para extração de peso/preço total.
16. **`perifericos`**: Leitores, gavetas locais e displays torre.
17. **`paineis_senhas_chamadas`**: Configurações de telas e displays eletrônicos de senhas.

---

## 🔌 API Rust (Endpoints e Rotas)
Todas as rotas da Fase 5 consomem exclusivamente o prefixo `/configuracoes/operacionais` (nenhum endpoint consome `/configuracoes/operacoes/`):

- `GET /configuracoes/operacionais/pdv` e `POST /configuracoes/operacionais/pdv`
- `GET/POST/PUT /configuracoes/operacionais/terminais` (+ `/autorizar` e `/inativar`)
- `GET/POST/PUT /configuracoes/operacionais/registradoras` (+ `/inativar`)
- `GET/POST /configuracoes/operacionais/mesas/configuracao`
- `GET/POST/PUT /configuracoes/operacionais/mesas`
- `GET/POST /configuracoes/operacionais/comandas/configuracao`
- `GET/POST/PUT /configuracoes/operacionais/comandas`
- `GET/POST /configuracoes/operacionais/pre-vendas`
- `GET/POST /configuracoes/operacionais/orcamentos`
- `GET/POST /configuracoes/operacionais/regras-venda`
- `GET/POST/PUT /configuracoes/operacionais/series-numeracao`
- `GET/POST/PUT /configuracoes/operacionais/impressoras` (+ `/testar`)
- `GET/POST/PUT /configuracoes/operacionais/setores-producao`
- `GET/POST/PUT /configuracoes/operacionais/balancas` (+ `/ler-peso`)
- `GET/POST/PUT /configuracoes/operacionais/etiquetas-balanca`
- `GET/POST/PUT /configuracoes/operacionais/perifericos` (+ `/testar`)
- `GET/POST/PUT /configuracoes/operacionais/senhas-chamadas` (+ `/testar`)

---

## 🎨 Interface Blazor WebAssembly (15 Telas)
O menu **Configurações Operacionais** sob a URL `/configuracoes/operacionais/pdv` utiliza um componente dinâmico de abas `SubmenuOperacionais.razor` para alternar fluidamente entre as páginas de parametrização:
- `ConfiguracoesPdv.razor`
- `TerminaisPdv.razor`
- `Registradoras.razor`
- `MesasConfiguracao.razor` (Unifica parâmetros e mapa)
- `ComandasConfiguracao.razor` (Unifica parâmetros e faixa)
- `PreVendasConfiguracao.razor`
- `OrcamentosConfiguracao.razor`
- `RegrasVenda.razor`
- `SeriesNumeracao.razor`
- `Impressoras.razor`
- `SetoresProducao.razor`
- `Balancas.razor`
- `EtiquetasBalanca.razor`
- `Perifericos.razor`
- `SenhasChamadas.razor`

---

## 🛡️ Segurança, Auditoria e Eventos de Publicação
- **Token de Acesso**: Mantém o padrão estabelecido na Fase 3 de **Token Opaco UUID** para validação das sessões da empresa e usuário.
- **Auditoria Integrada**: Toda alteração de parâmetro operacional ou cadastro de periférico dispara automaticamente inserção de log na tabela `auditoria` com dados do usuário, hora, IP do terminal e dados alterados.
- **Eventos de Publicação**: Preparados e disparados para `eventos_publicacao`, possibilitando sincronização nativa futura para bases locais do PDV offline.

---

## ⚠️ Limitações Conhecidas e Escopo Não Abrangido
Ficam explicitamente fora da Fase 5 as funcionalidades operacionais em tempo real:
- **Sem Vendas Ativas**: Não há registro de cupons fiscais ou fechamentos.
- **Sem Controle de Caixa Operacional**: Sem abertura, suprimento, sangria ou fechamento de turno do caixa.
- **Sem Impressão Física Real Obrigatória**: Endpoints de `/testar` simulam o comando de teste, preparando o layout de bobina, mas não controlam filas do sistema operacional (APS real).
- **Sem Leitura de Balança Física Real**: Endpoint `/ler-peso` retorna um valor simulado com base no protocolo configurado, sem interagir com drivers locais COM na retaguarda web.
- **Sem TTS Operacional Ativo**: O parâmetro de voz sintetizada nas senhas é apenas uma configuração de dados. Não foi criado display de TV ou chamador de voz sintetizada funcional nesta fase.
- **Sem Movimentações Financeiras ou Estoque**: Sem débito de estoque ou geração de contas a pagar/receber.
- **Sem Sincronização Completa**: O banco de dados SQLite local das estações e o sincronizador de rede não fazem parte desta entrega.
