# Registro de Decisões de Projeto (ADR) — Fase 5

Este documento compila as decisões de arquitetura e padrões técnicos adotados durante o desenvolvimento da Fase 5.

---

## 🛡️ ADR 01: Segurança via Token Opaco UUID
- **Contexto**: A API necessita validar sessões de usuário e chaves de segurança da empresa para cada transação e alteração de parâmetros.
- **Decisão**: Rejeitado o uso de JWT (Json Web Tokens) para manter o alinhamento estrito com o padrão estabelecido na Fase 3. As requisições locais enviam o cabeçalho `Authorization: Bearer <token_uuid>`, validado diretamente na tabela `sessoes_usuarios` com hash SHA-256 no banco de dados local.
- **Consequência**: Garantia de revogabilidade imediata de chaves e sessões e menor sobrecarga computacional em hardware modesto local, mantendo a arquitetura offline simples e robusta.

---

## 🔌 ADR 02: Padronização Rígida de Rotas Operacionais
- **Contexto**: Diversos endpoints operacionais e cadastros de hardware foram propostos sob diferentes nomenclaturas em fases anteriores.
- **Decisão**: Padronizar rigidamente o prefixo `/configuracoes/operacionais` para todos os 17 endpoints operacionais. Foi banido completamente o uso do termo `/configuracoes/operacoes/`.
- **Consequência**: Uniformidade no roteamento Axum, facilidade de auditoria centralizada nas rotas locais de rede e consistência absoluta no consumo de APIs na retaguarda Blazor.

---

## ⚡ ADR 03: Separação de Parâmetros e Funcionamento Operacional Real
- **Contexto**: A Fase 5 foca em configurações e preparação física do ecossistema. Funcionalidades como transações financeiras, fechamentos, escuta real de balanças ou chamadores ativos de senhas eletrônicas exigiriam bibliotecas nativas de sistema operacional (Tauri/APS) que não pertencem ao escopo da retaguarda web.
- **Decisão**: Todos os endpoints de testes físicos (`/impressoras/{id}/testar`, `/perifericos/{id}/testar`, `/senhas-chamadas/{id}/testar` e `/balancas/{id}/ler-peso`) funcionam de forma simulada/mockada em ambiente web. O banco de dados armazena os parâmetros reais que serão consumidos futuramente pelo executável do PDV offline nativo na Fase 6.
- **Consequência**: Agilidade na homologação da retaguarda administrativa WebAssembly, isolando os drivers de hardware para o escopo nativo apropriado.

---

# Registro de Decisões de Projeto (ADR) — Fase 6

Decisões de arquitetura adotadas na Fase 6 — Sincronização Base e Publicação para Terminais.

---

## 🔄 ADR 04: Reaproveitamento de sync_idempotencia (PostgreSQL)
- **Contexto**: A migration `009_sync_base.sql` precisaria criar controle de idempotência para operações de publicação e confirmação de pacotes.
- **Decisão**: A tabela `sync_idempotencia` **já existia** na migration `001_schema_inicial.sql` com os campos `idempotency_key (PK)`, `event_type`, `processado_em` e `resultado`. **Não foi recriada nem alterada** — campos existentes são suficientes para o escopo da Fase 6.
- **Consequência**: Zero risco de perda de dados de idempotência registrados em fases anteriores. Reutilização direta pelos novos endpoints de sync.

---

## 🔄 ADR 05: Reaproveitamento de eventos_publicacao (PostgreSQL)
- **Contexto**: A Fase 6 requer eventos de publicação como `TERMINAL_REGISTRADO`, `PUBLICACAO_CRIADA`, etc.
- **Decisão**: A tabela `eventos_publicacao` **já existia** na migration `006_cadastros_pessoas.sql` com estrutura genérica (`tipo_evento`, `entidade`, `entidade_id`, `payload`, `processado`). **Não foi recriada**. Os novos tipos de evento da Fase 6 serão inseridos via INSERT durante a operação normal da API.
- **Consequência**: Histórico completo de eventos preservado. Tabela genérica cobre todos os novos tipos sem alteração estrutural.

