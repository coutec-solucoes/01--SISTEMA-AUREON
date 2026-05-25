# Registro de DecisÃµes de Projeto (ADR) â€” Fase 5

Este documento compila as decisÃµes de arquitetura e padrÃµes tÃ©cnicos adotados durante o desenvolvimento da Fase 5.

---

## ðŸ›¡ï¸� ADR 01: SeguranÃ§a via Token Opaco UUID
- **Contexto**: A API necessita validar sessÃµes de usuÃ¡rio e chaves de seguranÃ§a da empresa para cada transaÃ§Ã£o e alteraÃ§Ã£o de parÃ¢metros.
- **DecisÃ£o**: Rejeitado o uso de JWT (Json Web Tokens) para manter o alinhamento estrito com o padrÃ£o estabelecido na Fase 3. As requisiÃ§Ãµes locais enviam o cabeÃ§alho `Authorization: Bearer <token_uuid>`, validado diretamente na tabela `sessoes_usuarios` com hash SHA-256 no banco de dados local.
- **ConsequÃªncia**: Garantia de revogabilidade imediata de chaves e sessÃµes e menor sobrecarga computacional em hardware modesto local, mantendo a arquitetura offline simples e robusta.

---

## ðŸ”Œ ADR 02: PadronizaÃ§Ã£o RÃ­gida de Rotas Operacionais
- **Contexto**: Diversos endpoints operacionais e cadastros de hardware foram propostos sob diferentes nomenclaturas em fases anteriores.
- **DecisÃ£o**: Padronizar rigidamente o prefixo `/configuracoes/operacionais` para todos os 17 endpoints operacionais. Foi banido completamente o uso do termo `/configuracoes/operacoes/`.
- **ConsequÃªncia**: Uniformidade no roteamento Axum, facilidade de auditoria centralizada nas rotas locais de rede e consistÃªncia absoluta no consumo de APIs na retaguarda Blazor.

---

## âš¡ ADR 03: SeparaÃ§Ã£o de ParÃ¢metros e Funcionamento Operacional Real
- **Contexto**: A Fase 5 foca em configuraÃ§Ãµes e preparaÃ§Ã£o fÃ­sica do ecossistema. Funcionalidades como transaÃ§Ãµes financeiras, fechamentos, escuta real de balanÃ§as ou chamadores ativos de senhas eletrÃ´nicas exigiriam bibliotecas nativas de sistema operacional (Tauri/APS) que nÃ£o pertencem ao escopo da retaguarda web.
- **DecisÃ£o**: Todos os endpoints de testes fÃ­sicos (`/impressoras/{id}/testar`, `/perifericos/{id}/testar`, `/senhas-chamadas/{id}/testar` e `/balancas/{id}/ler-peso`) funcionam de forma simulada/mockada em ambiente web. O banco de dados armazena os parÃ¢metros reais que serÃ£o consumidos futuramente pelo executÃ¡vel do PDV offline nativo na Fase 6.
- **ConsequÃªncia**: Agilidade na homologaÃ§Ã£o da retaguarda administrativa WebAssembly, isolando os drivers de hardware para o escopo nativo apropriado.

---

# Registro de DecisÃµes de Projeto (ADR) â€” Fase 6

DecisÃµes de arquitetura adotadas na Fase 6 â€” SincronizaÃ§Ã£o Base e PublicaÃ§Ã£o para Terminais.

---

## ðŸ”„ ADR 04: Reaproveitamento de sync_idempotencia (PostgreSQL)
- **Contexto**: A migration `009_sync_base.sql` precisaria criar controle de idempotÃªncia para operaÃ§Ãµes de publicaÃ§Ã£o e confirmaÃ§Ã£o de pacotes.
- **DecisÃ£o**: A tabela `sync_idempotencia` **jÃ¡ existia** na migration `001_schema_inicial.sql` com os campos `idempotency_key (PK)`, `event_type`, `processado_em` e `resultado`. **NÃ£o foi recriada nem alterada** â€” campos existentes sÃ£o suficientes para o escopo da Fase 6.
- **ConsequÃªncia**: Zero risco de perda de dados de idempotÃªncia registrados em fases anteriores. ReutilizaÃ§Ã£o direta pelos novos endpoints de sync.

---

## ðŸ”„ ADR 05: Reaproveitamento de eventos_publicacao (PostgreSQL)
- **Contexto**: A Fase 6 requer eventos de publicaÃ§Ã£o como `TERMINAL_REGISTRADO`, `PUBLICACAO_CRIADA`, etc.
- **DecisÃ£o**: A tabela `eventos_publicacao` **jÃ¡ existia** na migration `006_cadastros_pessoas.sql` com estrutura genÃ©rica (`tipo_evento`, `entidade`, `entidade_id`, `payload`, `processado`). **NÃ£o foi recriada**. Os novos tipos de evento da Fase 6 serÃ£o inseridos via INSERT durante a operaÃ§Ã£o normal da API.
- **ConsequÃªncia**: HistÃ³rico completo de eventos preservado. Tabela genÃ©rica cobre todos os novos tipos sem alteraÃ§Ã£o estrutural.

---

## ðŸ”§ ADR 06: ALTER TABLE terminais_pdv (PostgreSQL)
- **Contexto**: A tabela `terminais_pdv` existia desde a migration `008_configuracoes_operacionais.sql` mas sem os campos necessÃ¡rios para controle de sincronizaÃ§Ã£o da Fase 6.
- **DecisÃ£o**: Aplicado `ALTER TABLE` idempotente usando bloco `DO $$ ... IF NOT EXISTS ... $$` para adicionar **5 colunas novas**: `chave_terminal`, `status_sync`, `ultima_versao_recebida`, `ultima_sincronizacao`, `primeiro_sync_concluido`. Nenhuma coluna existente foi alterada ou removida.
- **ConsequÃªncia**: Registros existentes preservados com valores padrÃ£o nas novas colunas (`status_sync = 'PENDENTE'`, `primeiro_sync_concluido = FALSE`). Migration Ã© segura para re-execuÃ§Ã£o.

---

