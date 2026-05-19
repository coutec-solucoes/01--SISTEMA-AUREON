# PDV Core (Núcleo Local)

O Núcleo Local do PDV Aureon é uma aplicação construída em Rust (Tauri) utilizando banco de dados SQLite. 
A premissa principal é a autonomia offline: o terminal deve poder vender, abrir e fechar caixa indefinidamente sem depender de internet.

## Pilares de Engenharia

1. **State Management:** Conexões com o SQLite são mantidas via `tauri::State`, protegidas por Mutex (`Arc<Mutex<Connection>>`) para evitar *database locking* concorrente originado do Blazor.
2. **Matemática Inteira:** A regra de ouro é nunca utilizar `f64` ou `f32` para dinheiro no Backend, Frontend ou Banco de Dados. Apenas Minor Units (centavos) são suportados para evitar as perigosas imprecisões de ponto flutuante, comuns na computação financeira.
3. **Idempotência por Transação:** Operações sensíveis ocorrem via `conn.transaction()`. Se o banco falhar ou a energia cair entre o pagamento e a finalização, haverá rollback nativo.
4. **Desacoplamento Assíncrono:** Nenhuma ação local tenta comunicação imediata de rede. Tudo é gravado na tabela `sync_outbox` para ser processado no background (Fases seguintes).
