# Configuração e Execução do XMLDSig Fiscal

A assinatura digital definitiva da Nota Fiscal Eletrônica (NF-e/NFC-e) exige canonicalização C14N e Enveloped Signature. Como isso depende de bibliotecas nativas C (libxmlsec1, libxml2, libssl), o Aureon foi estruturado com uma arquitetura de adapters e features opcionais em Rust para não quebrar o desenvolvimento local no Windows.

## Estratégia de Build

O crate `aureon-api-local` possui três níveis de assinatura, gerenciados por Cargo features:

1. **Build Padrão (Sem features - Windows/Local)**
   ```bash
   cargo check -p aureon-api-local
   ```
   *Comportamento:* Apenas MOCK. Não exige openssl ou xmlsec. Compilação rápida e à prova de falhas no Windows. Rota de assinatura técnica retorna mock indisponível.

2. **Feature `fiscal_real` (OpenSSL nativo - Validação de Certificado)**
   ```bash
   cargo check -p aureon-api-local --features fiscal_real
   ```
   *Comportamento:* Usa OpenSSL (`vendored` ou nativo do sistema) para leitura do PFX/A1 e hash/assinatura básica. Não aplica canonicalização C14N real, apenas validação técnica do certificado.

3. **Feature `fiscal_xmldsig_real` (XMLSec nativo - Assinatura SEFAZ)**
   ```bash
   cargo check -p aureon-api-local --features fiscal_real,fiscal_xmldsig_real
   ```
   *Comportamento:* Habilita o backend de assinatura fiscal completo com a crate `xmlsec`. Implementa a assinatura envelopada exigida pelos esquemas PL_009_V4.

## Ambiente de Execução Real

Para emitir documentos e fazer assinatura válida:
- É preferível rodar a aplicação em um ambiente Linux (Docker, WSL ou Servidor CI) com os pacotes instalados:
  ```bash
  apt-get install libxmlsec1-dev libxml2-dev pkg-config libssl-dev
  ```

## Restrições Atuais (Fase 19 - Bloco 1)

* **Homologação Apenas:** A assinatura funciona estritamente para `<tpAmb>2</tpAmb>`.
* **Sem Transmissão:** O XML é assinado e envelopado, mas NÃO é transmitido aos webservices da SEFAZ.
* **Segurança:** O certificado A1 e sua senha transitam em memória para a assinatura e nunca são retornados, logados ou persistidos nos endpoints de preview técnico.

## Diagnóstico

Para verificar a disponibilidade do módulo em runtime:
```bash
GET /diagnostico/basico
```
*Retorno esperado (`dados.fiscal`):*
```json
{
  "fiscal_real": true,
  "fiscal_xmldsig_real": false,
  "backend_assinatura": "openssl_tecnico"
}
```