## ðŸ“¦ ADR 07: Migration SQLite como versÃ£o 002 (em vez de 001_schema_local)
- **Contexto**: O prompt sugeria criar `001_schema_local.sql` no SQLite, mas jÃ¡ existia `001_schema_inicial.sql` com `sync_inbox`, `sync_outbox`, `sync_logs`, `configuracoes_locais` e `terminais`.
- **DecisÃ£o**: Criada `002_sync_fase6.sql` como **segunda migration** no sistema versionado existente. As tabelas jÃ¡ presentes na migration 001 **nÃ£o foram duplicadas**. O arquivo `crates/aureon-infra/src/sqlite/migrations.rs` foi atualizado para registrar a versÃ£o 2.
- **ConsequÃªncia**: Sistema de migrations preserva o histÃ³rico. O PDV nunca re-executa migrations jÃ¡ aplicadas (verificaÃ§Ã£o por `schema_migrations_local`). Rollback seguro se a migration 002 falhar na inicializaÃ§Ã£o.

---

## ðŸ”„ ADR 08: Reaproveitamento de sync_outbox, sync_inbox e sync_logs (SQLite)
- **Contexto**: A migration SQLite 002 precisaria dessas tabelas de controle de fila e log.
- **DecisÃ£o**: `sync_outbox`, `sync_inbox` e `sync_logs` **jÃ¡ existiam** na migration `001_schema_inicial.sql` com estrutura compatÃ­vel. **NÃ£o foram recriadas** na migration 002.
- **ConsequÃªncia**: Dados de fila e log existentes no SQLite preservados. A migration 002 apenas adiciona tabelas novas sem tocar nas existentes.

---

## ðŸ”’ ADR 09: Armazenamento seguro da chave_terminal no SQLite
- **Contexto**: O terminal PDV precisa armazenar sua `chave_terminal` (token opaco UUID) localmente para autenticar chamadas subsequentes Ã  API.
- **DecisÃ£o**: Em produÃ§Ã£o, o valor sensÃ­vel Ã© gravado na tabela `configuracoes_locais` (campo `valor_criptografado`). A coluna `chave_terminal` em `terminal_local` serve apenas como referÃªncia de status â€” nunca Ã© exposta em `sync_logs` ou `logs_locais`.
- **ConsequÃªncia**: ProteÃ§Ã£o dupla: dado sensÃ­vel criptografado + log sem exposiÃ§Ã£o de segredos. Segue o padrÃ£o oficial da Fase 3 de nÃ£o logar tokens.

---

## ðŸ“¦ ADR 10: IntegraÃ§Ã£o Real PostgreSQL para Pacotes de SincronizaÃ§Ã£o
- **Contexto**: A rota de primeira sincronizaÃ§Ã£o inicialmente usava payloads mockados para catÃ¡logo de produtos, preÃ§os, fiscal, perifÃ©ricos e complementos.
- **DecisÃ£o**: SubstituÃ­mos todos os mocks JSON por consultas dinÃ¢micas reais ao PostgreSQL usando funÃ§Ãµes SQL agregadoras como `json_agg` e `row_to_json`. As queries cobrem 100% dos 9 grupos de dados requeridos.
- **ConsequÃªncia**: SincronizaÃ§Ã£o ponta a ponta com dados reais cadastrados na retaguarda, eliminando o isolamento de dados artificiais.

---

## ðŸ–¥ï¸� ADR 11: Interface Blazor para AdministraÃ§Ã£o de Sync
- **Contexto**: A retaguarda necessita expor os status de sincronizaÃ§Ã£o e diagnÃ³stico para controle gerencial dos administradores.
- **DecisÃ£o**: Criada uma seÃ§Ã£o "SincronizaÃ§Ã£o" no menu principal com 4 telas Blazor WebAssembly dedicadas: Status de Terminais, PublicaÃ§Ã£o de Dados, Logs de Sync e DiagnÃ³sticos, consumindo os endpoints reais da API Axum.
- **ConsequÃªncia**: VisualizaÃ§Ã£o centralizada e em tempo real do ecossistema de terminais ativos com fluxo operacional limpo e responsivo.

---

# Registro de DecisÃµes de Projeto (ADR) â€” Fase 7

DecisÃµes de arquitetura adotadas na Fase 7 â€” PDV NÃºcleo.

---

## ðŸ’° ADR 12: EliminaÃ§Ã£o Absoluta de Ponto Flutuante (MatemÃ¡tica Inteira)
- **Contexto**: O sistema precisa garantir exatidÃ£o em cÃ¡lculos financeiros. O uso de `Float` (como em `f64` ou `REAL`) causa dÃ­zimas infinitas em cÃ¡lculos binÃ¡rios, resultando em centavos perdidos no arredondamento durante pagamentos multimoeda.
- **DecisÃ£o**: O banco de dados (SQLite), o Backend (Rust) e o Frontend (Blazor) aboliram tipos flutuantes para dinheiro. Adotou-se o formato *Minor Unit* onde `R$ 10,50` vira `1050` (inteiro `i64`). O `Float` Ã© usado no C# apenas no instante de renderizar a mÃ¡scara visual na interface grÃ¡fica. A escala de quantidade usa `escala3` e a taxa de cÃ¢mbio usa `escala6`.
- **ConsequÃªncia**: Garantia financeira matemÃ¡tica provada de ponta a ponta sem risco de perda de transaÃ§Ã£o por mismatch de arredondamento.

---

## ðŸ”’ ADR 13: ProteÃ§Ã£o da NumeraÃ§Ã£o Oficial (Seq. Idempotente)
- **Contexto**: Cancelamentos de venda e abandono de carrinho na frente de caixa queimariam buracos na numeraÃ§Ã£o legal/fiscal de vendas, proibido na maioria das legislaÃ§Ãµes.
- **DecisÃ£o**: O nÃºmero definitivo de venda (`numero_venda`) foi desatrelado da criaÃ§Ã£o da venda. Vendas abertas possuem apenas UUID. A numeraÃ§Ã£o oficial fica blindada e sÃ³ Ã© requerida em bloco de transaÃ§Ã£o atÃ´mica (`conn.transaction`) no exato momento da quitaÃ§Ã£o de pagamento final (`finalizar_venda`).
- **ConsequÃªncia**: PrevenÃ§Ã£o total contra lacunas numÃ©ricas em relatÃ³rios fiscais sem necessidade de reaproveitamento complexo de nÃºmeros cancelados.