---

## 🔧 ADR 06: ALTER TABLE terminais_pdv (PostgreSQL)
- **Contexto**: A tabela `terminais_pdv` existia desde a migration `008_configuracoes_operacionais.sql` mas sem os campos necessários para controle de sincronização da Fase 6.
- **Decisão**: Aplicado `ALTER TABLE` idempotente usando bloco `DO $$ ... IF NOT EXISTS ... $$` para adicionar **5 colunas novas**: `chave_terminal`, `status_sync`, `ultima_versao_recebida`, `ultima_sincronizacao`, `primeiro_sync_concluido`. Nenhuma coluna existente foi alterada ou removida.
- **Consequência**: Registros existentes preservados com valores padrão nas novas colunas (`status_sync = 'PENDENTE'`, `primeiro_sync_concluido = FALSE`). Migration é segura para re-execução.

---

## 📦 ADR 07: Migration SQLite como versão 002 (em vez de 001_schema_local)
- **Contexto**: O prompt sugeria criar `001_schema_local.sql` no SQLite, mas já existia `001_schema_inicial.sql` com `sync_inbox`, `sync_outbox`, `sync_logs`, `configuracoes_locais` e `terminais`.
- **Decisão**: Criada `002_sync_fase6.sql` como **segunda migration** no sistema versionado existente. As tabelas já presentes na migration 001 **não foram duplicadas**. O arquivo `crates/aureon-infra/src/sqlite/migrations.rs` foi atualizado para registrar a versão 2.
- **Consequência**: Sistema de migrations preserva o histórico. O PDV nunca re-executa migrations já aplicadas (verificação por `schema_migrations_local`). Rollback seguro se a migration 002 falhar na inicialização.

---

## 🔄 ADR 08: Reaproveitamento de sync_outbox, sync_inbox e sync_logs (SQLite)
- **Contexto**: A migration SQLite 002 precisaria dessas tabelas de controle de fila e log.
- **Decisão**: `sync_outbox`, `sync_inbox` e `sync_logs` **já existiam** na migration `001_schema_inicial.sql` com estrutura compatível. **Não foram recriadas** na migration 002.
- **Consequência**: Dados de fila e log existentes no SQLite preservados. A migration 002 apenas adiciona tabelas novas sem tocar nas existentes.

---

## 🔒 ADR 09: Armazenamento seguro da chave_terminal no SQLite
- **Contexto**: O terminal PDV precisa armazenar sua `chave_terminal` (token opaco UUID) localmente para autenticar chamadas subsequentes à API.
- **Decisão**: Em produção, o valor sensível é gravado na tabela `configuracoes_locais` (campo `valor_criptografado`). A coluna `chave_terminal` em `terminal_local` serve apenas como referência de status — nunca é exposta em `sync_logs` ou `logs_locais`.
- **Consequência**: Proteção dupla: dado sensível criptografado + log sem exposição de segredos. Segue o padrão oficial da Fase 3 de não logar tokens.

---

## 📦 ADR 10: Integração Real PostgreSQL para Pacotes de Sincronização
- **Contexto**: A rota de primeira sincronização inicialmente usava payloads mockados para catálogo de produtos, preços, fiscal, periféricos e complementos.
- **Decisão**: Substituímos todos os mocks JSON por consultas dinâmicas reais ao PostgreSQL usando funções SQL agregadoras como `json_agg` e `row_to_json`. As queries cobrem 100% dos 9 grupos de dados requeridos.
- **Consequência**: Sincronização ponta a ponta com dados reais cadastrados na retaguarda, eliminando o isolamento de dados artificiais.

---

