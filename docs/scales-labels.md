# Balanças e Layouts de Etiquetas Comerciais

Este documento detalha o suporte e a integração lógica para pesagem em tempo real e leitura de etiquetas geradas em balanças de setor.

---

## ⚖️ Balanças de Checkout (Pesagem Direta)
Integração de balanças de pesagem rápida acopladas fisicamente à estação do caixa:
- **Modelos e Protocolos**: Toledo (P03), Filizola, Urano (POP) e Welmy.
- **Portas de Conexão**: Portas Seriais Físicas ou virtuais (COM1, COM2) e redes TCP/IP.
- **Leitura em Tempo Real**:
  O PDV realiza requisições assíncronas de leitura na porta configurada, interpreta o byte array retornado pelo hardware e extrai o peso líquido da balança em KG.

---

## 🏷️ Layouts de Etiquetas (Balança de Setor)
Para produtos pesados no setor (ex: Açougue, Padaria, Hortifrúti) e etiquetados previamente na balança comercial. O código de barras gerado segue o padrão de 13 dígitos EAN-13, estruturado sob a seguinte regra:

1. **Prefixo do Código**:
   - Geralmente o dígito `2`. Identifica que é uma etiqueta interna gerada por balança.
2. **Posição e Extração do Código do Produto**:
   - Mapeamento dinâmico configurável (ex: posição inicial `2` à final `7`). Permite extrair exatamente a identificação cadastrada no banco de dados.
3. **Tipo de Informação Embutida**:
   - **PESO**: O código de barras contém o peso líquido do produto (ex: `2.125` kg). O PDV calcula o valor total multiplicando pelo preço do cadastro.
   - **VALOR TOTAL**: O código contém o valor monetário total impresso (ex: `R$ 42,50`). O PDV extrai o valor e calcula a quantidade fracionada correspondente de forma retroativa.
4. **Casas Decimais**:
   - Parametrização de precisão matemática para divisão monetária (geralmente 2 ou 3 decimais).