---

## ðŸ’± ADR 14: Caixa Multimoeda Nativo
- **Contexto**: A atuaÃ§Ã£o em regiÃµes de fronteira exige troco e pagamento em Reais, DÃ³lar e Guarani no mesmo ticket e fechamento de caixa.
- **DecisÃ£o**: A estrutura de caixa (`sessoes_caixa_moedas`) armazena abertura, esperado, informado e diferenÃ§a para cada moeda independentemente. Pagamentos travam a cotaÃ§Ã£o e realizam rateio exato para o banco.
- **ConsequÃªncia**: Dispensa integraÃ§Ãµes contÃ¡beis complexas na retaguarda, o PDV jÃ¡ devolve o DRE exato e as sobras de gaveta na respectiva moeda apurada.

---

# Registro de DecisÃµes de Projeto (ADR) â€” Fase 8

DecisÃµes de arquitetura adotadas na Fase 8 â€” PDV Operacional.

---

## ðŸ”’ ADR 15: ValidaÃ§Ã£o do Supervisor via Cache Local com Hash Bcrypt
- **Contexto**: O sistema necessita autorizar aÃ§Ãµes crÃ­ticas (sangria, vale, reimpressÃ£o, estornos) de forma segura no PDV local sem conectividade sÃ­ncrona com a retaguarda PostgreSQL.
- **DecisÃ£o**: Banimento de qualquer PIN hardcoded ("1234") nos fontes. A autorizaÃ§Ã£o do supervisor Ã© autenticada comparando a senha inserida contra o campo `pin_hash` na tabela `supervisores_cache` local. O hash Ã© validado por meio da biblioteca Bcrypt.
- **ConsequÃªncia**: Garante alto nÃ­vel de seguranÃ§a mesmo em ambiente puramente offline, impedindo vazamentos de PINs por meio de engenharia reversa no binÃ¡rio do PDV ou leitura simples de logs do SQLite.

---

## ðŸ‘¥ ADR 16: AssociaÃ§Ã£o de Clientes com ValidaÃ§Ã£o no Cache Local
- **Contexto**: OperaÃ§Ãµes de balcÃ£o necessitam associar o CPF/CNPJ de clientes ao carrinho de compras e bloquear vendas para cadastros inativos/bloqueados no retaguarda.
- **DecisÃ£o**: A associaÃ§Ã£o de cliente (`associar_cliente_venda`) efetua uma consulta na tabela `clientes_cache` local. Caso o cliente selecionado retorne com o status `ativo = 0`, a operaÃ§Ã£o Ã© imediatamente abortada e retorna erro financeiro amigÃ¡vel, impedindo o checkout de clientes devedores ou bloqueados.
- **ConsequÃªncia**: OperaÃ§Ã£o veloz e alinhada Ã s restriÃ§Ãµes corporativas sem latÃªncia de rede.

---

## âš¡ ADR 17: PersistÃªncia AtÃ´mica de Eventos e Outbox
- **Contexto**: Toda movimentaÃ§Ã£o local de gaveta (sangria, suprimento, vale), reimpressÃµes ou cancelamentos deve gerar um evento de sincronizaÃ§Ã£o que serÃ¡ enviado ao servidor no prÃ³ximo ciclo de sync.
- **DecisÃ£o**: Todas as criaÃ§Ãµes de registros operacionais (ex: tabela `caixa_movimentacoes`) e suas respectivas inserÃ§Ãµes no `sync_outbox` sÃ£o envelopadas em uma Ãºnica transaÃ§Ã£o atÃ´mica (`conn.transaction`).
- **ConsequÃªncia**: Garantia de que a fila de sincronizaÃ§Ã£o nunca ficarÃ¡ inconsistente com os dados reais de gaveta locais, mesmo em casos de quedas de energia repentinas do terminal de venda.

---

## ðŸ“¦ ADR 18: Cache Local via Migration Incremental e SeparaÃ§Ã£o de Seeds de Teste
- **Contexto**: Para suportar a validaÃ§Ã£o real de clientes e supervisores sem acoplar a rede sÃ­ncrona, faz-se necessÃ¡rio expandir o modelo relacional local de dados temporÃ¡rios.
- **DecisÃ£o**: Criada a migration `006_pdv_operacional_fase8_cache.sql` para estruturar estritamente as tabelas `clientes_cache` e `supervisores_cache` e seus Ã­ndices locais (apenas DDL). Todos os dados de semente para homologaÃ§Ã£o e desenvolvimento (como o supervisor default e o PIN `"1234"`) foram isolados em um script SQL externo: `database/seeds/dev/sqlite/seed_fase8_dev.sql`.
- **ConsequÃªncia**: Garantia de que credenciais e dados de teste jamais serÃ£o embarcados automaticamente em ambientes de produÃ§Ã£o, enquanto a flexibilidade de testes locais Ã© mantida atravÃ©s de comandos de seeding manuais ou manuais controlados.

---

# Registro de DecisÃµes de Projeto (ADR) â€” Fase 9

DecisÃµes de arquitetura adotadas na Fase 9 â€” PDV Gourmet.

---

## ðŸ�½ï¸� ADR 19: Fluxo de Fechamento Transicional (Mesa/Comanda)
- **Contexto**: O Gourmet exige que mesas continuem operacionais enquanto o faturamento ocorre.
- **DecisÃ£o (Fechamento Transicional)**: A mesa/comanda nÃ£o Ã© extinta imediatamente no pedido da conta. Ao chamar `fechar_em_venda`, cria-se o espelho de venda com status `EM_ANDAMENTO` sem `numero_venda`. O PDV balcÃ£o assume o faturamento. Se pago, a mesa vira `FECHADA`.
- **DecisÃ£o (Tabelas de OperaÃ§Ã£o Isoladas)**: Diferenciar `mesas_cache` e `mesas_operacionais`. A primeira Ã© estrutural do restaurante. A segunda nasce e morre no ciclo de vida de uso do cliente. Idem para comandas.
- **DecisÃ£o (Bloqueio de Saldo)**: Se houver venda `EM_ANDAMENTO`, a adiÃ§Ã£o de novos itens, transferÃªncias ou cancelamentos no Gourmet sÃ£o explicitamente bloqueados para nÃ£o corromper o troco em processamento do caixa.
- **DecisÃ£o (Inteiros para Escalas)**: Segue-se estritamente a lei global do Aureon: NENHUM float/double no Rust. Minor units para `TotalConsumoMinor` e escala 3 para `QuantidadeEscala3`.
- **ConsequÃªncia**: ConsistÃªncia absoluta entre o consumo da mesa e o caixa final, prevenindo race conditions em ambientes multi-usuÃ¡rio.