## 🖥️ ADR 11: Interface Blazor para Administração de Sync
- **Contexto**: A retaguarda necessita expor os status de sincronização e diagnóstico para controle gerencial dos administradores.
- **Decisão**: Criada uma seção "Sincronização" no menu principal com 4 telas Blazor WebAssembly dedicadas: Status de Terminais, Publicação de Dados, Logs de Sync e Diagnósticos, consumindo os endpoints reais da API Axum.
- **Consequência**: Visualização centralizada e em tempo real do ecossistema de terminais ativos com fluxo operacional limpo e responsivo.

---

# Registro de Decisões de Projeto (ADR) — Fase 7

Decisões de arquitetura adotadas na Fase 7 — PDV Núcleo.

---

## 💰 ADR 12: Eliminação Absoluta de Ponto Flutuante (Matemática Inteira)
- **Contexto**: O sistema precisa garantir exatidão em cálculos financeiros. O uso de `Float` (como em `f64` ou `REAL`) causa dízimas infinitas em cálculos binários, resultando em centavos perdidos no arredondamento durante pagamentos multimoeda.
- **Decisão**: O banco de dados (SQLite), o Backend (Rust) e o Frontend (Blazor) aboliram tipos flutuantes para dinheiro. Adotou-se o formato *Minor Unit* onde `R$ 10,50` vira `1050` (inteiro `i64`). O `Float` é usado no C# apenas no instante de renderizar a máscara visual na interface gráfica. A escala de quantidade usa `escala3` e a taxa de câmbio usa `escala6`.
- **Consequência**: Garantia financeira matemática provada de ponta a ponta sem risco de perda de transação por mismatch de arredondamento.

---

## 🔒 ADR 13: Proteção da Numeração Oficial (Seq. Idempotente)
- **Contexto**: Cancelamentos de venda e abandono de carrinho na frente de caixa queimariam buracos na numeração legal/fiscal de vendas, proibido na maioria das legislações.
- **Decisão**: O número definitivo de venda (`numero_venda`) foi desatrelado da criação da venda. Vendas abertas possuem apenas UUID. A numeração oficial fica blindada e só é requerida em bloco de transação atômica (`conn.transaction`) no exato momento da quitação de pagamento final (`finalizar_venda`).
- **Consequência**: Prevenção total contra lacunas numéricas em relatórios fiscais sem necessidade de reaproveitamento complexo de números cancelados.

---

## 💱 ADR 14: Caixa Multimoeda Nativo
- **Contexto**: A atuação em regiões de fronteira exige troco e pagamento em Reais, Dólar e Guarani no mesmo ticket e fechamento de caixa.
- **Decisão**: A estrutura de caixa (`sessoes_caixa_moedas`) armazena abertura, esperado, informado e diferença para cada moeda independentemente. Pagamentos travam a cotação e realizam rateio exato para o banco.
- **Consequência**: Dispensa integrações contábeis complexas na retaguarda, o PDV já devolve o DRE exato e as sobras de gaveta na respectiva moeda apurada.

---

# Registro de Decisões de Projeto (ADR) — Fase 8

Decisões de arquitetura adotadas na Fase 8 — PDV Operacional.

---

## 🔒 ADR 15: Validação do Supervisor via Cache Local com Hash Bcrypt
- **Contexto**: O sistema necessita autorizar ações críticas (sangria, vale, reimpressão, estornos) de forma segura no PDV local sem conectividade síncrona com a retaguarda PostgreSQL.
- **Decisão**: Banimento de qualquer PIN hardcoded ("1234") nos fontes. A autorização do supervisor é autenticada comparando a senha inserida contra o campo `pin_hash` na tabela `supervisores_cache` local. O hash é validado por meio da biblioteca Bcrypt.
- **Consequência**: Garante alto nível de segurança mesmo em ambiente puramente offline, impedindo vazamentos de PINs por meio de engenharia reversa no binário do PDV ou leitura simples de logs do SQLite.

---

