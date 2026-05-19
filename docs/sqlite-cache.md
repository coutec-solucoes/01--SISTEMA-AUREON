# Armazenamento em Cache SQLite Local do PDV

Para assegurar o funcionamento "Local-First" e tolerar quedas ou interrupções de internet de longa duração, cada terminal PDV Aureon utiliza um arquivo SQLite local exclusivo (`sqlite_local.db`).

## Estrutura do Banco Local

O SQLite local replica a estrutura cadastral do PostgreSQL necessária para a venda:
- `empresas_local`
- `moedas_local`
- `usuarios_local` e `perfis_local`
- `produtos_local` e `produtos_precos_local`
- `produtos_fiscal_local`
- `adicionais_local` e `produtos_adicionais_local`
- `configuracoes_pdv_local` e `regras_venda_local`
- `perifericos_local`

## Processamento em Lote Transacional

1. **Baixada de Dados**: O PDV recebe o pacote JSON da API.
2. **Execução Transacional (Tauri/Rust)**:
   - Uma transação do SQLite é aberta localmente.
   - Os dados são limpos e aplicados em lote (Bulk Upsert/Insert) para cada tabela correspondente.
   - Em caso de falha de qualquer inserção, o `ROLLBACK` é acionado e o estado original do banco local é 100% preservado.
   - Em caso de sucesso, a transação é confirmada (`COMMIT`) e o terminal envia a confirmação para a Retaguarda.