---

---

## ðŸ› ï¸� ADR 20: Delivery Operacional e SeparaÃ§Ã£o da Taxa de Entrega
- **Contexto**: A Fase 10 introduz o mÃ³dulo de Delivery, necessitando gerenciar pedidos locais e online, alÃ©m de lidar com a taxa de entrega.
- **DecisÃ£o (Taxa de Entrega Separada)**: A taxa de entrega Ã© armazenada em coluna prÃ³pria (`taxa_entrega_minor`) tanto no delivery quanto nas vendas. Ela jamais Ã© misturada em `acrescimo_total_minor`.
- **DecisÃ£o (Pagamento Delegado)**: O delivery nÃ£o processa pagamentos. Ele Ã© convertido em uma venda `EM_ANDAMENTO` e o pagamento ocorre no PDV (Fase 7).
- **DecisÃ£o (Sem Float/Double)**: Valores monetÃ¡rios seguem como `i64` (minor units) e quantidades como `i64` (escala 3).
- **ConsequÃªncia**: RelatÃ³rios financeiros precisos (frete vs. consumo) e fluxo de caixa centralizado no nÃºcleo de vendas existente.

---

# Registro de DecisÃµes de Projeto (ADR) â€” Fase 11

DecisÃµes de arquitetura adotadas na Fase 11 â€” Estoque Operacional.

---

## ðŸ“¦ ADR 21: Kardex Local ImutÃ¡vel e Baixa Negativa
- **Contexto**: O sistema PDV precisa baixar o estoque ao final de cada venda, mas nÃ£o pode de forma alguma bloquear a frente de caixa por falta de saldo, e deve manter um rastro contÃ¡bil seguro offline.
- **DecisÃ£o (Imutabilidade)**: A tabela de histÃ³rico `estoque_movimentacoes` no SQLite nÃ£o permite `UPDATE` ou `DELETE`. CorreÃ§Ãµes sÃ£o tratadas unicamente como novos registros de estorno compensatÃ³rio (ex: `ESTORNO_VENDA`).
- **DecisÃ£o (Saldo Negativo)**: Foi explicitamente aprovado nÃ£o utilizar restriÃ§Ãµes do tipo `CHECK(quantidade >= 0)`. O PDV aceita saldos negativos (ex: vende e fica -5). O acerto ocorre via LanÃ§amento de InventÃ¡rio (`registrar_inventario`).
- **DecisÃ£o (IdempotÃªncia)**: O backend Rust engole pedidos repetidos e duplos cliques no frontend checando se jÃ¡ hÃ¡ um registro prÃ©vio na tabela com a mesma origem para aquela operaÃ§Ã£o (`processar_baixa_venda`).
- **DecisÃ£o (Escala e Inteiros)**: Nenhuma operaÃ§Ã£o de estoque usou `double/float`. A API espera `i64` para quantidades (em `escala 3`). As views em Blazor formatam localmente via `decimal / 1000m`.
- **ConsequÃªncia**: OperaÃ§Ã£o de caixa super-resiliente, livre de impeditivos sistÃªmicos operacionais e totalmente transparente Ã  malha contÃ¡bil (Kardex seguro).

---

# Registro de DecisÃµes de Projeto (ADR) â€” Fase 12

DecisÃµes de arquitetura adotadas na Fase 12 â€” Compras e Entrada Manual.

---

## ðŸ›’ ADR 22: Compras Manuais, Entrada no Estoque, Estorno e Custo UnitÃ¡rio em CotaÃ§Ã£o Snapshot
- **Contexto**: A entrada manual de mercadorias no PDV local deve registrar a entrada no estoque, alimentar o Kardex, atualizar o Ãºltimo custo e suportar transaÃ§Ãµes em mÃºltiplas moedas com cotaÃ§Ã£o travada.
- **DecisÃ£o (Snapshot de CÃ¢mbio)**: A cotaÃ§Ã£o da compra Ã© gravada em escala 6 no momento da criaÃ§Ã£o da compra (`taxa_cambio_escala6`). Todos os custos e totais convertidos usam matemÃ¡tica inteira com essa taxa de cÃ¢mbio snapshot, independente de variaÃ§Ãµes cambiais futuras.
- **DecisÃ£o (Entrada e Estorno)**: Ao finalizar uma compra (`FINALIZADA`), as quantidades em escala 3 dos produtos configurados com `controla_estoque = 1` sÃ£o adicionadas ao `produtos_estoque_cache` e uma movimentaÃ§Ã£o `ENTRADA_COMPRA` Ã© gravada no Kardex de forma atÃ´mica. Se a compra for cancelada (`CANCELADA`), gera-se uma nova movimentaÃ§Ã£o do tipo `ESTORNO_ENTRADA_COMPRA` com sinal inverso no Kardex, deduzindo os saldos, sem alterar o histÃ³rico anterior.
- **DecisÃ£o (Ãšltimo Custo)**: O Ãºltimo custo em BRL convertida (`ultimo_custo_minor`) do produto Ã© atualizado na finalizaÃ§Ã£o da compra usando o custo unitÃ¡rio convertido pela taxa da compra, sem cÃ¡lculo de preÃ§o mÃ©dio ponderado no PDV.
- **ConsequÃªncia**: ConsistÃªncia absoluta do estoque local, com histÃ³rico completo de auditoria no Kardex, rastreabilidade de custos em moedas estrangeiras, e garantia de imutabilidade de compras fechadas/canceladas.

---

# Registro de DecisÃµes de Projeto (ADR) â€” Fase 13

