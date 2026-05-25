use axum::{
    routing::{get, post, put},
    Router, middleware,
};
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
use crate::routes::{
    health, diagnostico, empresa, auth,
    cadastros::{pessoas, grupos, produtos},
    configuracoes::operacionais,
    sync::{terminais, pacotes, publicacao, diagnostico as sync_diag},
    fiscal::{configuracoes as fiscal_config, dicionarios as fiscal_dic, regras as fiscal_reg, versoes as fiscal_ver, publicacao as fiscal_pub, certificados as fiscal_cert, assinatura as fiscal_ass}
};
use crate::middleware::auth_middleware;


/// Estado global compartilhado da API
#[derive(Clone)]
pub struct AppState {
    pub pool: Option<PgPool>,
}

/// Monta o roteador principal da API
pub fn criar_app(pool: Option<PgPool>) -> Router {
    let state = AppState { pool: pool.clone() };

    // Rotas protegidas (exigem token)
    let rotas_protegidas = Router::new()
        .route("/auth/logout", post(auth::logout))
        .route("/auth/me", get(auth::me))
        
        // Módulo de Segurança (Fase 3 - Bloco 3)
        .route("/seguranca/usuarios", get(crate::routes::seguranca::listar_usuarios).post(crate::routes::seguranca::criar_usuario))
        .route("/seguranca/usuarios/:id", put(crate::routes::seguranca::atualizar_usuario))
        .route("/seguranca/usuarios/:id/redefinir-senha", put(crate::routes::seguranca::redefinir_senha))
        
        .route("/seguranca/perfis", get(crate::routes::seguranca::listar_perfis).post(crate::routes::seguranca::criar_perfil))
        .route("/seguranca/perfis/:id", put(crate::routes::seguranca::atualizar_perfil))
        .route("/seguranca/perfis/:id/permissoes", get(crate::routes::seguranca::obter_permissoes).put(crate::routes::seguranca::salvar_permissoes))
        
        .route("/seguranca/supervisores", get(crate::routes::seguranca::listar_supervisores).post(crate::routes::seguranca::criar_supervisor))
        .route("/seguranca/supervisores/:id", put(crate::routes::seguranca::atualizar_supervisor))
        
        .route("/seguranca/autorizacoes", get(crate::routes::seguranca::listar_autorizacoes))
        .route("/seguranca/logs", get(crate::routes::seguranca::listar_logs))
        
        // Módulo de Cadastros (Fase 4 - Bloco 2)
        .route("/cadastros/pessoas", get(pessoas::listar_pessoas).post(pessoas::criar_pessoa))
        .route("/cadastros/pessoas/:id", get(pessoas::obter_pessoa).put(pessoas::atualizar_pessoa))
        .route("/cadastros/pessoas/:id/inativar", put(pessoas::inativar_pessoa))
        .route("/cadastros/clientes", get(pessoas::listar_clientes))
        .route("/cadastros/fornecedores", get(pessoas::listar_fornecedores))
        .route("/cadastros/funcionarios", get(pessoas::listar_funcionarios))
        .route("/cadastros/vendedores", get(pessoas::listar_vendedores))
        .route("/cadastros/entregadores", get(pessoas::listar_entregadores))
        .route("/cadastros/transportadoras", get(pessoas::listar_transportadoras))
        
        .route("/cadastros/produtos/grupos", get(grupos::listar_grupos).post(grupos::criar_grupo))
        .route("/cadastros/produtos/grupos/:id", put(grupos::atualizar_grupo))
        .route("/cadastros/produtos/subgrupos", get(grupos::listar_subgrupos).post(grupos::criar_subgrupo))
        .route("/cadastros/produtos/subgrupos/:id", put(grupos::atualizar_subgrupo))
        .route("/cadastros/produtos/marcas", get(grupos::listar_marcas).post(grupos::criar_marca))
        .route("/cadastros/produtos/marcas/:id", put(grupos::atualizar_marca))
        
        // Módulo de Produtos (Fase 4 - Bloco 3)
        .route("/cadastros/produtos", get(produtos::listar_produtos).post(produtos::criar_produto))
        .route("/cadastros/produtos/:id", get(produtos::obter_produto).put(produtos::atualizar_produto))
        .route("/cadastros/produtos/:id/inativar", put(produtos::inativar_produto))
        .route("/cadastros/produtos/:id/historico-precos", get(produtos::listar_historico_precos))
        
        .route("/cadastros/produtos/sabores-pizza", get(produtos::listar_sabores_pizza).post(produtos::criar_sabor_pizza))
        .route("/cadastros/produtos/sabores-pizza/:id", put(produtos::atualizar_sabor_pizza))
        
        .route("/cadastros/produtos/combos", get(produtos::listar_combos).post(produtos::criar_combo_item))
        
        .route("/cadastros/produtos/adicionais", get(produtos::listar_adicionais).post(produtos::criar_adicional))
        .route("/cadastros/produtos/adicionais/:id", put(produtos::atualizar_adicional))
        
        .route("/cadastros/produtos/locais-producao", get(produtos::listar_locais_producao).post(produtos::criar_local_producao))
        .route("/cadastros/produtos/locais-producao/:id", put(produtos::atualizar_local_producao))
        
        // Módulo de Configurações Operacionais (Fase 5 - Bloco 2)
        .route("/configuracoes/operacionais/pdv", get(operacionais::obter_configuracao_pdv).post(operacionais::salvar_configuracao_pdv).put(operacionais::salvar_configuracao_pdv))
        
        .route("/configuracoes/operacionais/terminais", get(operacionais::listar_terminais).post(operacionais::criar_terminal))
        .route("/configuracoes/operacionais/terminais/:id", put(operacionais::atualizar_terminal))
        .route("/configuracoes/operacionais/terminais/:id/inativar", put(operacionais::inativar_terminal))
        .route("/configuracoes/operacionais/terminais/:id/autorizar", put(operacionais::autorizar_terminal))
        
        .route("/configuracoes/operacionais/registradoras", get(operacionais::listar_registradoras).post(operacionais::criar_registradora))
        .route("/configuracoes/operacionais/registradoras/:id", put(operacionais::atualizar_registradora))
        .route("/configuracoes/operacionais/registradoras/:id/inativar", put(operacionais::inativar_registradora))
        
        .route("/configuracoes/operacionais/mesas/configuracao", get(operacionais::obter_mesas_configuracao).post(operacionais::salvar_mesas_configuracao).put(operacionais::salvar_mesas_configuracao))
        .route("/configuracoes/operacionais/mesas", get(operacionais::listar_mesas).post(operacionais::criar_mesa))
        .route("/configuracoes/operacionais/mesas/:id", put(operacionais::atualizar_mesa))
        
        .route("/configuracoes/operacionais/comandas/configuracao", get(operacionais::obter_comandas_configuracao).post(operacionais::salvar_comandas_configuracao).put(operacionais::salvar_comandas_configuracao))
        .route("/configuracoes/operacionais/comandas", get(operacionais::listar_comandas).post(operacionais::criar_comanda))
        .route("/configuracoes/operacionais/comandas/:id", put(operacionais::atualizar_comanda))
        
        .route("/configuracoes/operacionais/pre-vendas", get(operacionais::obter_prevendas).post(operacionais::salvar_prevendas).put(operacionais::salvar_prevendas))
        .route("/configuracoes/operacionais/orcamentos", get(operacionais::obter_orcamentos).post(operacionais::salvar_orcamentos).put(operacionais::salvar_orcamentos))
        
        .route("/configuracoes/operacionais/regras-venda", get(operacionais::obter_regras_venda).post(operacionais::salvar_regras_venda).put(operacionais::salvar_regras_venda))
        
        .route("/configuracoes/operacionais/series-numeracao", get(operacionais::listar_series_numeracao).post(operacionais::criar_serie_numeracao))
        .route("/configuracoes/operacionais/series-numeracao/:id", put(operacionais::atualizar_serie_numeracao))
        
        .route("/configuracoes/operacionais/impressoras", get(operacionais::listar_impressoras).post(operacionais::criar_impressora))
        .route("/configuracoes/operacionais/impressoras/:id", put(operacionais::atualizar_impressora))
        
        .route("/configuracoes/operacionais/setores-producao", get(operacionais::listar_setores_producao).post(operacionais::criar_setor_producao))
        .route("/configuracoes/operacionais/setores-producao/:id", put(operacionais::atualizar_setor_producao))
        
        .route("/configuracoes/operacionais/balancas", get(operacionais::listar_balancas).post(operacionais::criar_balanca))
        .route("/configuracoes/operacionais/balancas/:id", put(operacionais::atualizar_balanca))
        
        .route("/configuracoes/operacionais/etiquetas-balanca", get(operacionais::listar_etiquetas_balanca).post(operacionais::criar_etiqueta_balanca))
        .route("/configuracoes/operacionais/etiquetas-balanca/:id", put(operacionais::atualizar_etiqueta_balanca))
        
        .route("/configuracoes/operacionais/perifericos", get(operacionais::listar_perifericos).post(operacionais::criar_periferico))
        .route("/configuracoes/operacionais/perifericos/:id", put(operacionais::atualizar_periferico))
        
        .route("/configuracoes/operacionais/senhas-chamadas", get(operacionais::obter_senhas_chamadas).post(operacionais::salvar_senhas_chamadas).put(operacionais::salvar_senhas_chamadas))
        
        // Módulo de Sync Fase 6 (Retaguarda)
        .route("/sync/terminais/status", get(terminais::status_geral_terminais))
        .route("/sync/terminais/:id/diagnostico", get(terminais::diagnostico_terminal))
        .route("/sync/versoes", get(pacotes::listar_versoes))
        .route("/sync/publicar", post(publicacao::publicar))
        .route("/sync/publicacoes", get(publicacao::listar_publicacoes))
        .route("/sync/publicacoes/:id", get(publicacao::obter_publicacao))
        .route("/sync/publicacoes/:id/reprocessar", post(publicacao::reprocessar_publicacao))
        .route("/sync/diagnostico", get(sync_diag::diagnostico_geral))
        .route("/sync/logs", get(sync_diag::listar_logs))
        
        // Módulo Retaguarda Fiscal Mestre (Fase 17 - Bloco 2)
        // Configurações Fiscais
        .route("/fiscal/configuracoes", get(fiscal_config::obter_configuracoes).post(fiscal_config::criar_configuracao))
        .route("/fiscal/configuracoes/:id", put(fiscal_config::atualizar_configuracao))
        
        // Certificados (Fase 18 - Bloco 1)
        .route("/fiscal/certificado/validar", post(fiscal_cert::validar_certificado))
        .route("/fiscal/certificado/status", get(fiscal_cert::status_certificado))

        // Assinatura Técnica XML (Fase 18 - Bloco 2)
        .route("/fiscal/assinatura/testar", post(fiscal_ass::testar_assinatura))
        .route("/fiscal/assinatura/assinar-preview", post(fiscal_ass::assinar_preview))
        .route("/fiscal/assinatura/verificar-preview", post(fiscal_ass::verificar_preview))

        // Dicionários
        .route("/fiscal/dicionarios/ncm", get(fiscal_dic::obter_ncms).post(fiscal_dic::criar_ncm))
        .route("/fiscal/dicionarios/ncm/:id", put(fiscal_dic::atualizar_ncm))
        .route("/fiscal/dicionarios/ncm/:id/inativar", put(fiscal_dic::inativar_ncm))
        
        .route("/fiscal/dicionarios/cfop", get(fiscal_dic::obter_cfops).post(fiscal_dic::criar_cfop))
        .route("/fiscal/dicionarios/cfop/:id", put(fiscal_dic::atualizar_cfop))
        .route("/fiscal/dicionarios/cfop/:id/inativar", put(fiscal_dic::inativar_cfop))
        
        .route("/fiscal/dicionarios/cst-csosn", get(fiscal_dic::obter_cst_csosns).post(fiscal_dic::criar_cst_csosn))
        .route("/fiscal/dicionarios/cst-csosn/:id", put(fiscal_dic::atualizar_cst_csosn))
        .route("/fiscal/dicionarios/cst-csosn/:id/inativar", put(fiscal_dic::inativar_cst_csosn))
        
        .route("/fiscal/dicionarios/iva", get(fiscal_dic::obter_ivas).post(fiscal_dic::criar_iva))
        .route("/fiscal/dicionarios/iva/:id", put(fiscal_dic::atualizar_iva))
        .route("/fiscal/dicionarios/iva/:id/inativar", put(fiscal_dic::inativar_iva))
        
        // Regras Tributárias
        .route("/fiscal/regras", get(fiscal_reg::obter_regras).post(fiscal_reg::criar_regra))
        .route("/fiscal/regras/:id", put(fiscal_reg::atualizar_regra))
        .route("/fiscal/regras/:id/inativar", put(fiscal_reg::inativar_regra))
        
        // Versionamento e Auditoria
        .route("/fiscal/versoes", get(fiscal_ver::obter_versoes))
        .route("/fiscal/versoes/rascunho", post(fiscal_ver::criar_versao_rascunho))
        .route("/fiscal/versoes/:id/cancelar", put(fiscal_ver::cancelar_versao))
        .route("/fiscal/versoes/:id/itens", get(fiscal_ver::listar_itens_versao))
        .route("/fiscal/auditoria", get(fiscal_ver::obter_auditoria))
        
        // Publicação Fiscal Versionada (Bloco 3)
        .route("/fiscal/versoes/:id/publicar", post(fiscal_pub::publicar_versao))
        .route("/fiscal/versoes/:id/reprocessar", post(fiscal_pub::reprocessar_versao))
        .route("/fiscal/versoes/:id/payload", get(fiscal_pub::obter_payload_versao))
        .route("/fiscal/publicacoes", get(fiscal_pub::listar_publicacoes_fiscais))
        .route("/fiscal/publicacoes/:id", get(fiscal_pub::obter_publicacao_fiscal))
        
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));


    Router::new()
        .route("/health",            get(health::handler_health))
        .route("/diagnostico/basico", get({
            let pool = pool.clone();
            move || diagnostico::handler_diagnostico(pool)
        }))
        
        // Autenticação (públicas)
        .route("/auth/login", post(auth::login))
        .route("/auth/setup", post(auth::setup_admin))
        
        // Endpoints do módulo Empresa (Fase 2)
        .route("/empresa/configuracao", get(empresa::obter_configuracao).post(empresa::salvar_configuracao).put(empresa::salvar_configuracao))
        .route("/empresa/moedas", get(empresa::obter_moedas).put(empresa::salvar_moedas))
        .route("/empresa/cotacoes", get(empresa::obter_cotacoes).post(empresa::criar_cotacao))
        .route("/empresa/cotacoes/:id/cancelar", put(empresa::cancelar_cotacao))
        .route("/empresa/fiscal", get(empresa::obter_fiscal).put(empresa::salvar_fiscal))
        .route("/empresa/parametros-operacionais", get(empresa::obter_parametros).put(empresa::salvar_parametros))
        .route("/empresa/auditoria", get(empresa::obter_auditoria))
        .route("/empresa/status-configuracao", get(empresa::status_configuracao))
        
        // Módulo de Sync Fase 6 (Endpoints do PDV)
        .route("/sync/terminais/registrar", post(terminais::registrar_terminal))
        .route("/sync/terminais/:codigo_terminal/status", get(terminais::status_terminal))
        .route("/sync/terminais/:id/confirmar-autorizacao", put(terminais::confirmar_autorizacao))
        .route("/sync/primeira-sincronizacao", post(pacotes::primeira_sincronizacao))
        .route("/sync/confirmar-aplicacao", post(pacotes::confirmar_aplicacao))
        .route("/sync/eventos-pendentes/:terminal_id", get(publicacao::eventos_pendentes))
        .route("/sync/eventos-confirmar", post(publicacao::confirmar_evento))
        
        .merge(rotas_protegidas)
        .with_state(state)
        .layer(CorsLayer::permissive())
}
