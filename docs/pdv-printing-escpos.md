# ESC/POS no Aureon PDV

## Visão Geral

O Aureon PDV usa um builder ESC/POS próprio implementado em Rust puro, sem bibliotecas externas. Ele produz um `Vec<u8>` de bytes que são enviados diretamente para a impressora (TCP/IP, spooler ou arquivo).

---

## Struct `EscPosBuilder`

```rust
// Localizado em: commands_impressao.rs
struct EscPosBuilder {
    buffer: Vec<u8>,
    largura: usize,
}
```

O builder é instanciado com a largura configurada pelo operador:

```rust
let mut doc = EscPosBuilder::new(req.destino.largura_colunas as usize);
```

---

## Comandos Básicos Usados

| Método | Bytes ESC/POS | Função |
|---|---|---|
| `init()` | `ESC @` (0x1B 0x40) | Inicializa a impressora |
| `bold(true)` | `ESC E 1` | Ativa negrito |
| `bold(false)` | `ESC E 0` | Desativa negrito |
| `center()` | `ESC a 1` | Alinha ao centro |
| `left()` | `ESC a 0` | Alinha à esquerda |
| `feed(n)` | `ESC d n` | Avança N linhas |
| `cut()` | `ESC i` (0x1B 0x69) | Corta o papel |
| `open_drawer()` | `ESC p 0 25 250` | Pulso de gaveta |
| `line(texto)` | — | Adiciona linha de texto com `\n` |
| `separator()` | — | Linha de traços (`---...---`) |
| `two_col(l, r)` | — | Linha com texto esquerdo e valor direito |

---

## Larguras Suportadas

| Colunas | Bobina típica | Uso |
|---|---|---|
| 32 | 58 mm | Impressoras compactas / balcão |
| 42 | Intermediária | Casos especiais |
| 48 | 80 mm | Padrão recomendado para o Aureon PDV |

A separação de colunas (`two_col`) e o `separator()` se adaptam automaticamente à largura configurada.

---

## Corte de Papel

Controlado pelo campo `cortar_papel: bool` no `ImpressoraDestinoReq`.

```rust
if req.destino.cortar_papel {
    doc.cut();
}
```

Impressoras sem guilhotina ignoram o comando fisicamente sem erro.

---

## Pulso de Gaveta

O comando de gaveta (`open_drawer`) é enviado via o mesmo canal da impressora. A impressora deve ter a gaveta conectada no conector RJ-11/RJ-12 traseiro.

```rust
if req.destino.abrir_gaveta {
    doc.open_drawer();
}
```

O command `abrir_gaveta_dinheiro` usa exclusivamente este mecanismo.

---

## Limitações Conhecidas

1. **Sem suporte a QR Code** via ESC/POS nativo nesta versão.
2. **Sem imagens/logos** — apenas texto ASCII.
3. **Windows RAW não implementado** — stub presente, retorna erro controlado.
4. **Timeout TCP/IP** fixo em 3 segundos — pode bloquear thread.
5. **Sem retry automático** — falhas de envio retornam erro imediatamente.
6. **Charset**: apenas caracteres ASCII e alguns caracteres latinos com mapeamento manual. Caracteres fora do range podem ser substituídos.
