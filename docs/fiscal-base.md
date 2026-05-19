# 🌍 Fiscal Base — Guia de Referência Técnica

Este documento fornece as diretrizes e mapeamentos fiscais para o funcionamento do Aureon nos cenários tributários do Brasil (NFC-e/NF-e) e Paraguai (SIFEN).

---

## 1. Mapeamento Tributário Brasil

*   **Regime de Tributação (CRT)**:
    *   `1 - Simples Nacional` (Padrão para a maioria dos pequenos comerciantes da fronteira).
    *   `2 - Simples Nacional - Excesso de Sublimite`.
    *   `3 - Regime Normal` (Lucro Presumido ou Lucro Real).
*   **Documentos Habilitados**:
    *   **NFC-e (Nota Fiscal de Consumidor Eletrônica)**: Impressa via bobina térmica de 80mm no caixa PDV.
    *   **NF-e (Nota Fiscal Eletrônica)**: Usada para emissão de notas de devolução, transferência de mercadorias ou vendas corporativas (B2B).
    *   **NFS-e (Nota Fiscal de Serviços Eletrônica)**: Habilitada se houver cobrança de taxas de entrega ou serviços integrados.
*   **Provedor Fiscal**: Aureon Emissor Nível 1 local ou Focus NFe.

---

## 2. Mapeamento Tributário Paraguai

*   **Regime SET**:
    *   `REGIME_GERAL`: Enquadramento comum (IVA de 10% geral ou 5% reduzido).
    *   `IRE_SIMPLIFICADO`: Imposto de Renda Empresarial Simplificado.
    *   `RESIMPLE`: Imposto simplificado para microempresas com tributação mensal de valor fixo.
*   **Documentos Habilitados**:
    *   **Factura Electrónica (SIFEN)**: Nota eletrônica oficial integrada via chave digital XML à Subsecretaria de Estado de Tributação do Paraguai.
*   **Provedor Fiscal**: Sifen Direct (comunicação Aureon API).