## 👥 ADR 16: Associação de Clientes com Validação no Cache Local
- **Contexto**: Operações de balcão necessitam associar o CPF/CNPJ de clientes ao carrinho de compras e bloquear vendas para cadastros inativos/bloqueados no retaguarda.
- **Decisão**: A associação de cliente (`associar_cliente_venda`) efetua uma consulta na tabela `clientes_cache` local. Caso o cliente selecionado retorne com o status `ativo = 0`, a operação é imediatamente abortada e retorna erro financeiro amigável, impedindo o checkout de clientes devedores ou bloqueados.
- **Consequência**: Operação veloz e alinhada às restrições corporativas sem latência de rede.

---

## ⚡ ADR 17: Persistência Atômica de Eventos e Outbox
- **Contexto**: Toda movimentação local de gaveta (sangria, suprimento, vale), reimpressões ou cancelamentos deve gerar um evento de sincronização que será enviado ao servidor no próximo ciclo de sync.
- **Decisão**: Todas as criações de registros operacionais (ex: tabela `caixa_movimentacoes`) e suas respectivas inserções no `sync_outbox` são envelopadas em uma única transação atômica (`conn.transaction`).
- **Consequência**: Garantia de que a fila de sincronização nunca ficará inconsistente com os dados reais de gaveta locais, mesmo em casos de quedas de energia repentinas do terminal de venda.

---

## 📦 ADR 18: Cache Local via Migration Incremental e Separação de Seeds de Teste
- **Contexto**: Para suportar a validação real de clientes e supervisores sem acoplar a rede síncrona, faz-se necessário expandir o modelo relacional local de dados temporários.
- **Decisão**: Criada a migration `006_pdv_operacional_fase8_cache.sql` para estruturar estritamente as tabelas `clientes_cache` e `supervisores_cache` e seus índices locais (apenas DDL). Todos os dados de semente para homologação e desenvolvimento (como o supervisor default e o PIN `"1234"`) foram isolados em um script SQL externo: `database/seeds/dev/sqlite/seed_fase8_dev.sql`.
- **Consequência**: Garantia de que credenciais e dados de teste jamais serão embarcados automaticamente em ambientes de produção, enquanto a flexibilidade de testes locais é mantida através de comandos de seeding manuais ou manuais controlados.

---

# Registro de Decisões de Projeto (ADR) — Fase 9

Decisões de arquitetura adotadas na Fase 9 — PDV Gourmet.

---

## 🍽️ ADR 19: Fluxo de Fechamento Transicional (Mesa/Comanda)
- **Contexto**: O Gourmet exige que mesas continuem operacionais enquanto o faturamento ocorre.
- **Decisão (Fechamento Transicional)**: A mesa/comanda não é extinta imediatamente no pedido da conta. Ao chamar `fechar_em_venda`, cria-se o espelho de venda com status `EM_ANDAMENTO` sem `numero_venda`. O PDV balcão assume o faturamento. Se pago, a mesa vira `FECHADA`.
- **Decisão (Tabelas de Operação Isoladas)**: Diferenciar `mesas_cache` e `mesas_operacionais`. A primeira é estrutural do restaurante. A segunda nasce e morre no ciclo de vida de uso do cliente. Idem para comandas.
- **Decisão (Bloqueio de Saldo)**: Se houver venda `EM_ANDAMENTO`, a adição de novos itens, transferências ou cancelamentos no Gourmet são explicitamente bloqueados para não corromper o troco em processamento do caixa.
- **Decisão (Inteiros para Escalas)**: Segue-se estritamente a lei global do Aureon: NENHUM float/double no Rust. Minor units para `TotalConsumoMinor` e escala 3 para `QuantidadeEscala3`.
- **Consequência**: Consistência absoluta entre o consumo da mesa e o caixa final, prevenindo race conditions em ambientes multi-usuário.

---

---

