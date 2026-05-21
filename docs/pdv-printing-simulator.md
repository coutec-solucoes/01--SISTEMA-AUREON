# Simulador de Impressão — Aureon PDV

## O que é o Simulador

O simulador de impressão é um destino virtual que **grava os dados de impressão em arquivo local** em vez de enviar para uma impressora física. É o destino padrão em todos os formulários do módulo de impressão.

Ele permite validar layouts de cupons, comprovantes e tickets sem necessidade de hardware físico.

---

## Localização dos Arquivos

```
C:/Aureon/print-sim/
```

O diretório é criado automaticamente na primeira impressão se não existir.

### Nomes dos Arquivos Gerados

Cada impressão gera **dois arquivos**:

| Arquivo | Conteúdo |
|---|---|
| `aureon_YYYY-MM-DD_HH-MM-SS.txt` | Texto puro legível — layout visual do cupom |
| `aureon_YYYY-MM-DD_HH-MM-SS.escpos.txt` | Bytes ESC/POS em hexadecimal — para debug técnico |

Exemplo:
```
C:/Aureon/print-sim/aureon_2026-05-20_22-10-00.txt
C:/Aureon/print-sim/aureon_2026-05-20_22-10-00.escpos.txt
```

---

## Caminho Retornado pela API

O command Tauri retorna o caminho do arquivo gerado no campo `caminho_arquivo_simulado` dentro de `dados` no `ResultadoImpressao`:

```json
{
  "sucesso": true,
  "mensagem": "Impressão concluída no simulador",
  "dados": {
    "caminho_arquivo_simulado": "C:/Aureon/print-sim/aureon_2026-05-20_22-10-00.txt"
  }
}
```

A UI Blazor exibe esse caminho automaticamente no alerta de sucesso.

---

## Caminho Customizável

O operador pode especificar um caminho diferente no campo `caminho_simulador` do `ImpressoraDestinoReq`:

```json
{
  "tipo_destino": "SIMULADOR",
  "caminho_simulador": "D:/prints/aureon_cupom.txt"
}
```

Se `caminho_simulador` estiver vazio ou nulo, o sistema usa o padrão `C:/Aureon/print-sim/`.

---

## Como Validar o Layout sem Impressora

1. Abra a tela `/reimpressoes` no Aureon PDV.
2. Verifique que o **Destino Atual** exibe `SIMULADOR`.
3. Clique em **Imprimir Teste** (aba Teste).
4. O alerta de sucesso exibirá o caminho do arquivo `.txt`.
5. Abra o arquivo em qualquer editor de texto (Bloco de Notas, VS Code, etc).
6. Verifique o layout: cabeçalho, itens, totais, rodapé não fiscal.

---

## Exemplo de Arquivo `.txt` Gerado (Cupom de Teste)

```
================================================
              AUREON PDV
================================================
           TESTE DE IMPRESSAO
         *** DOCUMENTO NAO FISCAL ***
------------------------------------------------
Data/Hora: 2026-05-20 22:10:00
Impressora: Aureon Print Padrao
------------------------------------------------
  Este e um cupom de teste de impressao.
  Nenhuma operacao foi realizada.
------------------------------------------------
     NAO E VALIDO COMO DOCUMENTO FISCAL
================================================
```

---

## Recomendações

- Use o simulador em **todo o desenvolvimento e testes de homologação**.
- Só mude para `TCP_IP` após validar completamente o layout no simulador.
- Mantenha o diretório `C:/Aureon/print-sim/` limpo periodicamente para evitar acúmulo de arquivos.
- O arquivo `.escpos.txt` é útil para debugar problemas com caracteres especiais ou formatação.
