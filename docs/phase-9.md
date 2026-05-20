# Fase 9: PDV Gourmet (Mesas, Comandas, Transferências e Produção)

## 1. Resumo da Fase

A Fase 9 introduziu o módulo **Gourmet** ao Aureon PDV, com foco na operação de balcão estendida para restaurantes e bares. O objetivo foi permitir o controle de contas em aberto (Mesas e Comandas), adição de itens ao longo do tempo, envio para setores de produção (cozinha/bar) e posterior recebimento no caixa (integrando com a Fase 7).

## 2. Blocos Implementados

A fase foi executada de forma sequencial através de 5 blocos:

- **Bloco 1**: Migration 007 (`mesas_operacionais`, `comandas_operacionais`, `gourmet_itens`, `gourmet_transferencias`, `producao_envios` etc.) e Seed de Desenvolvimento separado.
- **Bloco 2**: Backend em Rust (Commands) para Mesas, Comandas e Itens Gourmet (abrir, reservar, bloquear, adicionar itens, etc.).
- **Bloco 3**: Backend em Rust (Commands) para Transferências, Produção e Fechamento em Venda.
- **Bloco 4**: Interface de Usuário (Blazor) com telas de listagem, painel de detalhes da mesa/comanda, modais de transferência e visualização do histórico de produção.
- **Bloco 5**: Testes de fluxo completo, builds finais e documentação oficial.

## 3. Estrutura de Dados (Migrations)

- **`007_fase9_gourmet.sql`**
  - Tabelas de cache recebidas do Master Data (`mesas_cache`, `comandas_cache`).
  - Tabelas de operação local (`mesas_operacionais`, `comandas_operacionais`, `gourmet_itens`).
  - Tabelas de transferência (`gourmet_transferencias`, `gourmet_transferencias_itens`).
  - Tabelas de produção (`producao_envios`, `producao_envios_itens`).

## 4. Commands Tauri (Rust)

Foram implementados comandos focados na manipulação do estado operacional, garantindo transacionalidade e registro no `sync_outbox`:

- **Mesas:** `listar_mesas_pdv`, `abrir_mesa`, `reservar_mesa`, `bloquear_mesa`, `cancelar_mesa`, `obter_mesa`.
- **Comandas:** `listar_comandas_pdv`, `abrir_comanda`, `bloquear_comanda`, `cancelar_comanda`, `obter_comanda_por_numero`, `obter_comanda`.
- **Itens:** `adicionar_item_mesa`, `cancelar_item_mesa`, `adicionar_item_comanda`, `cancelar_item_comanda`.
- **Transferência:** `transferir_mesa_total`, `transferir_itens_mesa`, `transferir_comanda_total`, `transferir_itens_comanda`.
- **Produção:** `enviar_itens_producao`, `gerar_texto_producao`, `reimprimir_envio_producao`, `listar_envios_producao`, `listar_todos_envios_producao`.
- **Fechamento:** `fechar_mesa_em_venda`, `fechar_comanda_em_venda`.

## 5. Interface Blazor (UI)

- **`Gourmet.razor`**: Painel central com abas (Mesas, Comandas, Produção).
- **`MesasPdv.razor` / `ComandasPdv.razor`**: Grid visual para acompanhamento dos status das origens e campo de busca rápida para comandas.
- **`MesaDetalhe.razor` / `ComandaDetalhe.razor`**: Interface para operação ativa da conta (adição, cancelamento, totalizadores, fechamento).
- **Modais de Transferência**: `TransferenciaMesaModal.razor` e `TransferenciaComandaModal.razor` com opções de transferência Total ou Parcial (via checkbox).
- **`ProducaoPdv.razor`**: Visualizador histórico de envios com mock textual da impressão física.

## 6. Eventos `sync_outbox` Adicionados

- `MESA_ABERTA`, `MESA_RESERVADA`, `MESA_BLOQUEADA`, `MESA_CANCELADA`, `MESA_CONVERTIDA_EM_VENDA`.
- `COMANDA_ABERTA`, `COMANDA_BLOQUEADA`, `COMANDA_CANCELADA`, `COMANDA_CONVERTIDA_EM_VENDA`.
- `MESA_ITEM_ADICIONADO`, `MESA_ITEM_CANCELADO`, `COMANDA_ITEM_ADICIONADO`, `COMANDA_ITEM_CANCELADO`.
- `MESA_TRANSFERIDA`, `MESA_ITENS_TRANSFERIDOS`, `COMANDA_TRANSFERIDA`, `COMANDA_ITENS_TRANSFERIDOS`.
- `ITEM_ENVIADO_PRODUCAO`, `ITEM_CANCELAMENTO_ENVIADO_PRODUCAO`, `PRODUCAO_ENVIO_GERADO`, `PRODUCAO_REIMPRESSAO_GERADA`, `PRODUCAO_CANCELAMENTO_GERADO`.

## 7. Regras de Negócio e Validações

- **Sem Float/Double**: Utiliza-se exclusivamente Inteiros em minor units (escala 2) para dinheiro e escala 3 para quantidades.
- **Inferência de Estado Livre**: O status `LIVRE` não é persistido nas tabelas operacionais. Se uma mesa/comanda não existe ou está finalizada (`FECHADA`, `CANCELADA`), ela é reportada como `LIVRE` ao consultar os dados de cache.
- **Bloqueio EM_ANDAMENTO**: Impede a manipulação de itens de uma mesa/comanda caso ela possua uma Venda `EM_ANDAMENTO` ativa (processo de pagamento iniciado).
- **Fechamento de Origem**: Apenas após o pagamento (em `commands_pagamento.rs`, Fase 7) a mesa/comanda recebe o status oficial de `FECHADA`.
- **Produção Incremental**: Enviar para produção afeta somente os itens ainda "pendentes", agrupando-os por setor de produção (`local_producao_id`).

## 8. O que FICOU DE FORA do escopo

Em respeito ao planejamento do Aureon PDV e para manter a complexidade gerenciável no módulo PDV nativo, os seguintes itens não foram implementados nesta fase:
- Delivery (rotas, taxa de entrega, iFood).
- App Garçom ou Painel de Autoatendimento.
- Emissão de NFC-e/NF-e/SIFEN/Fiscal Real.
- Integração TEF/Pix ativa com maquininhas de cartão.
- Baixa de Estoque ou Ficha Técnica (Kardex).
- Painel KDS (Kitchen Display System) completo com telas nas praças.
- Driver de Impressão Física obrigatória (utilizado mock TXT para simular).

---

A Fase 9 foi oficialmente concluída, cobrindo o ciclo completo da operação Gourmet em PDV.
