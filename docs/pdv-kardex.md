# Kardex e Rastreabilidade Imutável

## O que é o Kardex
Kardex (Tabela: `estoque_movimentacoes`) é a trilha de auditoria contábil de vida de todos os produtos do estoque.
Cada mudança de saldo — seja ela originada por vendas, ajustes de quebra, recebimento de mercadoria ou inventários manuais — insere um novo evento nesta tabela descrevendo a data, operação exata, tipo de movimento, motivo, origem do evento e ID atrelado.

## Regra de Ouro: Imutabilidade Absoluta
Nenhum registro pode ser atualizado (`UPDATE`) ou deletado (`DELETE`).
Qualquer falha ou cancelamento se dá via evento compensatório no formato de **Estorno**.

## Formatação do Saldo "Post-Facto"
Dentro de cada movimentação do Kardex, existe o campo `saldo_apos_escala3`.
Esse campo registra instantaneamente em formato integer o saldo residual daquele momento temporal do produto após a execução daquele evento. Permite gerar relatórios visuais muito mais práticos na tela.

## Tipos de Movimentações Operacionais
- `VENDA`: Ocorre ao finalizar o ciclo de caixa da venda.
- `ESTORNO_VENDA`: Ocorre ao invalidar uma venda que já fora finalizada. Devolve quantitativamente o produto ao estoque.
- `AJUSTE_ENTRADA`: Bonificações, produtos devolvidos avulsos.
- `AJUSTE_SAIDA`: Quebras, furtos, perdas e estragos com motivo imputável.
- `INVENTARIO`: Ajuste bruto de delta proveniente de uma batida técnica de correção entre o físico e o sistema.
