# Fase 15 — Impressão Operacional, Comprovantes, Recibos, Cupons Não Fiscais e Layouts de Documentos

## Resumo

A Fase 15 implementou toda a camada de impressão térmica operacional não fiscal do Aureon PDV. A impressão é tratada estritamente como **saída documental de controle interno**. Nenhum comando desta fase altera vendas, saldos de caixa, estoques, financeiro, delivery ou produção (exceto o pulso físico de gaveta e registros pontuais de auditoria).

---

## Blocos Implementados

| Bloco | Conteúdo | Commit |
|---|---|---|
| Bloco 1 | Infraestrutura ESC/POS, Builder, Simulador, TCP/IP, Teste de Impressora | (pré-checkpoint) |
| Bloco 2 | Cupom de Venda Não Fiscal e Reimpressão | (pré-checkpoint) |
| Bloco 3 | Comprovantes Financeiros e Comprovantes de Caixa | `3cf7967` |
| Bloco 4 | Tickets de Produção, Cancelamento, Romaneio Delivery e Gaveta | `359c94e` |
| Bloco 5 | UI Blazor: Central de Reimpressões e Modal de Destino | `48cbfe5` |
| Bloco 6 | Homologação e Documentação Final | (este commit) |

---

## Commands Criados (Tauri/Rust)

| Command | Descrição |
|---|---|
| `testar_impressora` | Envia cupom de teste, retorna caminho do simulador |
| `imprimir_cupom_venda_nao_fiscal` | Cupom completo de venda (não fiscal) |
| `reimprimir_cupom_venda_nao_fiscal` | Reimpressão com motivo obrigatório e auditoria |
| `imprimir_comprovante_baixa_financeira` | Comprovante de baixa financeira (pagamento/recebimento) |
| `imprimir_comprovante_movimentacao_caixa` | Comprovante avulso de sangria, suprimento ou vale |
| `imprimir_comprovante_abertura_caixa` | Extrato de abertura da sessão de caixa |
| `imprimir_comprovante_fechamento_caixa` | Extrato de fechamento da sessão de caixa |
| `imprimir_resumo_gerencial_caixa` | Resumo gerencial completo com totais por moeda |
| `imprimir_ticket_producao` | Ticket de produção para cozinha/bar (sem preços) |
| `imprimir_ticket_cancelamento_producao` | Ticket de cancelamento de item ou envio de produção |
| `imprimir_romaneio_delivery` | Romaneio de expedição do pedido delivery |
| `abrir_gaveta_dinheiro` | Pulso físico de gaveta + registro de auditoria |

---

## DTOs Criados

### Rust (`aureon-core/src/dtos.rs`)
- `TesteImpressoraReq`
- `ImprimirVendaReq`
- `ReimprimirVendaReq`
- `ImprimirBaixaFinanceiraReq`
- `ImprimirMovimentacaoCaixaReq`
- `ImprimirSessaoCaixaReq`
- `ImprimirProducaoReq`
- `ImprimirCancelamentoProducaoReq`
- `ImprimirRomaneioDeliveryReq`
- `AbrirGavetaReq`
- `ImpressoraDestinoReq` (destino unificado)
- `ResultadoImpressao` (retorno padrão)

### C# (`ui-blazor/Services/PdvModels.cs`)
Espelhos em C# de todos os DTOs acima, mais:
- `ImpressoraDestinoReq` como `class` com `{ get; set; }` (necessário para binding Blazor)
- Enum `TipoDestinoImpressao` (`SIMULADOR`, `TCP_IP`, `WINDOWS_RAW`)

---

## Telas Blazor Criadas

| Arquivo | Rota | Função |
|---|---|---|
| `Pages/ReimpressoesPdv.razor` | `/reimpressoes` | Central de documentos e ferramentas de impressão |
| `Shared/ImpressoraDestinoModal.razor` | — | Modal de configuração do destino de impressão |

Link **"Impressão"** adicionado ao `MainLayout.razor`.

---

## Destinos de Impressão

| Destino | Status | Descrição |
|---|---|---|
| `SIMULADOR` | ✅ Funcional | Grava arquivo `.txt` e `.escpos.txt` em `C:/Aureon/print-sim/` |
| `TCP_IP` | ✅ Funcional | Conecta via socket TCP com timeout de 3 segundos |
| `WINDOWS_RAW` | ⚠️ Stub | Esqueleto presente, sem implementação real do spooler RAW |

---

## Layouts ESC/POS Implementados

