# Manifest e Registro de Schemas Fiscais

O projeto Aureon armazena e valida os arquivos oficiais base que descrevem as regras estruturais e de dados das notas fiscais:
- **Brasil:** `XSD` (XML Schema Definition), ex: PL_009_V4.
- **Paraguai:** `JSON Schema` (e-Kuatia).

Para garantir que o cliente/PDV rodando offline possua as mesmas regras que o governo homologou, a Fase 19 preparou um `manifest.json`.

## Regras de Funcionamento
Os arquivos `.xsd` e `.json` devem ser incluídos na raiz `assets/schemas_fiscal/br/` e `assets/schemas_fiscal/py/`.
Nenhuma operação de validação oficial XML deverá ocorrer até que o motor verifique os Hashes `SHA-256` informados no manifesto.
**(Pendência Técnica: no momento, estão fisicamente ausentes por questões de IP e segurança de payload).**