DecisÃµes de arquitetura adotadas na Fase 13 â€” Financeiro Base.

---

## ðŸª™ ADR 23: Contas a Pagar, Contas a Receber, Livro-Caixa ImutÃ¡vel e Regras de SessÃ£o Ativa
- **Contexto**: A introduÃ§Ã£o de contas a pagar, contas a receber (crediÃ¡rio) e lanÃ§amentos de livro-caixa no PDV offline exige seguranÃ§a nas baixas e conformidade contÃ¡bil.
- **DecisÃ£o (Imutabilidade do Livro-Caixa)**: Os registros da tabela `financeiro_lancamentos` sÃ£o de inserÃ§Ã£o Ãºnica (`INSERT ONLY`). OperaÃ§Ãµes de alteraÃ§Ã£o (`UPDATE`) ou exclusÃ£o (`DELETE`) sÃ£o explicitamente proibidas no cÃ³digo-fonte e bloqueadas pela integridade referencial.
- **DecisÃ£o (Isolamento do CrediÃ¡rio)**: Vendas finalizadas com a forma de pagamento `CREDITO_CLIENTE` geram um tÃ­tulo a receber, mas seus saldos **nÃ£o entram** no saldo fÃ­sico do caixa ativo no ato da venda. O valor sÃ³ entra no saldo real da sessÃ£o de caixa no exato momento da quitaÃ§Ã£o parcial ou total via recebimento do crediÃ¡rio (`baixar_conta_receber`).
- **DecisÃ£o (SessÃ£o de Caixa Aberta para Baixas)**: Ã‰ obrigatÃ³rio que haja uma sessÃ£o de caixa aberta (`status = 'ABERTO'`) para a registradora em que a baixa de contas a pagar ou a receber Ã© executada. O backend Rust valida isso a nÃ­vel de banco de dados na transaÃ§Ã£o atÃ´mica.
- **DecisÃ£o (Multimoeda com CotaÃ§Ã£o Fixa)**: O valor principal em BRL Ã© calculado no ato de lanÃ§amentos e baixas usando a taxa de cÃ¢mbio da operaÃ§Ã£o em escala 6, prevenindo distorÃ§Ãµes matemÃ¡ticas com o uso estrito de inteiros (`i64/long`).
- **ConsequÃªncia**: ConsistÃªncia absoluta do saldo fÃ­sico de caixa no momento do fechamento, histÃ³rico imutÃ¡vel para auditorias fiscais e suporte offline robusto para recebimento de parcelas e pagamentos de despesas.

---

# Registro de DecisÃµes de Projeto (ADR) â€” Fase 14

DecisÃµes de arquitetura adotadas na Fase 14 â€” RelatÃ³rios Operacionais, Dashboard Local e ExportaÃ§Ã£o.

---

## ðŸ“Š ADR 24: RelatÃ³rios como MÃ³dulo Estritamente Somente Leitura

- **Contexto**: A introduÃ§Ã£o de um mÃ³dulo de relatÃ³rios e dashboard local exige que nenhuma query de consulta altere dados operacionais, especialmente em um ambiente offline-first com SQLite local.
- **DecisÃ£o (Somente SELECT)**: Todos os commands Tauri de relatÃ³rios (`commands_relatorios.rs`) utilizam exclusivamente instruÃ§Ãµes `SELECT`. Ã‰ proibido executar `INSERT`, `UPDATE` ou `DELETE` em qualquer tabela operacional a partir do mÃ³dulo de relatÃ³rios.
- **DecisÃ£o (Filtro PadrÃ£o de 30 dias)**: O perÃ­odo padrÃ£o de todos os relatÃ³rios e do dashboard Ã© sempre os **Ãºltimos 30 dias**, calculado dinamicamente no cliente Blazor. Evita varredura completa das tabelas e protege a performance em dispositivos com hardware limitado.
- **DecisÃ£o (Multimoeda Segregada)**: Totais de relatÃ³rios sÃ£o sempre exibidos separados por moeda. Nunca sÃ£o somadas moedas diferentes em um Ãºnico valor. ConversÃµes para BRL sÃ£o exibidas como campos auxiliares de comparaÃ§Ã£o, nÃ£o como soma principal.
- **DecisÃ£o (ExportaÃ§Ã£o Local)**: O arquivo CSV Ã© gerado inteiramente no processo Blazor/C# e entregue ao sistema operacional via a funÃ§Ã£o JavaScript `aureon.downloadFile`, usando a API de Blob do navegador. Nenhum dado Ã© enviado a servidores externos.
- **DecisÃ£o (ImpressÃ£o Nativa)**: A funcionalidade de impressÃ£o/PDF usa `window.print()` com CSS `@media print` para separar o layout interativo do layout de impressÃ£o limpo. Nenhuma biblioteca de PDF de terceiros foi adicionada.
- **ConsequÃªncia**: O mÃ³dulo de relatÃ³rios Ã© seguro para uso em produÃ§Ã£o sem risco de corrupÃ§Ã£o de dados operacionais, com performance protegida por filtros de perÃ­odo e total compatibilidade offline-first.

---

# Registro de DecisÃµes de Projeto (ADR) â€” Fase 15

DecisÃµes de arquitetura adotadas na Fase 15 â€” ImpressÃ£o Operacional NÃ£o Fiscal.

---

## ðŸ–¨ï¸� ADR 25: ESC/POS como PadrÃ£o Operacional â€” HTML/PDF apenas como Fallback Administrativo

- **Contexto**: O PDV precisa imprimir cupons, comprovantes, tickets de produÃ§Ã£o e romaneios de delivery em impressoras tÃ©rmicas nÃ£o fiscais. Duas abordagens foram consideradas: (a) ESC/POS nativo via Rust, e (b) HTML/PDF gerado pelo Blazor.
- **DecisÃ£o**: Adotado ESC/POS nativo como padrÃ£o operacional exclusivo para o PDV tÃ©rmico. Um builder prÃ³prio (`EscPosBuilder`) foi implementado em Rust puro, sem dependÃªncias de terceiros. HTML/PDF via `window.print()` fica restrito ao uso administrativo da retaguarda web (ex: relatÃ³rios).
- **Motivo**: Impressoras tÃ©rmicas de PDV (Elgin, Daruma, Epson TM-T20, Bematech) nÃ£o possuem drivers de impressÃ£o web. ESC/POS garante velocidade, corte de papel, pulso de gaveta e compatibilidade direta com todos os modelos comerciais via TCP/IP ou porta serial.
- **ConsequÃªncia**: O mÃ³dulo de impressÃ£o do PDV Ã© totalmente offline-first, sem dependÃªncia de browser, sistema operacional grÃ¡fico ou drivers externos. O builder cobre 100% dos documentos operacionais da Fase 15. Documentos fiscais (NFC-e, NF-e, SAT, SIFEN) ficam explicitamente fora do escopo como mÃ³dulo separado.

