# Impressão Não Fiscal no Aureon PDV

## Obrigação Legal e Operacional

Todos os documentos gerados pelo módulo de impressão do Aureon PDV são **documentos de controle interno operacional**. Eles **não têm validade fiscal** perante a legislação brasileira, paraguaia ou de qualquer jurisdição fiscal.

Todo documento impresso pela Fase 15 exibe obrigatoriamente:

```
*** DOCUMENTO NAO FISCAL ***
NAO E VALIDO COMO DOCUMENTO FISCAL
```

---

## Diferença entre Comprovante Operacional e Documento Fiscal

| Característica | Comprovante Operacional (Aureon) | Documento Fiscal |
|---|---|---|
| Validade fiscal | ❌ Nenhuma | ✅ Sim |
| Assinatura digital | ❌ Não | ✅ Sim (XML/chave) |
| Comunicação com SEFAZ/SIFEN | ❌ Não | ✅ Obrigatório |
| QR Code fiscal | ❌ Não | ✅ Sim |
| Série/Numeração fiscal | ❌ Não (apenas controle interno) | ✅ Regulamentada |
| Uso pelo consumidor para declaração | ❌ Não | ✅ Sim |
| Gerado pelo Aureon PDV Fase 15 | ✅ Sim | ❌ Fora do escopo |

---

## Exclusões Explícitas

A Fase 15 **não implementa** e **não deve implementar** nenhum dos seguintes:

### Brasil
- **NFC-e** — Nota Fiscal de Consumidor Eletrônica (SEFAZ)
- **NF-e** — Nota Fiscal Eletrônica (SEFAZ)
- **SAT** — Sistema Autenticador e Transmissor (CF-e SAT / MF-e)
- **DANFE** — Documento Auxiliar da NF-e
- **CF-e** — Cupom Fiscal Eletrônico

### Paraguai
- **SIFEN** — Sistema Integrado de Facturación Electrónica Nacional
- **Factura Electrónica**

### Geral
- QR Code fiscal
- Assinatura digital de documentos fiscais
- Comunicação com autoridades tributárias
- XML fiscal (envio, consulta ou cancelamento)
- Chave de acesso fiscal (44 dígitos)

---

## Regra de Ouro

> **Impressão é apenas saída documental. Não pode mexer em venda, caixa, estoque ou financeiro.**

Qualquer implementação futura que necessite de documentação fiscal deve ser tratada como um módulo separado, com infraestrutura dedicada de homologação fiscal, certificados digitais A1/A3 e testes com ambiente de homologação das autoridades competentes.