## 🛠️ ADR 20: Delivery Operacional e Separação da Taxa de Entrega
- **Contexto**: A Fase 10 introduz o módulo de Delivery, necessitando gerenciar pedidos locais e online, além de lidar com a taxa de entrega.
- **Decisão (Taxa de Entrega Separada)**: A taxa de entrega é armazenada em coluna própria (`taxa_entrega_minor`) tanto no delivery quanto nas vendas. Ela jamais é misturada em `acrescimo_total_minor`.
- **Decisão (Pagamento Delegado)**: O delivery não processa pagamentos. Ele é convertido em uma venda `EM_ANDAMENTO` e o pagamento ocorre no PDV (Fase 7).
- **Decisão (Sem Float/Double)**: Valores monetários seguem como `i64` (minor units) e quantidades como `i64` (escala 3).
- **Consequência**: Relatórios financeiros precisos (frete vs. consumo) e fluxo de caixa centralizado no núcleo de vendas existente.

---

# Registro de Decisões de Projeto (ADR) — Fase 11

Decisões de arquitetura adotadas na Fase 11 — Estoque Operacional.

---

## 📦 ADR 21: Kardex Local Imutável e Baixa Negativa
- **Contexto**: O sistema PDV precisa baixar o estoque ao final de cada venda, mas não pode de forma alguma bloquear a frente de caixa por falta de saldo, e deve manter um rastro contábil seguro offline.
- **Decisão (Imutabilidade)**: A tabela de histórico `estoque_movimentacoes` no SQLite não permite `UPDATE` ou `DELETE`. Correções são tratadas unicamente como novos registros de estorno compensatório (ex: `ESTORNO_VENDA`).
- **Decisão (Saldo Negativo)**: Foi explicitamente aprovado não utilizar restrições do tipo `CHECK(quantidade >= 0)`. O PDV aceita saldos negativos (ex: vende e fica -5). O acerto ocorre via Lançamento de Inventário (`registrar_inventario`).
- **Decisão (Idempotência)**: O backend Rust engole pedidos repetidos e duplos cliques no frontend checando se já há um registro prévio na tabela com a mesma origem para aquela operação (`processar_baixa_venda`).
- **Decisão (Escala e Inteiros)**: Nenhuma operação de estoque usou `double/float`. A API espera `i64` para quantidades (em `escala 3`). As views em Blazor formatam localmente via `decimal / 1000m`.
- **Consequência**: Operação de caixa super-resiliente, livre de impeditivos sistêmicos operacionais e totalmente transparente à malha contábil (Kardex seguro).

---

# Registro de Decisões de Projeto (ADR) — Fase 12

Decisões de arquitetura adotadas na Fase 12 — Compras e Entrada Manual.

---

## 🛒 ADR 22: Compras Manuais, Entrada no Estoque, Estorno e Custo Unitário em Cotação Snapshot
- **Contexto**: A entrada manual de mercadorias no PDV local deve registrar a entrada no estoque, alimentar o Kardex, atualizar o último custo e suportar transações em múltiplas moedas com cotação travada.
- **Decisão (Snapshot de Câmbio)**: A cotação da compra é gravada em escala 6 no momento da criação da compra (`taxa_cambio_escala6`). Todos os custos e totais convertidos usam matemática inteira com essa taxa de câmbio snapshot, independente de variações cambiais futuras.
- **Decisão (Entrada e Estorno)**: Ao finalizar uma compra (`FINALIZADA`), as quantidades em escala 3 dos produtos configurados com `controla_estoque = 1` são adicionadas ao `produtos_estoque_cache` e uma movimentação `ENTRADA_COMPRA` é gravada no Kardex de forma atômica. Se a compra for cancelada (`CANCELADA`), gera-se uma nova movimentação do tipo `ESTORNO_ENTRADA_COMPRA` com sinal inverso no Kardex, deduzindo os saldos, sem alterar o histórico anterior.
- **Decisão (Último Custo)**: O último custo em BRL convertida (`ultimo_custo_minor`) do produto é atualizado na finalização da compra usando o custo unitário convertido pela taxa da compra, sem cálculo de preço médio ponderado no PDV.
- **Consequência**: Consistência absoluta do estoque local, com histórico completo de auditoria no Kardex, rastreabilidade de custos em moedas estrangeiras, e garantia de imutabilidade de compras fechadas/canceladas.