---

## ðŸ”’ ADR 26: ImpressÃ£o como SaÃ­da Documental Pura â€” SeparaÃ§Ã£o de Concerns

- **Contexto**: Em sistemas de PDV Ã© comum que a impressÃ£o esteja acoplada Ã  operaÃ§Ã£o (ex: finalizar venda â†’ imprimir automaticamente). Esse acoplamento cria riscos de falha silenciosa quando a impressora estÃ¡ offline.
- **DecisÃ£o**: ImpressÃ£o e operaÃ§Ã£o sÃ£o **mÃ³dulos completamente separados**. Commands de impressÃ£o sÃ£o independentes dos commands operacionais. A UI oferece botÃµes de impressÃ£o avulsos em tela dedicada (`/reimpressoes`). O PDV pode fechar vendas, processar pagamentos e gerir caixa mesmo que a impressora esteja desligada.
- **ExceÃ§Ã£o fÃ­sica permitida**: O pulso de abertura de gaveta (`abrir_gaveta_dinheiro`) Ã© a Ãºnica operaÃ§Ã£o de hardware que nÃ£o Ã© puramente documental, mas tambÃ©m nÃ£o altera dados financeiros â€” apenas dispara o sinal elÃ©trico.
- **ConsequÃªncia**: ResiliÃªncia operacional garantida. Impressoras offline nÃ£o travam o caixa. ReimpressÃµes manuais sÃ£o sempre possÃ­veis via interface. Risco de inconsistÃªncia por falha de impressÃ£o Ã© eliminado da camada transacional.

---

# Registro de DecisÃµes de Projeto (ADR) â€” Fase 16

DecisÃµes de arquitetura adotadas na Fase 16 â€” Fiscal Base e Espelho TÃ©cnico Sem EmissÃ£o.

---

## ðŸ�›ï¸� ADR 27: Espelho TÃ©cnico Isolado sem AlteraÃ§Ã£o Transacional

- **Contexto**: O PDV precisava de estrutura fiscal (NCM, CFOP, CST, IVA) para cÃ¡lculo de impostos como preparaÃ§Ã£o estrutural, mas o software nÃ£o pode emitir ou transmitir documentos (NF-e/SIFEN) em sua versÃ£o de prateleira offline.
- **DecisÃ£o**: A matemÃ¡tica fiscal atua como um "Espelho TÃ©cnico/Preview". A funÃ§Ã£o calcula o imposto e salva os valores em colunas `fiscal_*_preview` apenas para documentaÃ§Ã£o/validaÃ§Ã£o visual na interface, sem alterar o valor original da venda, o estoque, os lanÃ§amentos financeiros de contas a receber ou as movimentaÃ§Ãµes de caixa.
- **ConsequÃªncia**: PreparaÃ§Ã£o estrutural massiva e completa, mas preservaÃ§Ã£o estrita da nÃ£o-emissÃ£o fiscal. Nenhum contador ou Ã³rgÃ£o governamental recebe essas informaÃ§Ãµes a partir deste terminal.

---

## ðŸ”¢ ADR 28: PadronizaÃ§Ã£o de AlÃ­quotas em Minor Unit Escala 6

- **Contexto**: AlÃ­quotas percentuais fiscais exigem extrema precisÃ£o matemÃ¡tica para evitar perdas ou distorÃ§Ãµes de centavos (ex: 10,5% de R$ 5,00). 
- **DecisÃ£o**: Foi explicitamente rejeitado o uso de `double` ou `float` para persistÃªncia e cÃ¡lculos. Adotou-se o armazenamento de alÃ­quotas em `i64` multiplicando o percentual visual por 10.000 (Escala 6). Ex: `10.5%` torna-se o inteiro `105000`. O cÃ¡lculo final Ã© efetuado por `(base_minor * aliquota_escala_6) / 1_000_000`. 
- **ConsequÃªncia**: Garantia financeira determinÃ­stica sem arredondamentos inesperados no hardware local. As mÃ¡scaras de float/decimal (`step="0.01"`) foram permitidas exclusivamente na camada de interface Blazor.

---

## ðŸ§¾ ADR 29: Retaguarda Fiscal como Fonte Mestre e PDV como Consumidor de Pacotes Fiscais Versionados (Fase 17)

- **Contexto**: A Fase 16 criou as tabelas `fiscal_*_cache` no SQLite do PDV com dados fiscais estÃ¡ticos inseridos manualmente. Era necessÃ¡rio um mecanismo controlado, versionado e auditÃ¡vel para atualizar esses dados a partir de uma fonte centralizada.
- **DecisÃ£o**: Adotou-se o modelo **Publisher/Subscriber Fiscal**:
  1. A **Retaguarda/PostgreSQL** Ã© a Ãºnica fonte de verdade de dicionÃ¡rios e regras fiscais.
  2. O administrador publica uma **versÃ£o fiscal** com payload JSON consolidado.
  3. O payload Ã© armazenado em `pacotes_sincronizacao` com `tipo_pacote = 'SYNC_FISCAL'`.
  4. Os **PDVs** recebem e aplicam o pacote de forma idempotente nas tabelas `fiscal_*_cache`.
  5. Nenhum PDV edita, cria ou transmite dados fiscais para autoridades.
- **MotivaÃ§Ã£o**:
  - Garantir que todos os PDVs operam com a mesma versÃ£o de regras fiscais.
  - Permitir rollout controlado de atualizaÃ§Ãµes fiscais (ex: mudanÃ§a de alÃ­quota ICMS).
  - Manter rastreabilidade completa via `fiscal_auditoria_mestre` e `fiscal_versoes_publicacao`.
  - Isolar a responsabilidade: Retaguarda = governanÃ§a fiscal; PDV = execuÃ§Ã£o local.
