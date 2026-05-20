# Idempotência Financeira — PDV Local

O sistema PDV opera localmente e offline-first. Para evitar duplicidades de cobrança ou pagamentos causados por cliques duplos na interface Blazor, latências no Tauri ou reexecuções de sincronização, implementamos mecanismos de idempotência tanto na geração de contas a pagar quanto nas contas a receber.

## 1. Geração de Contas a Receber (Vendas em Crediário)

Durante o fluxo de finalização de vendas (`finalizar_venda`):
* **Checagem de Duplicidade por Venda ID**: Antes de inserir o título em `contas_receber`, o backend Rust realiza uma checagem relacional na tabela de contas a receber para verificar se já existe algum registro associado àquela `venda_id`:
  ```sql
  SELECT EXISTS(SELECT 1 FROM contas_receber WHERE venda_id = ?1)
  ```
* **Comportamento**: Caso o título já exista, o backend simplesmente ignora a nova inserção silenciosamente (ou retorna a conta existente) dentro da transação atômica, evitando duplicar a dívida do cliente em caso de duplo envio do comando.

## 2. Geração de Contas a Pagar (Finalização de Compra)

Durante a finalização de uma Nota de Compra (`finalizar_compra`):
* **Checagem de Duplicidade por Compra ID**: Similar ao fluxo de vendas, antes de criar o título a pagar, é efetuada uma consulta para verificar se a `compra_id` já gerou um título a pagar:
  ```sql
  SELECT EXISTS(SELECT 1 FROM contas_pagar WHERE compra_id = ?1)
  ```
* **Comportamento**: Em caso positivo, o backend ignora o insert e preserva o estado original. Isso garante a consistência do módulo financeiro mesmo diante de concorrências.

## 3. Imutabilidade do Livro-Caixa (`financeiro_lancamentos`)

* **Garantia por Chave Primária**: Cada baixa de conta a pagar (`baixar_conta_pagar`) ou a receber (`baixar_conta_receber`) gera um UUID único para o lançamento financeiro.
* **Operações Permitidas**: Apenas `INSERT` é suportado no código Rust para a tabela de lançamentos. Não há comandos Tauri ou métodos de banco que executem `UPDATE` ou `DELETE` nessa tabela. Toda correção financeira necessita de um lançamento estornador ou cancelamento, de forma que o rastro cronológico nunca possa ser maquiado ou sobrescrito por erros de concorrência ou ações fraudulentas.