---

# Registro de Decisões de Projeto (ADR) — Fase 13

Decisões de arquitetura adotadas na Fase 13 — Financeiro Base.

---

## 🪙 ADR 23: Contas a Pagar, Contas a Receber, Livro-Caixa Imutável e Regras de Sessão Ativa
- **Contexto**: A introdução de contas a pagar, contas a receber (crediário) e lançamentos de livro-caixa no PDV offline exige segurança nas baixas e conformidade contábil.
- **Decisão (Imutabilidade do Livro-Caixa)**: Os registros da tabela `financeiro_lancamentos` são de inserção única (`INSERT ONLY`). Operações de alteração (`UPDATE`) ou exclusão (`DELETE`) são explicitamente proibidas no código-fonte e bloqueadas pela integridade referencial.
- **Decisão (Isolamento do Crediário)**: Vendas finalizadas com a forma de pagamento `CREDITO_CLIENTE` geram um título a receber, mas seus saldos **não entram** no saldo físico do caixa ativo no ato da venda. O valor só entra no saldo real da sessão de caixa no exato momento da quitação parcial ou total via recebimento do crediário (`baixar_conta_receber`).
- **Decisão (Sessão de Caixa Aberta para Baixas)**: É obrigatório que haja uma sessão de caixa aberta (`status = 'ABERTO'`) para a registradora em que a baixa de contas a pagar ou a receber é executada. O backend Rust valida isso a nível de banco de dados na transação atômica.
- **Decisão (Multimoeda com Cotação Fixa)**: O valor principal em BRL é calculado no ato de lançamentos e baixas usando a taxa de câmbio da operação em escala 6, prevenindo distorções matemáticas com o uso estrito de inteiros (`i64/long`).
- **Consequência**: Consistência absoluta do saldo físico de caixa no momento do fechamento, histórico imutável para auditorias fiscais e suporte offline robusto para recebimento de parcelas e pagamentos de despesas.

---

# Registro de Decisões de Projeto (ADR) — Fase 14

Decisões de arquitetura adotadas na Fase 14 — Relatórios Operacionais, Dashboard Local e Exportação.

---

## 📊 ADR 24: Relatórios como Módulo Estritamente Somente Leitura

- **Contexto**: A introdução de um módulo de relatórios e dashboard local exige que nenhuma query de consulta altere dados operacionais, especialmente em um ambiente offline-first com SQLite local.
- **Decisão (Somente SELECT)**: Todos os commands Tauri de relatórios (`commands_relatorios.rs`) utilizam exclusivamente instruções `SELECT`. É proibido executar `INSERT`, `UPDATE` ou `DELETE` em qualquer tabela operacional a partir do módulo de relatórios.
- **Decisão (Filtro Padrão de 30 dias)**: O período padrão de todos os relatórios e do dashboard é sempre os **últimos 30 dias**, calculado dinamicamente no cliente Blazor. Evita varredura completa das tabelas e protege a performance em dispositivos com hardware limitado.
- **Decisão (Multimoeda Segregada)**: Totais de relatórios são sempre exibidos separados por moeda. Nunca são somadas moedas diferentes em um único valor. Conversões para BRL são exibidas como campos auxiliares de comparação, não como soma principal.
- **Decisão (Exportação Local)**: O arquivo CSV é gerado inteiramente no processo Blazor/C# e entregue ao sistema operacional via a função JavaScript `aureon.downloadFile`, usando a API de Blob do navegador. Nenhum dado é enviado a servidores externos.
- **Decisão (Impressão Nativa)**: A funcionalidade de impressão/PDF usa `window.print()` com CSS `@media print` para separar o layout interativo do layout de impressão limpo. Nenhuma biblioteca de PDF de terceiros foi adicionada.
- **Consequência**: O módulo de relatórios é seguro para uso em produção sem risco de corrupção de dados operacionais, com performance protegida por filtros de período e total compatibilidade offline-first.



