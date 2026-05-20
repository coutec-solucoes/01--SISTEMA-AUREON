# Gestão de Custos e Câmbio na Compra

A Fase 12 introduz o controle de custos base de produtos com base nas notas de compras manuais, fornecendo suporte completo a transações multimoedas comuns em regiões fronteiriças (como Dólar e Guarani).

## Cotação Snapshot da Nota

A cotação de câmbio é travada na criação da compra:

- O usuário seleciona a moeda da nota (ex: USD).
- A cotação informada é salva no campo `taxa_cambio_escala6` da tabela `compras`.
- **Snapshot Imutável**: Esta taxa atua como um retrato fixo daquele instante de compra. Mesmo que a cotação geral do sistema mude posteriormente, o valor cambial registrado na nota permanece inalterado, garantindo a integridade dos relatórios contábeis e fiscais históricos.

## Conversão de Custo sem Floats

Para cumprir a regra de ouro de matemática puramente inteira do Aureon, os custos dos itens informados na moeda de compra são convertidos para BRL (moeda principal do sistema) usando multiplicação inteira com escala fixa:

$$\text{custo\_convertido\_minor} = \frac{\text{custo\_unitario\_minor} \times \text{taxa\_cambio\_escala6}}{1.000.000}$$

### Exemplo de Cálculo

Suponha uma compra de produto com custo de **$ 10,50 USD** e taxa de câmbio a **R$ 5,250000 BRL/USD**:

1. **Custo Unitário (Minor Units)**: `1050` (equivalente a $ 10,50).
2. **Taxa de Câmbio (Escala 6)**: `5250000` (equivalente a 5,250000).
3. **Cálculo de Conversão**:
   $$\text{custo\_convertido\_minor} = \frac{1050 \times 5250000}{1000000} = \frac{5512500000}{1000000} = 5512,5 \approx 5513$$
4. **Custo em BRL**: `5513` minor units (R$ 55,13).

## Atualização de Último Custo do Produto

Ao finalizar a compra, o sistema executa o reajuste do custo do produto localmente no cache:

- **Atualização**: O campo `ultimo_custo_minor` na tabela `produtos_cache` é atualizado com o `custo_convertido_minor` calculado a partir da última entrada finalizada.
- **Simplificação PDV**: O terminal PDV atualiza apenas o *último custo*, sem recalcular Preço Médio Ponderado (PMP). Esta simplificação mantém a performance do banco de dados local leve para vendas rápidas. O cálculo de PMP e reajustes complexos de margens ficam delegados para o software de retaguarda centralizado na nuvem (PostgreSQL) após receber os outboxes de compras.
- **Preço de Venda Isolado**: O preço de venda do produto (`preco_venda_minor` em `produtos_cache`) **nunca é alterado automaticamente** por este fluxo de compra manual, protegendo as operações de caixa contra reajustes não autorizados ou oscilações cambiais acidentais.
