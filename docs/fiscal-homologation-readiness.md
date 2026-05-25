# Checklist de Prontidão Técnica (Readiness)

O painel de prontidão (criado no Bloco 9) serve como um bloqueio condicional antes de liberar qualquer tentativa de transmissão para as SEFAZ ou DNIT no ambiente do Aureon.

## Critérios de Prontidão (Obrigatórios)
Para atingir `pronto_para_homologacao = true`, os seguintes itens técnicos precisam retornar `OK` via API:
1. **CERTIFICADO_A1_CONFIGURADO**: O sistema tem registro ativo do path/alias do PFX em banco.
2. **FISCAL_REAL_FEATURE**: Compilação ativada (permite rustls/native-tls real).
3. **XMLDSIG_REAL_FEATURE / XMLSEC_RUNTIME**: Compilação ativada e bindings C++ respondendo (permite C14N XML e RSA-SHA256 padrão W3C).
4. **SCHEMAS_NFE_NFCE_PRESENTES / SCHEMAS_SIFEN_PRESENTES**: Arquivos `XSD` e `JSON Schema` existem nas pastas `assets/schemas_fiscal/...`.
5. **MANIFEST_SCHEMAS_VALIDO**: Todos os schemas listados estão com SHA-256 íntegro no `manifest.json`.
6. **ENDPOINTS_HOMOLOGACAO_REGISTRADOS**: Banco contendo as URLs da SVRS e e-Kuatia pré-cadastradas para o ambiente = 2 (Homologação).
7. **BLOQUEIO_PRODUCAO_ATIVO**: Regras estáticas mapeadas bloqueando `tpAmb=1`.

Se a resposta da API em `/fiscal/homologacao/prontidao` acusar falha, o botão ou fluxo que consumiria a Sefaz deve permanecer bloqueado.
