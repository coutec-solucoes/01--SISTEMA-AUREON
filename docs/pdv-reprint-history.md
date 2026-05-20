# Histórico de Vendas e Reimpressão de Comprovante Não Fiscal

Este documento descreve o fluxo de auditoria, listagem e reimpressão de comprovantes não fiscais gerados localmente pelo PDV.

---

## 🧾 Comprovante Não Fiscal TXT

A impressão não fiscal é gerada pelo Tauri em formato de texto limpo (.txt), estruturado em blocos para bobinas térmicas de 80mm ou 58mm:

- **Cabeçalho**: Dados fictícios da empresa, data/hora e número da venda.
- **Lista de Itens**: Quantidade (formato 3 casas decimais), descrição, preço unitário e valor total do item.
- **Totais**: Subtotal, descontos, acréscimos e valor líquido.
- **Multimoeda**: Relação detalhada de quanto foi pago em cada moeda (BRL, USD, PYG) e taxas cambiais aplicadas.
- **Troco**: Detalhamento do troco entregue e moeda correspondente.

---

## 🔒 Auditoria de Reimpressão

Reimprimir um cupom é considerado uma operação sensível em frentes de caixa devido ao risco de fraude de entrega de mercadorias duplicadas.

1. **Justificativa Obrigatória**: O operador deve inserir uma justificativa textual para a reimpressão.
2. **Autorização de Supervisor**: A reimpressão exige a inserção de um PIN de supervisor válido.
3. **Persistência do Registro**: Salva-se o log em `vendas_reimpressoes` contendo `venda_id`, `operador_id`, `supervisor_id`, `justificativa`, `numero_reimpressao` e `criado_em`.
4. **Outbox**: Gera-se o evento `COMPROVANTE_REIMPRESSO` contendo o payload da auditoria.
