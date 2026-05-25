# Infraestrutura XMLDSig Real (xmlsec)

O Aureon precisa assinar arquivos XML do Brasil (NF-e/NFC-e) segundo o padrão rigoroso W3C XMLDSig, especificamente envolvendo **Canonicalization (C14N)**, transformações enveloped, e geração do **cHashQR** (C-HASH).

Para realizar isso no Rust, a solução técnica encontrada foi fazer binding direto com a biblioteca C `xmlsec1`.

## O Desafio (Windows vs Linux)
A compilação cruzada dessa biblioteca em ambientes Windows/MSVC é notória por quebrar silenciosamente ou exigir ferramentas obsoletas. Assim, foi isolado o build dessa feature apenas para Linux (Docker/WSL). 

## As Features no Cargo
O backend (`aureon-api-local`) possui as flags controladas no `Cargo.toml`:
- `--features fiscal_real`: Troca o mock de criptografia por OpenSSL padrão.
- `--features fiscal_xmldsig_real`: Tenta carregar e linkar a crate `xmlsec`.

## Adoção Operacional (Bloco 8)
Foi fornecido um `Dockerfile` no repositório que instala todo o toolchain:
- `libssl-dev`
- `pkg-config`
- `libxml2-dev`
- `libxmlsec1-dev`
A verificação desta compilação fica atrelada ao laboratório Linux para garantir a fidedignidade da assinatura.
