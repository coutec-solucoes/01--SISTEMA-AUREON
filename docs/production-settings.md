# Setores de Produção, Mesas e Comandas Gourmet

Este documento descreve a organização interna de atendimento gourmet e direcionamento de impressão por setor.

---

## 🍽️ Setores de Preparação e Produção
Divisões físicas do estabelecimento onde são preparados alimentos ou bebidas:
- **Exemplos de Setores**: Cozinha Quente, Pizzaria, Copa/Bar, Churrasqueira.
- **Roteamento de Impressão**:
  Cada setor de produção possui um vínculo direto a uma impressora térmica de rede. Quando um pedido é finalizado em uma mesa/comanda pelo garçom:
  - Os itens de cozinha são enviados à impressora da cozinha.
  - As bebidas são enviadas à impressora do bar.
  - As pizzas são enviadas à impressora do setor pizzaria.
  - Isso otimiza os tempos de preparo e elimina gargalos operacionais.

---

## 🛋️ Mesas (Atendimento Presencial)
Representação lógica do mapa de mesas do estabelecimento:
- **Configuração de Mesas**:
  - Capacidade máxima de pessoas por mesa.
  - Setores físicos de localização das mesas (ex: Área Externa, Mezanino, Salão).
- **Parâmetros Globais Gourmet**:
  - Valor ou percentual de taxa de serviço (garçom).
  - Couvert artístico fixo por pessoa.
  - Permissão de transferência de itens e agrupamento entre mesas distintas.
  - Impressão automática de pré-conta ao solicitar encerramento.

---

## 📋 Comandas (Atendimento Rápido e Fichas)
Controle de consumo individual ou por grupo baseado em cartões numerados:
- **Configuração de Comandas**:
  - Cadastro de faixas numéricas de comandas ativas (ex: 1 a 500).
  - Vínculos individuais de códigos de barras ou QR codes para leitura e lançamento instantâneo via celular do garçom.
  - Impedimentos de liberação de comanda bloqueada por extravio ou pendências.
