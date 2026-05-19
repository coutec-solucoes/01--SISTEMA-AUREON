# Configuração de Impressoras e Periféricos de Checkout

Este documento consolida a parametrização de hardware térmico, gavetas, leitores, displays de torre e painéis eletrônicos de senhas.

---

## 🖨️ Impressoras Térmicas
As impressoras são configuradas com base na emulação genérica `ESC/POS` ou protocolos proprietários locais.
- **Tipos de Conexão**:
  - `USB`: Comunicação direta por porta serial virtual ou spooler do sistema operacional.
  - `REDE`: Protocolo TCP/IP direto (geralmente porta `9100`).
  - `COMPARTILHADA`: Endereço UNC do Windows (`\\NomeDoPC\NomeImpressora`).
- **Dimensões e Largura**:
  - 48 Colunas: Largura padrão de bobina térmica de 80mm.
  - 40/32 Colunas: Largura compacta para bobinas de 58mm.
- **Fluxo de Teste**:
  O botão de teste dispara o comando de corte de papel (cutter) e impressão de linhas padrão via protocolo selecionado.

---

## 💵 Gavetas e Periféricos Gerais
- **Gaveta de Dinheiro**:
  - Acionada por pulso elétrico RJ11 conectado diretamente na saída da impressora de cupom, ou via conexão direta por porta serial.
- **Leitor de Código de Barras**:
  - Emulação de teclado padrão ou via porta serial virtual para detecção assíncrona de gatilho.
- **Display de Cliente**:
  - Displays torre VFD de duas linhas para exibição de valores de itens e mensagens promocionais aos clientes durante a passagem das mercadorias.

---

## 📺 Painéis Eletrônicos (Senhas e Chamadas)
Configurações estruturais de exibição para atendimento de clientes em fila:
- **Painel de Senhas**: Exibição da senha de atendimento, mesa ou número do pedido.
- **Sintetizador de Voz (TTS - Text To Speech)**: Sinalização opcional de voz para ditar o número e o guichê/mesa da chamada.
- **Tempo Médio de Espera**: Exibição dinâmica na TV para controle visual do fluxo pelo cliente.

*Nota: Todas as rotas de hardware e painéis eletrônicos estão limitadas nesta fase a parâmetros estruturais de banco e interface de simulação, sem controle real de chamadas no painel operacional.*
