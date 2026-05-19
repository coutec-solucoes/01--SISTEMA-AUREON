# Parâmetros de PDV, Vendas e Orçamentos

Este documento detalha o comportamento lógico do Ponto de Venda (PDV) e as regras financeiras/comerciais de transação configuradas na Fase 5.

---

## 🛍️ Configurações Gerais do PDV
Os parâmetros globais do PDV definem a flexibilidade de operação no caixa:
- **Venda Sem Estoque**: Permite ou impede a comercialização de produtos sem saldo em estoque.
- **Modo Offline-First**: Configura o tempo máximo permitido de operação sem conexão com o servidor local central.
- **Desconto Máximo Habilitado**: Define a margem percentual limite que um operador de caixa pode aplicar a um item ou ao total da venda sem requisição de senha de supervisor.
- **Alteração de Preço Manual**: Habilita a livre digitação de valores de venda direto na tela do PDV para produtos parametrizados.

---

## 📈 Regras de Venda e Limites
Define controles restritivos aplicados automaticamente nas transações do PDV:
- **Valor Mínimo de Venda**: Impede a finalização de tickets com valores inferiores ao estabelecido.
- **Valor Máximo Sem Supervisor**: Transações acima desse limite exigem a validação de credencial de supervisor direto no checkout.
- **Limite de Itens por Venda**: Proteção contra saturação de memória física em impressoras térmicas ou sobrecarga visual do PDV.
- **Exigência de Motivo de Cancelamento**: Torna obrigatória a digitação justificada de qualquer cancelamento de item ou cupom completo, registrando na tabela de auditoria.

---

## 📄 Pré-Vendas e Orçamentos
Definições comerciais para geração e trâmite de pedidos prévios:
- **Pré-Vendas**:
  - Habilitação no PDV.
  - Horas de validade (expiração automática após o prazo).
  - Reserva de estoque preventiva de itens lançados.
  - Exigência ou não de cliente cadastrado para emissão.
- **Orçamentos**:
  - Habilitação na retaguarda e PDV.
  - Validade em dias comerciais.
  - Identificação fiscal obrigatória do cliente.
  - Sem reserva de estoque física ativa.
