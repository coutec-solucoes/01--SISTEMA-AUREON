# Histórico de Homologação Fiscal

Todos os eventos técnicos de diagnóstico ou geração de previews operados na Fase 18 e Fase 19 são rastreados assincronamente. O objetivo é criar logs de auditoria técnica local (para isolar erros e debugar certificados ou endpoints do governo que estejam inativos).

## Mecanismo (Blocos 5 e 6)
A tabela `fiscal_homologacao_historico` armazena:
- Modelo e país (`NFC-e`, `SIFEN`).
- Tipo de Evento (ex: `TESTE_CONECTIVIDADE`, `PREVIEW_ASSINATURA`, `PRODUCAO_BLOQUEADA`).
- Payload truncado em 4KB em formato JSON (excluindo PFX/senhas) de maneira assíncrona (`tokio::spawn`).

Não sendo de ordem contábil ou fiscal real, esse registro ajuda primariamente a equipe de suporte e infra a saber o que falhou durante os testes.
