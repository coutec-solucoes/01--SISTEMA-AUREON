# FASE 20 — BLOCO 5
## Ativação Offline de Licença Assinada no PDV

Este documento descreve como o PDV Aureon valida e aplica licenças de forma 100% offline usando assinaturas criptográficas Ed25519.

### Arquitetura de Segurança Offline
1. **Chave Privada Oculta**: Apenas a retaguarda/cloud possui a chave privada que gera a assinatura.
2. **Chave Pública no PDV**: O PDV recebe a chave pública e a assinatura junto com o JSON da licença.
3. **Verificação Determinística**: O PDV reconstrói o JSON canonicamente de forma idêntica à nuvem, gera o hash SHA-256 e valida a assinatura Ed25519 usando a biblioteca `ed25519-dalek`.

### Como Usar a Interface
No painel **Licenciamento Local & Assinatura Digital** do PDV:
1. Cole o **Payload JSON da Licença** (ex: `{"id":"...","empresa_id":"..."}`).
2. Cole a **Assinatura Criptográfica (Hex)** de 64 bytes gerada na retaguarda.
3. Cole a **Chave Pública da Licença (Hex)** de 32 bytes correspondente.
4. Clique em **Verificar Assinatura** para validar localmente antes de aplicar.
5. Clique em **Aplicar Licença Offline** para persistir na base de dados SQLite local e habilitar o terminal de forma definitiva offline.
