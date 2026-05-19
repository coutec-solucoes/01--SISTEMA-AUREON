# Registro e Autorização de Terminais PDV

Para que um terminal PDV possa consumir dados ou enviar vendas para o servidor central, ele precisa passar por um protocolo rigoroso de registro e autorização.

## Protocolo de Registro

1. **Primeira Conexão (Registro)**:
   - O PDV gera um identificador único de máquina e envia uma requisição `POST /sync/registrar` com o código identificador, o nome do terminal e o tipo de dispositivo.
   - A API local cria o registro com status `ativo = true` e `autorizado = false` na tabela `terminais_pdv`, gerando uma chave criptográfica secreta exclusiva (`chave_terminal`).
   
2. **Autorização Administrativa**:
   - Um administrador da Retaguarda revisa a solicitação do terminal e o autoriza (marca `autorizado = true` no painel web da retaguarda).

3. **Validação nas Requisições**:
   - Toda chamada de sincronização e envio de dados feita pelo PDV exige o cabeçalho:
     `Authorization: Bearer <chave_terminal>`
   - A API valida a chave, o status ativo e a autorização antes de liberar qualquer dado.
