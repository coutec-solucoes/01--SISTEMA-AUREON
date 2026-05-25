# Ambiente Técnico para Fiscal Real (Docker/WSL)

## Motivo do Docker/WSL
O sistema Aureon utiliza bibliotecas de criptografia e assinatura digital pesadas para suportar a emissão real de notas fiscais, como a geração do QR Code com C-HASH, assinatura XMLDSig (com referências específicas exigidas pela SEFAZ), e comunicação mTLS (Mutual TLS). 
A dependência nativa fundamental para assinar o XML da nota é a **xmlsec1** (através da crate `xmlsec`). Essa biblioteca exige um ecossistema C/C++ farto (libxml2, openssl, perl, pkg-config) que é extremamente instável para compilar no Windows de forma nativa. 
Portanto, a solução técnica segura e reproduzível foi isolar o **build e o runtime fiscal real** em um ambiente Linux via Docker (ou WSL).

## Dependências Nativas (Linux - Debian/Ubuntu)
Dentro do Dockerfile, instalamos:
- `pkg-config` e `build-essential` (Toolchain de compilação C/C++)
- `libssl-dev` e `openssl` (Comunicação mTLS e base de criptografia)
- `perl` (Requisito da crate openssl-sys)
- `libxml2` e `libxml2-dev` (Parser C para XML)
- `libxmlsec1`, `libxmlsec1-dev` e `xmlsec1` (Criptografia XMLDSig Real)

## Comandos de Build e Teste

Se você estiver em um ambiente **Windows**, você pode chamar o script em PowerShell para automatizar o Docker Compose:
```powershell
./scripts/check-fiscal-real.ps1
```

Se estiver nativo no **Linux / WSL**, ou via bash no Git Bash:
```bash
./scripts/check-fiscal-real.sh
```

Os scripts testam progressivamente:
1. `cargo check -p aureon-api-local` (Valida a compilação padrão sem quebras no mock)
2. `cargo check -p aureon-api-local --features fiscal_real` (Valida mTLS, OpenSSL e validação estrutural restrita)
3. `cargo check -p aureon-api-local --features fiscal_real,fiscal_xmldsig_real` (Valida a compilação do binding `xmlsec`, o mais crítico)

## Como Identificar Ativação das Features

### fiscal_real
Ativa a comunicação via `native-tls` utilizando as chaves privadas extraídas dos certificados PFX. É usado principalmente no diagnóstico mTLS e na assinatura preliminar. A flag `cfg(feature = "fiscal_real")` vai ignorar os mocks baseados em base64/sha256 padrão e forçar OpenSSL.

### fiscal_xmldsig_real
Ativa as structs do `xmlsec`. Caso a compilação falhe, o sistema fallback internamente (se no Windows) ou retorna erro em runtime caso o binário não contenha a assinatura estrita, mantendo como pendência na infra até a correção.

## Limitações e Segurança

1. **Volume de Target:** O `docker-compose.yml` mapeia o `/app/target` para um volume anônimo. Isso previne corrupção de compilação entre o host Windows (`x86_64-pc-windows-msvc`) e o contêiner (`x86_64-unknown-linux-gnu`).
2. **Senha e Certificado:** Nunca commitar arquivos `.pfx` ou senhas abertas. Variáveis recomendadas de ambiente (`FISCAL_CERT_PATH`, `FISCAL_CERT_PASSWORD`) não devem estar em repositórios abertos.

## AVISO: Não é Transmissão/Autorização
A execução do Docker para compilar e testar os pacotes **não transmite documento para SEFAZ/SIFEN, não gera protocolo e não gera DANFE legal**.
Este é exclusivamente um ambiente laboratorial de validação do binding C++ (xmlsec) e OpenSSL no Rust.
