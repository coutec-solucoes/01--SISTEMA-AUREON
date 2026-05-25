# Conectividade mTLS e Cliente Fiscal (Diagnóstico)

A Fase 19 integrou o Cliente Fiscal HTTP baseado no `reqwest` usando os certificados via OpenSSL (native-tls / rustls-tls). O intuito inicial não é enviar a nota, mas **garantir que as conexões de rede chegam até a Sefaz e retornam**.

## Bloqueios Ativos
Para evitar emissões reais indevidas, o Cliente intercepta e processa as requests de teste, impedindo:
- Utilização de `tpAmb=1` (Produção).
- Envio de XML assinados de fato (apenas requests `HEAD` ou `GET`).
- Bloqueia payloads sem certificado.

O objetivo do Bloco 4 foi fornecer o `/fiscal/homologacao/testar-conectividade`, revelando MS (latência), Validação TLS e o Response Status HTTP da SEFAZ, garantindo que o laboratório não tem restrições de Firewall (ex: IPs bloqueados).