- **LimitaÃ§Ãµes aceitas**:
  - O payload atual Ã© enviado como JSON Ãºnico (sem chunking). Para bases fiscais massivas (>10.000 NCM/CFOP), serÃ¡ necessÃ¡ria paginaÃ§Ã£o em fase futura.
  - A aplicaÃ§Ã£o manual de pacotes via UI Ã© apenas para diagnÃ³stico/homologaÃ§Ã£o tÃ©cnica.
- **ConsequÃªncia**: Arquitetura clara, auditÃ¡vel e preparada para futuras fases de emissÃ£o fiscal real (NF-e, NFC-e, SIFEN), sem comprometer a estabilidade operacional atual do PDV.

---

# Registro de Decisões de Projeto (ADR) — Fase 18

Decisões de arquitetura adotadas na Fase 18 — Homologação Técnica Fiscal: Certificados, Assinatura, XML/JSON Preview e Validação Local.

---

## ?? ADR 30: Certificado A1 Exclusivo na Retaguarda
- **Contexto**: Para assinar documentos fiscais, é necessário ler certificados digitais (A1, A3, HSM).
- **Decisão**: Apenas certificados A1 (arquivos PFX/P12) são suportados, e eles residem exclusivamente na Retaguarda. Certificados A3/HSM ficaram fora de escopo. A chave privada e a senha nunca são persistidas em banco de dados ou logadas.
- **Consequência**: Simplifica a gestão e evita problemas complexos de drivers locais. A Retaguarda vira um hub centralizado de assinatura. A chave não viaja pela rede para o PDV local.

---

## ?? ADR 31: Assinatura Técnica Preview (Sem XMLDSig definitivo)
- **Contexto**: A assinatura XMLDSig com C14N exigida pela SEFAZ exige bibliotecas criptográficas específicas (como libxmlsec1), difíceis de compilar estaticamente no Windows de forma portátil em cross-compilation.
- **Decisão**: A Fase 18 implementa uma assinatura RSA-SHA256 puramente técnica (preview), injetando uma tag <Signature> simplificada. A assinatura XMLDSig oficial fica como pendência técnica para a fase de transmissão real.
- **Consequência**: Permite validar todo o fluxo de leitura do certificado, extração de chaves e geração do hash, mantendo a compilação do projeto simples, mas bloqueando intencionalmente qualquer tentativa de usar a nota como documento fiscal legal.

---

## ?? ADR 32: Espelho Preview Sem Transmissão e Sem Validade Jurídica
- **Contexto**: O sistema precisa montar XML NFC-e/NF-e e JSON SIFEN/DTE, além de validá-los, mas sem transmitir para os órgãos competentes.
- **Decisão**: Todos os endpoints de preview (XML e JSON) geram arquivos com ambiente obrigatoriamente definido para HOMOLOGACAO (tpAmb=2), injetam avisos de DOCUMENTO TECNICO DE HOMOLOGACAO SEM VALIDADE FISCAL, e bloqueiam qualquer requisição de PRODUCAO. O QR gerado é SVG base64 sem cHashQR oficial.
- **Consequência**: A arquitetura técnica foi criada e validada (estruturação, formatação de minor units, cálculo segregado de IVA), garantindo que o PDV e a retaguarda estão preparados para integração real sem expor clientes a riscos fiscais de emissão indevida.

---

## ðŸ’¡ ADR 33: XMLDSig real via xmlsec/libxmlsec atrÃ¡s de feature fiscal_xmldsig_real
- **Contexto**: A assinatura de XML padrÃ£o SEFAZ (com canonicalizaÃ§Ã£o e C-HASH) requer manipulaÃ§Ã£o avanÃ§ada usando ferramentas C que dificultam o build multi-plataforma no Windows.
- **DecisÃ£o**: Isolar a dependÃªncia tÃ©cnica por trÃ¡s de uma macro condicional (`cfg(feature = "fiscal_xmldsig_real")`). O runtime local e laboratÃ³rio Docker em Linux/WSL serÃ£o usados para testes, enquanto o build padrÃ£o ignorarÃ¡ o binding.
- **ConsequÃªncia**: Preserva a experiÃªncia de desenvolvimento (cargo check rÃ¡pido no Windows) enquanto provÃª uma infraestrutura de produÃ§Ã£o escalÃ¡vel e correta.

---

## ðŸ’¡ ADR 34: Bloqueio absoluto de produÃ§Ã£o na Fase 19
- **Contexto**: Testar conexÃµes e URLs de serviÃ§os do governo acarreta riscos, inclusive emitir notas ou rejeiÃ§Ãµes em ambiente oficial de ProduÃ§Ã£o.
- **DecisÃ£o**: Bloqueio hard-coded para qualquer URL contendo ambientes de ProduÃ§Ã£o e obrigatoriedade da flag `tpAmb=2`.
- **ConsequÃªncia**: Zero chance de uma nota vazar para produÃ§Ã£o acidentalmente no perÃ­odo de desenvolvimento e validaÃ§Ã£o tÃ©cnica da Fase 19.

---

## ðŸ’¡ ADR 35: HomologaÃ§Ã£o real depende de artefatos externos e runtime Linux/WSL
- **Contexto**: Homologar e garantir as integraÃ§Ãµes nÃ£o pode ser falsificado via mocks.
- **DecisÃ£o**: Aceitar "PendÃªncias Externas" (ausÃªncia de `xsd` fÃ­sico, ambiente `WSL`, certificados A1 PFX nÃ£o commitados) como bloqueadores mapeados, nÃ£o ocultos. A API reporta o que falta, em vez de simular sucesso.
- **ConsequÃªncia**: TransparÃªncia para equipe Ops/DevOps. Os deploys passam a ter um painel de prontidÃ£o (Readiness) explÃ­cito.

---

