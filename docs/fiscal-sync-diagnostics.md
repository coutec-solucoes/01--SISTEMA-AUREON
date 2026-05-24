# Diagnóstico de Sincronização Fiscal no PDV

## Visão Geral

A tela **FiscalSyncPdv** (`/fiscal/sync`) permite ao operador técnico ou supervisor verificar o estado atual da sincronização fiscal local do PDV.

> ⚠️ "O PDV apenas consome regras fiscais publicadas pela Retaguarda. Esta tela não emite, não autoriza e não transmite documentos fiscais."

---

## Acesso

- **Rota:** `/fiscal/sync`
- **Link no menu:** "Sync Fiscal" (cor diferenciada: `#ce93d8`)
- **Perfil recomendado:** Técnico / Supervisor *(controle de perfil ainda não implementado — fase futura)*

---

## Seção 1: Status da Versão Fiscal Atual

Cards exibem:

| Card | Campo | Fonte |
|------|-------|-------|
| Versão Atual | `versao_atual` | `fiscal_versoes_aplicadas_cache` |
| Status | `status` | APLICADO/ERRO/PENDENTE |
| Total Registros | `total_registros` | Quantidade aplicada |
| Aplicado Em | `aplicado_em` | Timestamp da aplicação |

Linha adicional com:
- **Pacote ID** (UUID completo em `<code>`)
- **Payload Hash** (hash determinístico em `<code>`)

Se houver `ultimo_erro`, exibido em alerta vermelho.

**Command:** `obter_status_versao_fiscal_local`

---

## Seção 2: Logs de Sincronização Fiscal

Tabela com os últimos 100 eventos, com badges coloridos:

| Badge | Evento |
|-------|--------|
| 🟢 `bg-success` | `FISCAL_PACOTE_APLICADO` |
| 🔴 `bg-danger` | `FISCAL_PACOTE_ERRO` |
| 🟡 `bg-warning` | `FISCAL_PACOTE_IGNORADO_IDEMPOTENTE` |
| ⚫ `bg-secondary` | Outros (RECEBIDO, VALIDADO) |

**Command:** `listar_logs_sync_fiscal`

---

## Seção 3: Validação Manual de Payload

Permite colar um JSON de pacote fiscal e verificar se sua estrutura é válida **sem aplicar**.

Campos:
- `pacote_id` (opcional)
- `versao`
- `payload_hash`
- `idempotency_key` (opcional)
- `payload_json` (textarea grande)

Resultado exibido com ✅ ou ❌.

**Command:** `validar_pacote_fiscal_local`

> Esta operação é **somente leitura** — não persiste nada no SQLite.

---

## Seção 4: Aplicação Manual (Modo Diagnóstico)

Permite aplicar um pacote fiscal diretamente na tela, para uso em:
- Ambiente de homologação
- Diagnóstico técnico
- Recuperação de divergência

**Fluxo:**
1. Preencher o payload na seção de Validação
2. Clicar "Aplicar Pacote Fiscal (Diagnóstico)"
3. Modal de confirmação exibe lista detalhada do que será feito e o que NÃO será feito
4. Confirmar → chama `aplicar_pacote_fiscal`
5. Tela recarrega status e logs automaticamente

**Command:** `aplicar_pacote_fiscal`

---

## O que a Tela NÃO FAZ

- ❌ Não emite NF-e, NFC-e, SAT ou SIFEN
- ❌ Não gera XML assinado
- ❌ Não gera DANFE, KuDE ou QR Fiscal
- ❌ Não transmite para SEFAZ, DNIT ou SIFEN
- ❌ Não altera venda, caixa, estoque, financeiro, compras, delivery ou gourmet
- ❌ Não usa certificado digital operacional

---

## Pendências e Melhorias Futuras

1. **Controle de perfil:** Restringir acesso ao command `aplicar_pacote_fiscal` apenas a perfis técnicos/supervisores.
2. **Aplicação automática via Sync:** O PDV deve aplicar pacotes automaticamente quando receber via endpoint de sync — sem intervenção manual.
3. **Notificação visual de nova versão disponível:** Badge no menu quando há pacote fiscal novo pendente de aplicação.
