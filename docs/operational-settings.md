# Configurações Operacionais Globais

Este documento detalha o funcionamento operacional dos parâmetros globais do sistema e sua modelagem lógica.

---

## ⚙️ Escopo Geral
As configurações operacionais globais definem como as lojas, terminais de venda e registradoras físicas devem se comportar durante o expediente de vendas. 

---

## 🖥️ Terminais de PDV (Estaçoes de Trabalho)
Os terminais são as máquinas autorizadas a rodar o aplicativo de PDV.
- **Tipos de Terminais**:
  - `FIXO`: Caixa de checkout padrão.
  - `MOVEL`: Celular, tablet ou terminal portátil de garçom/vendedor.
  - `TOTEM`: Terminal de autoatendimento.
- **Fluxo de Autorização**:
  1. Uma nova máquina tenta se conectar enviando seus metadados (IP, nome do dispositivo).
  2. O backend registra o terminal com status `ativo = false`.
  3. O administrador acessa a tela de configurações e autoriza a estação de trabalho.
  4. O terminal recebe a permissão para rodar a aplicação local.

---

## 💵 Registradoras (Caixas de Dinheiro)
Representa as gavetas de dinheiro físicas ou virtuais vinculadas a operadores de caixa.
- **Parâmetros**:
  - `multimoeda`: Habilita o recebimento e troco em moedas secundárias (Dólar, Euro, Peso).
  - `limite_suprimento`: Valor máximo de dinheiro permitido em gaveta antes de exigir sangria de segurança.
  - `chave_autorizacao_abertura`: Exige código de supervisor para abertura remota ou manual da gaveta.
- **Finalidade**:
  Subsidiar a futura abertura, fechamento e controle de fluxo do caixa financeiro diário (Fase 6).