## ðŸ’¡ ADR 36: ProntidÃ£o fiscal nÃ£o equivale a autorizaÃ§Ã£o fiscal
- **Contexto**: Com todos os testes verdes, usuÃ¡rios poderiam assumir que notas estÃ£o valendo.
- **DecisÃ£o**: Criar a entidade ProntidÃ£o que reflete APENAS a infraestrutura ("tenho rede, tenho certificado, tenho libxmlsec"). Inserir banners de aviso de que "ProntidÃ£o nÃ£o Ã© autorizaÃ§Ã£o".
- **ConsequÃªncia**: Previne interpretaÃ§Ãµes dÃºbias e a falsa sensaÃ§Ã£o de que a emissÃ£o em si jÃ¡ estÃ¡ valendo. Protege contra problemas jurÃ­dicos.

---

## ðŸ’¡ ADR 37: Licenciamento Local Offline-First com TolerÃ¢ncia
- **Contexto**: PDVs frequentemente operam em ambientes com internet instÃ¡vel, mas o licenciamento de software SaaS depende de validaÃ§Ã£o online para prevenir pirataria e garantir cobranÃ§a.
- **DecisÃ£o**: Implementar uma arquitetura de duas camadas. O PDV consulta localmente (SQLite) as tabelas `licenca_local` e `instalacao_local` que mantÃªm o estado de `pode_operar`, tolerÃ¢ncia (ex: 10 dias) e Ãºltimo check. As regras de venda olham apenas para o estado local, garantindo zero latÃªncia.
- **ConsequÃªncia**: Aumenta a robustez operacional na ponta (o PDV nÃ£o trava no meio do expediente por queda de internet). No entanto, exigirÃ¡ (nos blocos subsequentes) um job de sincronizaÃ§Ã£o de fundo (sync de licenÃ§a) confiÃ¡vel e criptografado para evitar adulteraÃ§Ã£o do banco SQLite local.

---

## ðŸ’¡ ADR 38: Retaguarda como Fonte Mestre de Licenciamento Comercial
- **Contexto**: Para viabilizar a comercializaÃ§Ã£o do Aureon (PDV e Retaguarda), precisamos de um sistema de validaÃ§Ã£o que garanta que o cliente estÃ¡ com a fatura em dia e respeitando limites de caixas. Contudo, o PDV nÃ£o pode parar por falta momentÃ¢nea de internet.
- **DecisÃ£o**: A Retaguarda em Nuvem (PostgreSQL) serÃ¡ a fonte mestre, detendo toda a regra de negÃ³cio comercial, planos, tokens e status. O PDV local farÃ¡ cache assinado dessa licenÃ§a (ADR 37) com tolerÃ¢ncia de dias. Em caso de bloqueio gerado na retaguarda, o PDV localiza e impÃµe a barreira em sua prÃ³xima janela de conexÃ£o ou esgotamento de prazo.
- **ConsequÃªncia**: Isola a complexidade comercial no servidor cloud. O PDV sÃ³ precisa saber ler e validar o cache criptogrÃ¡fico da licenÃ§a enviada pela retaguarda. Facilita integraÃ§Ãµes com gateways (Stripe/Asaas) no futuro.

## ADR-020-004 â€” Licencas locais verificaveis offline por assinatura assimetrica
**Data**: 2026-05-25
**Status**: Aprovado
**Fase**: 20 Bloco 4

### Contexto
O PDV opera em modo offline-first. O payload de licenca precisa ser verificavel localmente sem internet, mas nao pode ser adulterado pelo operador ou por software malicioso.

### Decisao
Usar assinatura assimetrica Ed25519 (RFC 8032) para assinar payloads de licenca na Retaguarda. A chave privada fica exclusivamente na Retaguarda. A chave publica e distribuida ao PDV para verificacao offline.

### Algoritmo escolhido: Ed25519
- Alternativas consideradas: ECDSA P-256, RSA-PSS, HMAC-SHA256.
- Ed25519 venceu por: assinatura de 64 bytes (compacta), sem parametros de curva, resistencia nativa a timing attacks, velocidade superior ao ECDSA.
- HMAC descartado: exige segredo compartilhado no PDV, criando risco de exposicao.

### Canonicalizacao
- Campos do payload em ordem fixa explicita (nao depende de serde_json).
- Sem floats, sem campos opcionais ausentes.
- SHA-256 do payload canonico e o objeto assinado (nao o payload bruto).

### Consequencias
- PDV podera validar licencas offline com chave publica local (proximos blocos).
- Rotacao de chaves e suportada via key_id.
- Modo DEV usa chave efemera com warning explicito.
- Chave privada nunca sai da Retaguarda.

## ADR: Bloqueio Operacional Suave e Limitado (Fase 20, Bloco 8)

### Contexto
O licenciamento não pode destruir dados ou impedir definitivamente a regularização. Se o sistema travar todas as telas em caso de licença expirada, o operador não consegue acessar o próprio menu de licença para importar um payload assinado offline ou conectar e sincronizar.

### Decisão
Implementar uma "Guarda Operacional" (Bloqueio Suave) restrita a operações críticas do negócio.

### Restrições
- Operações críticas (ex: ABRIR_CAIXA, FINALIZAR_VENDA) chamam a guarda antes de iniciar.
- O bloqueio só ocorre se o nível da licença for explicitamente BLOQUEADA ou EXPIRADA fora do período de tolerância offline.
- A tela de licença, backup, relatórios de leitura e rotinas de sincronização NUNCA são bloqueados.
- Todas as decisões são auditadas na tabela `licenca_eventos`.

## ADR: Backup e Restauração Independentes de Licença (Fase 20, Bloco 9)

### Contexto
Sistemas PDV críticos precisam de rotinas de backup locais que garantam a preservação e a recuperação de dados independente de pagamentos ou bloqueios de licença. Cortar acesso à base de dados que pertence ao cliente é destrutivo.

### Decisão
A rotina de criação, listagem, validação e restauração de backups locais deve estar disponível para o operador independentemente do status da licença (`OK`, `EXPIRADA`, `BLOQUEADA`, etc). 

### Consequências
- A interface de backup e os commands de backup no Tauri não têm validações de `garantir_operacao_licenciada`.
- O cliente sempre tem o direito e a habilidade de salvar seus dados localmente e recuperar o PDV se algo der errado (ex: atualização falha, corrupção).

