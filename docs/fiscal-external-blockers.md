# Bloqueios Externos (Fase 19)

A Fase 19 mapeou 4 grandes gargalos ou pendências reais ("Pendências Externas") que exigem intervenção manual/física para liberar o sistema para testes de Homologação Governamental Real.

## 1. Bloco 2.2 Pendente (Schemas Oficiais)
- Os pacotes `.xsd` da PL_009_V4 (NF-e/NFC-e Brasil) e os schemas `.json` do SIFEN (Paraguai) não foram incluídos no código-fonte por motivos de licenciamento e segurança.
- **Ação Requerida:** Inserir os arquivos manualmente em `assets/schemas_fiscal/br/...` e `assets/schemas_fiscal/py/sifen/` e recalcular o `manifest.json`.

## 2. XMLDSig Real Pendente (xmlsec)
- O sistema de assinatura `xmlsec1` depende de bindings nativos (C/C++) que não rodam no Windows host sem altíssimo esforço (Msys2/MinGW).
- **Ação Requerida:** Utilizar a infra Docker/WSL (`scripts/check-fiscal-real.ps1`) dentro de ambiente Linux suportado para compilar a API local com a feature `fiscal_xmldsig_real`.

## 3. Certificado A1 Real
- Não há um certificado real homologado (PFX/P12) atrelado ao banco.
- **Ação Requerida:** Cadastrar e testar a validade da extração de chave no menu Fiscal Mestre (sem versionamento no git).

## 4. Homologação Governamental Real
- Até o momento, nenhum endpoint de Sefaz ou DNIT foi consumido com carga fiscal estruturada.
- **Produção Totalmente Bloqueada:** O sistema detecta URLs com `.gov.br/NFe...` em modo de produção e aborta o request preventivamente. Este escopo está adiado.