| Documento | Setor de uso | Tem valores? |
|---|---|---|
| Cupom de venda não fiscal | Caixa / Cliente | ✅ Sim |
| Comprovante de reimpressão | Caixa / Auditoria | ✅ Sim |
| Comprovante de baixa financeira | Financeiro | ✅ Sim |
| Comprovante de movimentação de caixa | Caixa | ✅ Sim |
| Extrato de abertura de caixa | Caixa | ✅ Sim |
| Extrato de fechamento de caixa | Caixa | ✅ Sim |
| Resumo gerencial de caixa | Gerência | ✅ Sim |
| Ticket de produção | Cozinha/Bar | ❌ Sem preços |
| Ticket de cancelamento de produção | Cozinha/Bar | ❌ Sem preços |
| Romaneio de delivery | Expedição | ✅ Sim |
| Cupom de teste | Diagnóstico | — |

---

## Estratégia ESC/POS

- Builder próprio (`EscPosBuilder`) implementado em Rust puro.
- Suporte a largura de 32, 42 e 48 colunas.
- Comandos utilizados: bold, center, left, right, corte de papel (`ESC i`), pulso de gaveta (`ESC p`).
- Sem dependência de bibliotecas externas de terceiros.
- Compatível com impressoras Elgin, Daruma, Epson TM-T20, Bematech e similares.

---

## Estratégia do Simulador

- Destino padrão em todos os formulários da UI.
- Grava dois arquivos por impressão:
  - `aureon_YYYY-MM-DD_HH-MM-SS.txt` — versão legível em texto puro.
  - `aureon_YYYY-MM-DD_HH-MM-SS.escpos.txt` — bytes ESC/POS em hexadecimal.
- Diretório padrão: `C:/Aureon/print-sim/`.
- Permite validação completa de layouts sem hardware físico.
- O `ResultadoImpressao` retorna `caminho_arquivo_simulado` para exibição na UI.

---

## Status TCP/IP

- Implementado com `TcpStream::connect_timeout` (3 segundos).
- Erro de conexão retorna mensagem amigável sem panic.
- Risco controlado: pode travar a thread por até 3 segundos em impressora offline.
- Mitigação futura: mover para thread separada com `tokio::spawn`.

---

## Status Windows RAW

- Stub declarado no `match` do destino de impressão.
- Retorna erro amigável: "Destino WINDOWS_RAW não implementado nesta versão".
- Não quebra build multiplataforma (Linux/macOS).
- Pendência registrada para implementação futura com `winapi`/`windows` crate.

---

## Regras de Não Fiscal

- Todo documento exibe obrigatoriamente:
  ```
  *** DOCUMENTO NAO FISCAL ***
  NAO E VALIDO COMO DOCUMENTO FISCAL
  ```
- Nenhum documento emite XML, QR fiscal, assinatura digital ou DANFE.
- Termos proibidos na UI e nos layouts: NFC-e, NF-e, SAT, SIFEN, SEFAZ, DANFE.
- Tickets de produção ocultam valores financeiros intencionalmente.

---

## Regras de Matemática

- Dinheiro: `i64` em minor unit (ex: R$ 10,50 = `1050`).
- Quantidade: `i64` em escala 3 (ex: 1,500 kg = `1500`).
- Proibido `f64`, `f32`, `REAL`, `FLOAT`, `DOUBLE` em regras operacionais.
- Conversão para exibição ocorre apenas na camada visual (C# `decimal / 100m`).

---

## Limitações Conhecidas

1. Windows RAW Spooler não implementado (stub).
2. TCP/IP pode bloquear thread por até 3 segundos.
3. Fila assíncrona de impressão não implementada (fire-and-forget).
4. Teste físico em hardware real pendente para homologação final.
5. `CurrentUserId` na UI usa valor mock `"UI-USER"` até integração com serviço de autenticação.
6. Ticket de produção exibe "N/A" para setor se `setores_producao_cache` não estiver populada.

---

## Fora do Escopo da Fase 15

- NFC-e, NF-e, SAT, SIFEN ou qualquer documento fiscal eletrônico.
- Assinatura digital de documentos.
- Comunicação com SEFAZ ou órgãos fiscais.
- Impressão de etiquetas (balança/gôndola) — módulo separado.
- Integração com chamador de senha eletrônica.
- Fila assíncrona de impressão com retry automático.
- Impressão de boletos bancários.
- PDF/HTML de documentos para email ou portal web.
