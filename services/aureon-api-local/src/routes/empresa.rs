use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use crate::app::AppState;
use aureon_core::{AureonError, RespostaBase};

// ================================================================
// DTOs & Modelos
// ================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmpresaConfiguracaoCompletaDto {
    pub id: Uuid,
    pub codigo: String,
    pub nome_fantasia: String,
    pub razao_social: String,
    pub tipo_pessoa: String,
    pub status_empresa: String,
    pub observacoes: Option<String>,
    pub criado_em: DateTime<Utc>,
    
    // País / Fiscal
    pub pais: String,
    pub ambiente_fiscal: String,
    
    // Idioma
    pub idioma_padrao: String,
    pub idioma_comprovantes: String,
    pub permitir_idioma_usuario: bool,
    pub idioma_autoatendimento: String,
    
    // Identificação
    pub cnpj: Option<String>,
    pub inscricao_estadual: Option<String>,
    pub inscricao_municipal: Option<String>,
    pub cpf: Option<String>,
    pub rg: Option<String>,
    pub ruc: Option<String>,
    pub ci: Option<String>,
    
    // Contato
    pub telefone_principal: String,
    pub whatsapp: Option<String>,
    pub telefone_secundario: Option<String>,
    pub email: Option<String>,
    pub responsavel: String,
    pub site: Option<String>,
    
    // Endereço
    pub estado: String,
    pub cidade: String,
    pub bairro: Option<String>,
    pub logradouro: String,
    pub numero: Option<String>,
    pub complemento: Option<String>,
    pub cep: Option<String>,
    pub referencia: Option<String>,
    
    // Logo
    pub caminho_logo: Option<String>,
    pub usar_comprovantes: bool,
    pub usar_relatorios: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoedaConfigDto {
    pub moeda_id: Uuid,
    pub codigo: String,
    pub nome: String,
    pub simbolo: String,
    pub tipo_moeda: String,
    pub ordem_exibicao: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmpresaMoedasCompletaDto {
    pub multimoeda_ativa: bool,
    pub permitir_pagamento_multiplo: bool,
    pub permitir_troco_diferente: bool,
    pub moedas: Vec<MoedaConfigDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CotacaoCadastroDto {
    pub moeda_origem_codigo: String,
    pub moeda_destino_codigo: String,
    pub taxa_direta: Decimal,
    pub observacao: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CotacaoExibicaoDto {
    pub id: Uuid,
    pub data_cotacao: NaiveDate,
    pub moeda_origem_id: Uuid,
    pub moeda_origem_codigo: String,
    pub moeda_destino_id: Uuid,
    pub moeda_destino_codigo: String,
    pub taxa_direta: Decimal,
    pub taxa_inversa: Decimal,
    pub observacao: Option<String>,
    pub status: String,
    pub criado_em: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FiscalBrasilDto {
    pub regime_tributario: String,
    pub preparar_nfce: bool,
    pub preparar_nfe: bool,
    pub preparar_nfse: bool,
    pub regra_tributaria_base: Option<String>,
    pub ambiente: String,
    pub provedor_fiscal: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FiscalParaguaiDto {
    pub regime_tributario: String,
    pub preparar_sifen: bool,
    pub regra_tributaria_base: Option<String>,
    pub ambiente: String,
    pub provedor_fiscal: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FiscalBaseCompletoDto {
    pub brasil: Option<FiscalBrasilDto>,
    pub paraguai: Option<FiscalParaguaiDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParametrosOperacionaisDto {
    pub permitir_estoque_negativo: bool,
    pub bloquear_produto_vencido: bool,
    pub alertar_produto_vencendo: bool,
    pub dias_alerta_vencimento: i32,
    pub permitir_alterar_preco_pdv: bool,
    pub permitir_desconto_pdv: bool,
    pub exigir_supervisor_desconto: bool,
    pub exigir_supervisor_cancelamento: bool,
    pub permitir_venda_prazo: bool,
    pub exigir_cliente_completo_crediario: bool,
    pub permitir_venda_offline: bool,
    pub dias_maximos_offline: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditoriaEventoDto {
    pub id: i64,
    pub empresa_id: Option<Uuid>,
    pub usuario_id: Option<Uuid>,
    pub acao: String,
    pub entidade: String,
    pub entidade_id: Option<String>,
    pub valor_anterior: Option<serde_json::Value>,
    pub valor_novo: Option<serde_json::Value>,
    pub motivo: Option<String>,
    pub criado_em: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusConfiguracaoDto {
    pub empresa_nome: String,
    pub pais_fiscal: String,
    pub moeda_principal: String,
    pub status: String,
    pub configurada: bool,
}

// ================================================================
// Helpers
// ================================================================

fn obter_pool(state: &AppState) -> Result<&PgPool, axum::response::Response> {
    state.pool.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(RespostaBase::<()>::falha_manual(
                "Conexão com banco de dados indisponível",
                "ERRO_CONEXAO_POSTGRES",
                "A API local está rodando em modo degradado e não tem conexão com o PostgreSQL."
            ))
        ).into_response()
    })
}

async fn garantir_empresa_padrao(pool: &PgPool) -> Result<Uuid, sqlx::Error> {
    let empresa: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM empresas LIMIT 1")
        .fetch_optional(pool)
        .await?;

    if let Some((id,)) = empresa {
        return Ok(id);
    }

    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO empresas (id, codigo, nome, pais, ativo) VALUES ($1, 'MAIN', 'Aureon Principal', 'BR', true)"
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(id)
}

async fn garantir_registros_dependentes(pool: &PgPool, empresa_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO configuracoes_empresa (empresa_id, razao_social, tipo_pessoa, status_empresa)
         VALUES ($1, 'Aureon Principal Ltda', 'JURIDICA', 'EM_CONFIGURACAO')
         ON CONFLICT (empresa_id) DO NOTHING"
    )
    .bind(empresa_id)
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO empresas_documentos (empresa_id) VALUES ($1) ON CONFLICT (empresa_id) DO NOTHING"
    )
    .bind(empresa_id)
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO empresas_contatos (empresa_id, telefone_principal, responsavel)
         VALUES ($1, '45999999999', 'Administrador')
         ON CONFLICT (empresa_id) DO NOTHING"
    )
    .bind(empresa_id)
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO empresas_enderecos (empresa_id, pais, estado, cidade, logradouro)
         VALUES ($1, 'BR', 'PR', 'Foz do Iguaçu', 'Avenida Brasil')
         ON CONFLICT (empresa_id) DO NOTHING"
    )
    .bind(empresa_id)
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO empresas_logos (empresa_id) VALUES ($1) ON CONFLICT (empresa_id) DO NOTHING"
    )
    .bind(empresa_id)
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO parametros_operacionais_empresa (empresa_id) VALUES ($1) ON CONFLICT (empresa_id) DO NOTHING"
    )
    .bind(empresa_id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn registrar_auditoria(
    pool: &PgPool,
    empresa_id: Uuid,
    acao: &str,
    entidade: &str,
    entidade_id: Option<&str>,
    valor_anterior: Option<serde_json::Value>,
    valor_novo: Option<serde_json::Value>,
    motivo: Option<&str>,
) {
    sqlx::query(
        "INSERT INTO auditoria_eventos (empresa_id, acao, entidade, entidade_id, valor_anterior, valor_novo, motivo)
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(empresa_id)
    .bind(acao)
    .bind(entidade)
    .bind(entidade_id)
    .bind(valor_anterior)
    .bind(valor_novo)
    .bind(motivo)
    .execute(pool)
    .await
    .ok();
}

async fn registrar_evento_sync(pool: &PgPool, empresa_id: Uuid, tipo_evento: &str, payload: serde_json::Value) {
    sqlx::query(
        "INSERT INTO eventos_publicacao_configuracao (empresa_id, tipo_evento, payload) VALUES ($1, $2, $3)"
    )
    .bind(empresa_id)
    .bind(tipo_evento)
    .bind(payload)
    .execute(pool)
    .await
    .ok();
}

// ================================================================
// Handlers
// ================================================================

/// GET /empresa/configuracao
pub async fn obter_configuracao(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pool = match obter_pool(&state) {
        Ok(p) => p,
        Err(e) => return e,
    };

    match async {
        let id = garantir_empresa_padrao(pool).await?;
        garantir_registros_dependentes(pool, id).await?;

        let row = sqlx::query(
            "SELECT 
                e.id, e.codigo, e.nome as nome_fantasia,
                c.razao_social, c.tipo_pessoa, c.status_empresa, c.observacoes, e.criado_em,
                c.idioma_padrao, c.idioma_comprovantes, c.permitir_idioma_usuario, c.idioma_autoatendimento, c.ambiente_fiscal,
                d.cnpj, d.inscricao_estadual, d.inscricao_municipal, d.cpf, d.rg, d.ruc, d.ci,
                co.telefone_principal, co.whatsapp, co.telefone_secundario, co.email, co.responsavel, co.site,
                en.pais as pais_endereco, en.estado, en.cidade, en.bairro, en.logradouro, en.numero, en.complemento, en.cep, en.referencia,
                l.caminho_logo, l.usar_comprovantes, l.usar_relatorios
             FROM empresas e
             JOIN configuracoes_empresa c ON e.id = c.empresa_id
             LEFT JOIN empresas_documentos d ON e.id = d.empresa_id
             LEFT JOIN empresas_contatos co ON e.id = co.empresa_id
             LEFT JOIN empresas_enderecos en ON e.id = en.empresa_id
             LEFT JOIN empresas_logos l ON e.id = l.empresa_id
             WHERE e.id = $1"
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        let dto = EmpresaConfiguracaoCompletaDto {
            id: row.get("id"),
            codigo: row.get("codigo"),
            nome_fantasia: row.get("nome_fantasia"),
            razao_social: row.get("razao_social"),
            tipo_pessoa: row.get("tipo_pessoa"),
            status_empresa: row.get("status_empresa"),
            observacoes: row.get("observacoes"),
            criado_em: row.get("criado_em"),
            pais: row.get("pais_endereco"),
            ambiente_fiscal: row.get("ambiente_fiscal"),
            idioma_padrao: row.get("idioma_padrao"),
            idioma_comprovantes: row.get("idioma_comprovantes"),
            permitir_idioma_usuario: row.get("permitir_idioma_usuario"),
            idioma_autoatendimento: row.get("idioma_autoatendimento"),
            cnpj: row.get("cnpj"),
            inscricao_estadual: row.get("inscricao_estadual"),
            inscricao_municipal: row.get("inscricao_municipal"),
            cpf: row.get("cpf"),
            rg: row.get("rg"),
            ruc: row.get("ruc"),
            ci: row.get("ci"),
            telefone_principal: row.get("telefone_principal"),
            whatsapp: row.get("whatsapp"),
            telefone_secundario: row.get("telefone_secundario"),
            email: row.get("email"),
            responsavel: row.get("responsavel"),
            site: row.get("site"),
            estado: row.get("estado"),
            cidade: row.get("cidade"),
            bairro: row.get("bairro"),
            logradouro: row.get("logradouro"),
            numero: row.get("numero"),
            complemento: row.get("complemento"),
            cep: row.get("cep"),
            referencia: row.get("referencia"),
            caminho_logo: row.get("caminho_logo"),
            usar_comprovantes: row.get("usar_comprovantes"),
            usar_relatorios: row.get("usar_relatorios"),
        };

        Ok::<_, sqlx::Error>(dto)
    }.await {
        Ok(dto) => (StatusCode::OK, Json(RespostaBase::ok("Configurações obtidas", dto))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(RespostaBase::<()>::falha("Falha ao obter configuração", &AureonError::Interno(e.to_string())))).into_response(),
    }
}

/// POST/PUT /empresa/configuracao
pub async fn salvar_configuracao(
    State(state): State<AppState>,
    Json(dados): Json<EmpresaConfiguracaoCompletaDto>,
) -> impl IntoResponse {
    let pool = match obter_pool(&state) {
        Ok(p) => p,
        Err(e) => return e,
    };

    // Validações básicas obrigatórias
    if dados.nome_fantasia.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Nome fantasia obrigatório", "ERRO_EMPRESA_NOME_OBRIGATORIO", "O nome fantasia não pode estar vazio."))).into_response();
    }
    if dados.razao_social.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Razão social obrigatória", "ERRO_EMPRESA_NOME_OBRIGATORIO", "A razão social não pode estar vazia."))).into_response();
    }

    match async {
        // Pega valor anterior para auditoria
        let anterior_val = sqlx::query(
            "SELECT razao_social, status_empresa, idioma_padrao FROM configuracoes_empresa WHERE empresa_id = $1"
        )
        .bind(dados.id)
        .fetch_optional(pool)
        .await?
        .map(|r| json!({
            "razao_social": r.get::<String, _>("razao_social"),
            "status_empresa": r.get::<String, _>("status_empresa"),
            "idioma_padrao": r.get::<String, _>("idioma_padrao"),
        }));

        let mut tx = pool.begin().await?;

        // 1. empresas
        sqlx::query("UPDATE empresas SET nome = $1, pais = $2, atualizado_em = NOW() WHERE id = $3")
            .bind(&dados.nome_fantasia)
            .bind(&dados.pais)
            .bind(dados.id)
            .execute(&mut *tx)
            .await?;

        // 2. configuracoes_empresa
        sqlx::query(
            "UPDATE configuracoes_empresa 
             SET razao_social = $1, tipo_pessoa = $2, status_empresa = $3, observacoes = $4,
                 idioma_padrao = $5, idioma_comprovantes = $6, permitir_idioma_usuario = $7,
                 idioma_autoatendimento = $8, ambiente_fiscal = $9, atualizado_em = NOW()
             WHERE empresa_id = $10"
        )
        .bind(&dados.razao_social)
        .bind(&dados.tipo_pessoa)
        .bind(&dados.status_empresa)
        .bind(&dados.observacoes)
        .bind(&dados.idioma_padrao)
        .bind(&dados.idioma_comprovantes)
        .bind(dados.permitir_idioma_usuario)
        .bind(&dados.idioma_autoatendimento)
        .bind(&dados.ambiente_fiscal)
        .bind(dados.id)
        .execute(&mut *tx)
        .await?;

        // 3. empresas_documentos
        sqlx::query(
            "UPDATE empresas_documentos 
             SET cnpj = $1, inscricao_estadual = $2, inscricao_municipal = $3,
                 cpf = $4, rg = $5, ruc = $6, ci = $7
             WHERE empresa_id = $8"
        )
        .bind(&dados.cnpj)
        .bind(&dados.inscricao_estadual)
        .bind(&dados.inscricao_municipal)
        .bind(&dados.cpf)
        .bind(&dados.rg)
        .bind(&dados.ruc)
        .bind(&dados.ci)
        .execute(&mut *tx)
        .await?;

        // 4. empresas_contatos
        sqlx::query(
            "UPDATE empresas_contatos 
             SET telefone_principal = $1, whatsapp = $2, telefone_secundario = $3,
                 email = $4, responsavel = $5, site = $6
             WHERE empresa_id = $7"
        )
        .bind(&dados.telefone_principal)
        .bind(&dados.whatsapp)
        .bind(&dados.telefone_secundario)
        .bind(&dados.email)
        .bind(&dados.responsavel)
        .bind(&dados.site)
        .execute(&mut *tx)
        .await?;

        // 5. empresas_enderecos
        sqlx::query(
            "UPDATE empresas_enderecos 
             SET pais = $1, estado = $2, cidade = $3, bairro = $4, logradouro = $5,
                 numero = $6, complemento = $7, cep = $8, referencia = $9
             WHERE empresa_id = $10"
        )
        .bind(&dados.pais)
        .bind(&dados.estado)
        .bind(&dados.cidade)
        .bind(&dados.bairro)
        .bind(&dados.logradouro)
        .bind(&dados.numero)
        .bind(&dados.complemento)
        .bind(&dados.cep)
        .bind(&dados.referencia)
        .bind(dados.id)
        .execute(&mut *tx)
        .await?;

        // 6. empresas_logos
        sqlx::query(
            "UPDATE empresas_logos 
             SET caminho_logo = $1, usar_comprovantes = $2, usar_relatorios = $3
             WHERE empresa_id = $4"
        )
        .bind(&dados.caminho_logo)
        .bind(dados.usar_comprovantes)
        .bind(dados.usar_relatorios)
        .bind(dados.id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Auditoria
        let novo_val = json!({
            "razao_social": dados.razao_social,
            "status_empresa": dados.status_empresa,
            "idioma_padrao": dados.idioma_padrao,
        });

        registrar_auditoria(
            pool,
            dados.id,
            "ALTERAR",
            "configuracoes_empresa",
            Some(&dados.id.to_string()),
            anterior_val,
            Some(novo_val),
            Some("Alteração de configurações gerais da empresa")
        ).await;

        registrar_evento_sync(
            pool,
            dados.id,
            "EMPRESA_ALTERADA",
            json!({ "empresa_id": dados.id, "nome": dados.nome_fantasia, "status": dados.status_empresa })
        ).await;

        Ok::<_, sqlx::Error>(())
    }.await {
        Ok(_) => (StatusCode::OK, Json(RespostaBase::ok("Configurações salvas com sucesso", true))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(RespostaBase::<()>::falha("Falha ao salvar configuração", &AureonError::Interno(e.to_string())))).into_response(),
    }
}

/// GET /empresa/moedas
pub async fn obter_moedas(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pool = match obter_pool(&state) {
        Ok(p) => p,
        Err(e) => return e,
    };

    match async {
        let id = garantir_empresa_padrao(pool).await?;

        // Verifica multimoedas
        let param = sqlx::query(
            "SELECT permitir_pagamento_multiplo, permitir_troco_diferente FROM empresas_moedas WHERE empresa_id = $1 LIMIT 1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        let (p_mult, p_troco) = match param {
            Some(r) => (r.get::<bool, _>("permitir_pagamento_multiplo"), r.get::<bool, _>("permitir_troco_diferente")),
            None => (true, true)
        };

        // Carrega moedas associadas
        let rows = sqlx::query(
            "SELECT em.moeda_id, m.codigo, m.nome, m.simbolo, em.tipo_moeda, em.ordem_exibicao
             FROM empresas_moedas em
             JOIN moedas m ON em.moeda_id = m.id
             WHERE em.empresa_id = $1
             ORDER BY em.ordem_exibicao"
        )
        .bind(id)
        .fetch_all(pool)
        .await?;

        let moedas: Vec<MoedaConfigDto> = rows.iter().map(|r| MoedaConfigDto {
            moeda_id: r.get("moeda_id"),
            codigo: r.get("codigo"),
            nome: r.get("nome"),
            simbolo: r.get("simbolo"),
            tipo_moeda: r.get("tipo_moeda"),
            ordem_exibicao: r.get("ordem_exibicao"),
        }).collect();

        let dto = EmpresaMoedasCompletaDto {
            multimoeda_ativa: !moedas.is_empty(),
            permitir_pagamento_multiplo: p_mult,
            permitir_troco_diferente: p_troco,
            moedas,
        };

        Ok::<_, sqlx::Error>(dto)
    }.await {
        Ok(dto) => (StatusCode::OK, Json(RespostaBase::ok("Moedas carregadas", dto))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(RespostaBase::<()>::falha("Erro ao obter moedas", &AureonError::Interno(e.to_string())))).into_response(),
    }
}

/// PUT /empresa/moedas
pub async fn salvar_moedas(
    State(state): State<AppState>,
    Json(dados): Json<EmpresaMoedasCompletaDto>,
) -> impl IntoResponse {
    let pool = match obter_pool(&state) {
        Ok(p) => p,
        Err(e) => return e,
    };

    // Validações
    if dados.moedas.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Configuração inválida", "ERRO_MOEDA_PRINCIPAL_OBRIGATORIA", "A empresa precisa ter pelo menos uma moeda configurada."))).into_response();
    }

    let mut codigos = Vec::new();
    let mut tem_principal = false;
    for m in &dados.moedas {
        if codigos.contains(&m.codigo) {
            return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Moeda duplicada", "ERRO_MOEDAS_REPETIDAS", format!("A moeda '{}' está duplicada.", m.codigo)))).into_response();
        }
        codigos.push(m.codigo.clone());
        if m.tipo_moeda == "PRINCIPAL" {
            tem_principal = true;
        }
    }

    if !tem_principal {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Sem moeda principal", "ERRO_MOEDA_PRINCIPAL_OBRIGATORIA", "É obrigatório definir uma moeda PRINCIPAL."))).into_response();
    }

    match async {
        let id = garantir_empresa_padrao(pool).await?;

        // Histórico anterior para auditoria
        let anterior_moedas = sqlx::query("SELECT moeda_id, tipo_moeda FROM empresas_moedas WHERE empresa_id = $1")
            .bind(id)
            .fetch_all(pool)
            .await?
            .iter()
            .map(|r| json!({ "id": r.get::<Uuid, _>("moeda_id"), "tipo": r.get::<String, _>("tipo_moeda") }))
            .collect::<Vec<_>>();

        let mut tx = pool.begin().await?;

        // Limpa moedas anteriores
        sqlx::query("DELETE FROM empresas_moedas WHERE empresa_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        // Insere as novas
        for m in &dados.moedas {
            sqlx::query(
                "INSERT INTO empresas_moedas (empresa_id, moeda_id, tipo_moeda, ordem_exibicao, permitir_pagamento_multiplo, permitir_troco_diferente)
                 VALUES ($1, $2, $3, $4, $5, $6)"
            )
            .bind(id)
            .bind(m.moeda_id)
            .bind(&m.tipo_moeda)
            .bind(m.ordem_exibicao)
            .bind(dados.permitir_pagamento_multiplo)
            .bind(dados.permitir_troco_diferente)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        // Auditoria
        let nova_config = dados.moedas.iter()
            .map(|m| json!({ "id": m.moeda_id, "tipo": m.tipo_moeda }))
            .collect::<Vec<_>>();

        registrar_auditoria(
            pool,
            id,
            "ALTERAR",
            "empresas_moedas",
            Some(&id.to_string()),
            Some(json!(anterior_moedas)),
            Some(json!(nova_config)),
            Some("Alteração da configuração de moedas e multimoedas")
        ).await;

        registrar_evento_sync(
            pool,
            id,
            "MOEDAS_CONFIGURADAS",
            json!({ "empresa_id": id, "moedas": nova_config })
        ).await;

        Ok::<_, sqlx::Error>(())
    }.await {
        Ok(_) => (StatusCode::OK, Json(RespostaBase::ok("Moedas atualizadas", true))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(RespostaBase::<()>::falha("Falha ao salvar moedas", &AureonError::Interno(e.to_string())))).into_response(),
    }
}

/// GET /empresa/cotacoes
pub async fn obter_cotacoes(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pool = match obter_pool(&state) {
        Ok(p) => p,
        Err(e) => return e,
    };

    match async {
        let id = garantir_empresa_padrao(pool).await?;

        let rows = sqlx::query(
            "SELECT c.id, c.data_cotacao, c.moeda_origem_id, mo.codigo as origem_cod,
                    c.moeda_destino_id, md.codigo as destino_cod, c.taxa_direta, c.taxa_inversa,
                    c.observacao, c.status, c.criado_em
             FROM cotacoes_moedas c
             JOIN moedas mo ON c.moeda_origem_id = mo.id
             JOIN moedas md ON c.moeda_destino_id = md.id
             WHERE c.empresa_id = $1
             ORDER BY c.data_cotacao DESC, c.criado_em DESC"
        )
        .bind(id)
        .fetch_all(pool)
        .await?;

        let cotacoes: Vec<CotacaoExibicaoDto> = rows.iter().map(|r| CotacaoExibicaoDto {
            id: r.get("id"),
            data_cotacao: r.get("data_cotacao"),
            moeda_origem_id: r.get("moeda_origem_id"),
            moeda_origem_codigo: r.get("origem_cod"),
            moeda_destino_id: r.get("moeda_destino_id"),
            moeda_destino_codigo: r.get("destino_cod"),
            taxa_direta: r.get("taxa_direta"),
            taxa_inversa: r.get("taxa_inversa"),
            observacao: r.get("observacao"),
            status: r.get("status"),
            criado_em: r.get("criado_em"),
        }).collect();

        Ok::<_, sqlx::Error>(cotacoes)
    }.await {
        Ok(lst) => (StatusCode::OK, Json(RespostaBase::ok("Cotações obtidas", lst))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(RespostaBase::<()>::falha("Falha ao carregar cotações", &AureonError::Interno(e.to_string())))).into_response(),
    }
}

/// POST /empresa/cotacoes
pub async fn criar_cotacao(
    State(state): State<AppState>,
    Json(dados): Json<CotacaoCadastroDto>,
) -> impl IntoResponse {
    let pool = match obter_pool(&state) {
        Ok(p) => p,
        Err(e) => return e,
    };

    // Validações
    if dados.taxa_direta <= Decimal::ZERO {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Taxa inválida", "ERRO_COTACAO_TAXA_INVALIDA", "A taxa de cotação deve ser maior que zero."))).into_response();
    }
    if dados.moeda_origem_codigo == dados.moeda_destino_codigo {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Moedas iguais", "ERRO_COTACAO_MOEDAS_IGUAIS", "A moeda de origem deve ser diferente da moeda de destino."))).into_response();
    }

    match async {
        let id = garantir_empresa_padrao(pool).await?;

        // Recupera IDs das moedas por código
        let m_origem: (Uuid,) = sqlx::query_as("SELECT id FROM moedas WHERE codigo = $1")
            .bind(&dados.moeda_origem_codigo)
            .fetch_one(pool)
            .await?;

        let m_destino: (Uuid,) = sqlx::query_as("SELECT id FROM moedas WHERE codigo = $1")
            .bind(&dados.moeda_destino_codigo)
            .fetch_one(pool)
            .await?;

        // Calcula taxa inversa
        let taxa_inversa = Decimal::ONE / dados.taxa_direta;

        let mut tx = pool.begin().await?;

        // Inativa cotações anteriores ativas deste par para o mesmo dia
        sqlx::query(
            "UPDATE cotacoes_moedas 
             SET status = 'SUBSTITUIDA', atualizado_em = NOW()
             WHERE empresa_id = $1 AND data_cotacao = CURRENT_DATE 
               AND moeda_origem_id = $2 AND moeda_destino_id = $3
               AND status = 'ATIVA'"
        )
        .bind(id)
        .bind(m_origem.0)
        .bind(m_destino.0)
        .execute(&mut *tx)
        .await?;

        // Insere a nova cotação
        let c_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO cotacoes_moedas (id, empresa_id, moeda_origem_id, moeda_destino_id, taxa_direta, taxa_inversa, status, observacao)
             VALUES ($1, $2, $3, $4, $5, $6, 'ATIVA', $7)"
        )
        .bind(c_id)
        .bind(id)
        .bind(m_origem.0)
        .bind(m_destino.0)
        .bind(dados.taxa_direta)
        .bind(taxa_inversa)
        .bind(&dados.observacao)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Auditoria
        registrar_auditoria(
            pool,
            id,
            "CRIAR",
            "cotacoes_moedas",
            Some(&c_id.to_string()),
            None,
            Some(json!({ "origem": dados.moeda_origem_codigo, "destino": dados.moeda_destino_codigo, "taxa": dados.taxa_direta })),
            Some("Criação de nova cotação manual")
        ).await;

        registrar_evento_sync(
            pool,
            id,
            "COTACAO_CRIADA",
            json!({ "cotacao_id": c_id, "origem": dados.moeda_origem_codigo, "destino": dados.moeda_destino_codigo, "taxa": dados.taxa_direta })
        ).await;

        Ok::<_, sqlx::Error>(())
    }.await {
        Ok(_) => (StatusCode::CREATED, Json(RespostaBase::ok("Cotação inserida", true))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(RespostaBase::<()>::falha("Erro ao cadastrar cotação", &AureonError::Interno(e.to_string())))).into_response(),
    }
}

/// PUT /empresa/cotacoes/{id}/cancelar
pub async fn cancelar_cotacao(
    State(state): State<AppState>,
    Path(cotacao_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = match obter_pool(&state) {
        Ok(p) => p,
        Err(e) => return e,
    };

    match async {
        let id = garantir_empresa_padrao(pool).await?;

        let mut tx = pool.begin().await?;

        // Cancela
        sqlx::query(
            "UPDATE cotacoes_moedas 
             SET status = 'CANCELADA', atualizado_em = NOW()
             WHERE id = $1 AND empresa_id = $2 AND status = 'ATIVA'"
        )
        .bind(cotacao_id)
        .bind(id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        registrar_auditoria(
            pool,
            id,
            "CANCELAR",
            "cotacoes_moedas",
            Some(&cotacao_id.to_string()),
            Some(json!({ "status": "ATIVA" })),
            Some(json!({ "status": "CANCELADA" })),
            Some("Cotação cancelada manualmente")
        ).await;

        Ok::<_, sqlx::Error>(())
    }.await {
        Ok(_) => (StatusCode::OK, Json(RespostaBase::ok("Cotação cancelada com sucesso", true))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(RespostaBase::<()>::falha("Erro ao cancelar cotação", &AureonError::Interno(e.to_string())))).into_response(),
    }
}

/// GET /empresa/fiscal
pub async fn obter_fiscal(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pool = match obter_pool(&state) {
        Ok(p) => p,
        Err(e) => return e,
    };

    match async {
        let id = garantir_empresa_padrao(pool).await?;

        // 1. Brasil
        let br_row = sqlx::query(
            "SELECT regime_tributario, preparar_nfce, preparar_nfe, preparar_nfse, regra_tributaria_base, ambiente, provedor_fiscal
             FROM configuracoes_fiscais_brasil
             WHERE empresa_id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        let br_dto = br_row.map(|r| FiscalBrasilDto {
            regime_tributario: r.get("regime_tributario"),
            preparar_nfce: r.get("preparar_nfce"),
            preparar_nfe: r.get("preparar_nfe"),
            preparar_nfse: r.get("preparar_nfse"),
            regra_tributaria_base: r.get("regra_tributaria_base"),
            ambiente: r.get("ambiente"),
            provedor_fiscal: r.get("provedor_fiscal"),
        });

        // 2. Paraguai
        let py_row = sqlx::query(
            "SELECT regime_tributario, preparar_sifen, regra_tributaria_base, ambiente, provedor_fiscal
             FROM configuracoes_fiscais_paraguai
             WHERE empresa_id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        let py_dto = py_row.map(|r| FiscalParaguaiDto {
            regime_tributario: r.get("regime_tributario"),
            preparar_sifen: r.get("preparar_sifen"),
            regra_tributaria_base: r.get("regra_tributaria_base"),
            ambiente: r.get("ambiente"),
            provedor_fiscal: r.get("provedor_fiscal"),
        });

        Ok::<_, sqlx::Error>(FiscalBaseCompletoDto { brasil: br_dto, paraguai: py_dto })
    }.await {
        Ok(dto) => (StatusCode::OK, Json(RespostaBase::ok("Fiscal base carregado", dto))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(RespostaBase::<()>::falha("Falha ao obter fiscal base", &AureonError::Interno(e.to_string())))).into_response(),
    }
}

/// PUT /empresa/fiscal
pub async fn salvar_fiscal(
    State(state): State<AppState>,
    Json(dados): Json<FiscalBaseCompletoDto>,
) -> impl IntoResponse {
    let pool = match obter_pool(&state) {
        Ok(p) => p,
        Err(e) => return e,
    };

    match async {
        let id = garantir_empresa_padrao(pool).await?;

        let mut tx = pool.begin().await?;

        // 1. Brasil
        if let Some(br) = &dados.brasil {
            sqlx::query(
                "INSERT INTO configuracoes_fiscais_brasil (empresa_id, regime_tributario, preparar_nfce, preparar_nfe, preparar_nfse, regra_tributaria_base, ambiente, provedor_fiscal)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                 ON CONFLICT (empresa_id) DO UPDATE 
                 SET regime_tributario = $2, preparar_nfce = $3, preparar_nfe = $4, preparar_nfse = $5,
                     regra_tributaria_base = $6, ambiente = $7, provedor_fiscal = $8, atualizado_em = NOW()"
            )
            .bind(id)
            .bind(&br.regime_tributario)
            .bind(br.preparar_nfce)
            .bind(br.preparar_nfe)
            .bind(br.preparar_nfse)
            .bind(&br.regra_tributaria_base)
            .bind(&br.ambiente)
            .bind(&br.provedor_fiscal)
            .execute(&mut *tx)
            .await?;
        }

        // 2. Paraguai
        if let Some(py) = &dados.paraguai {
            sqlx::query(
                "INSERT INTO configuracoes_fiscais_paraguai (empresa_id, regime_tributario, preparar_sifen, regra_tributaria_base, ambiente, provedor_fiscal)
                 VALUES ($1, $2, $3, $4, $5, $6)
                 ON CONFLICT (empresa_id) DO UPDATE 
                 SET regime_tributario = $2, preparar_sifen = $3, regra_tributaria_base = $4,
                     ambiente = $5, provedor_fiscal = $6, atualizado_em = NOW()"
            )
            .bind(id)
            .bind(&py.regime_tributario)
            .bind(py.preparar_sifen)
            .bind(&py.regra_tributaria_base)
            .bind(&py.ambiente)
            .bind(&py.provedor_fiscal)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        registrar_auditoria(
            pool,
            id,
            "ALTERAR",
            "fiscal_base",
            Some(&id.to_string()),
            None,
            Some(json!({ "atualizado": true })),
            Some("Alteração da configuração fiscal base")
        ).await;

        Ok::<_, sqlx::Error>(())
    }.await {
        Ok(_) => (StatusCode::OK, Json(RespostaBase::ok("Fiscal base atualizado", true))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(RespostaBase::<()>::falha("Falha ao salvar fiscal base", &AureonError::Interno(e.to_string())))).into_response(),
    }
}

/// GET /empresa/parametros-operacionais
pub async fn obter_parametros(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pool = match obter_pool(&state) {
        Ok(p) => p,
        Err(e) => return e,
    };

    match async {
        let id = garantir_empresa_padrao(pool).await?;
        garantir_registros_dependentes(pool, id).await?;

        let r = sqlx::query(
            "SELECT permitir_estoque_negativo, bloquear_produto_vencido, alertar_produto_vencendo,
                    dias_alerta_vencimento, permitir_alterar_preco_pdv, permitir_desconto_pdv,
                    exigir_supervisor_desconto, exigir_supervisor_cancelamento, permitir_venda_prazo,
                    exigir_cliente_completo_crediario, permitir_venda_offline, dias_maximos_offline
             FROM parametros_operacionais_empresa
             WHERE empresa_id = $1"
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        let dto = ParametrosOperacionaisDto {
            permitir_estoque_negativo: r.get("permitir_estoque_negativo"),
            bloquear_produto_vencido: r.get("bloquear_produto_vencido"),
            alertar_produto_vencendo: r.get("alertar_produto_vencendo"),
            dias_alerta_vencimento: r.get("dias_alerta_vencimento"),
            permitir_alterar_preco_pdv: r.get("permitir_alterar_preco_pdv"),
            permitir_desconto_pdv: r.get("permitir_desconto_pdv"),
            exigir_supervisor_desconto: r.get("exigir_supervisor_desconto"),
            exigir_supervisor_cancelamento: r.get("exigir_supervisor_cancelamento"),
            permitir_venda_prazo: r.get("permitir_venda_prazo"),
            exigir_cliente_completo_crediario: r.get("exigir_cliente_completo_crediario"),
            permitir_venda_offline: r.get("permitir_venda_offline"),
            dias_maximos_offline: r.get("dias_maximos_offline"),
        };

        Ok::<_, sqlx::Error>(dto)
    }.await {
        Ok(dto) => (StatusCode::OK, Json(RespostaBase::ok("Parâmetros obtidos", dto))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(RespostaBase::<()>::falha("Falha ao obter parâmetros", &AureonError::Interno(e.to_string())))).into_response(),
    }
}

/// PUT /empresa/parametros-operacionais
pub async fn salvar_parametros(
    State(state): State<AppState>,
    Json(dados): Json<ParametrosOperacionaisDto>,
) -> impl IntoResponse {
    let pool = match obter_pool(&state) {
        Ok(p) => p,
        Err(e) => return e,
    };

    // Validações
    if dados.dias_alerta_vencimento < 0 {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Parâmetro inválido", "ERRO_PARAMETRO_INVALIDO", "Dias de alerta de vencimento não pode ser negativo."))).into_response();
    }
    if dados.dias_maximos_offline <= 0 {
        return (StatusCode::BAD_REQUEST, Json(RespostaBase::<()>::falha_manual("Parâmetro inválido", "ERRO_PARAMETRO_INVALIDO", "Dias máximos offline deve ser maior que zero."))).into_response();
    }

    match async {
        let id = garantir_empresa_padrao(pool).await?;

        // Estado anterior para auditoria
        let anterior_val = sqlx::query(
            "SELECT permitir_estoque_negativo, bloquear_produto_vencido, permitir_venda_offline FROM parametros_operacionais_empresa WHERE empresa_id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .map(|r| json!({
            "permitir_estoque_negativo": r.get::<bool, _>("permitir_estoque_negativo"),
            "bloquear_produto_vencido": r.get::<bool, _>("bloquear_produto_vencido"),
            "permitir_venda_offline": r.get::<bool, _>("permitir_venda_offline")
        }));

        let mut tx = pool.begin().await?;

        sqlx::query(
            "UPDATE parametros_operacionais_empresa 
             SET permitir_estoque_negativo = $1, bloquear_produto_vencido = $2, alertar_produto_vencendo = $3,
                 dias_alerta_vencimento = $4, permitir_alterar_preco_pdv = $5, permitir_desconto_pdv = $6,
                 exigir_supervisor_desconto = $7, exigir_supervisor_cancelamento = $8, permitir_venda_prazo = $9,
                 exigir_cliente_completo_crediario = $10, permitir_venda_offline = $11, dias_maximos_offline = $12,
                 atualizado_em = NOW()
             WHERE empresa_id = $13"
        )
        .bind(dados.permitir_estoque_negativo)
        .bind(dados.bloquear_produto_vencido)
        .bind(dados.alertar_produto_vencendo)
        .bind(dados.dias_alerta_vencimento)
        .bind(dados.permitir_alterar_preco_pdv)
        .bind(dados.permitir_desconto_pdv)
        .bind(dados.exigir_supervisor_desconto)
        .bind(dados.exigir_supervisor_cancelamento)
        .bind(dados.permitir_venda_prazo)
        .bind(dados.exigir_cliente_completo_crediario)
        .bind(dados.permitir_venda_offline)
        .bind(dados.dias_maximos_offline)
        .bind(id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        let novo_val = json!({
            "permitir_estoque_negativo": dados.permitir_estoque_negativo,
            "bloquear_produto_vencido": dados.bloquear_produto_vencido,
            "permitir_venda_offline": dados.permitir_venda_offline
        });

        registrar_auditoria(
            pool,
            id,
            "ALTERAR",
            "parametros_operacionais_empresa",
            Some(&id.to_string()),
            anterior_val,
            Some(novo_val),
            Some("Alteração dos parâmetros operacionais da empresa")
        ).await;

        registrar_evento_sync(
            pool,
            id,
            "PARAMETROS_OPERACIONAIS_ALTERADOS",
            json!({
                "permitir_estoque_negativo": dados.permitir_estoque_negativo,
                "bloquear_produto_vencido": dados.bloquear_produto_vencido,
                "permitir_venda_offline": dados.permitir_venda_offline
            })
        ).await;

        Ok::<_, sqlx::Error>(())
    }.await {
        Ok(_) => (StatusCode::OK, Json(RespostaBase::ok("Parâmetros atualizados com sucesso", true))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(RespostaBase::<()>::falha("Falha ao salvar parâmetros", &AureonError::Interno(e.to_string())))).into_response(),
    }
}

/// GET /empresa/auditoria
pub async fn obter_auditoria(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pool = match obter_pool(&state) {
        Ok(p) => p,
        Err(e) => return e,
    };

    match async {
        let id = garantir_empresa_padrao(pool).await?;

        let rows = sqlx::query(
            "SELECT id, empresa_id, usuario_id, acao, entidade, entidade_id, valor_anterior, valor_novo, motivo, criado_em
             FROM auditoria_eventos
             WHERE empresa_id = $1
             ORDER BY criado_em DESC
             LIMIT 100"
        )
        .bind(id)
        .fetch_all(pool)
        .await?;

        let eventos: Vec<AuditoriaEventoDto> = rows.iter().map(|r| AuditoriaEventoDto {
            id: r.get("id"),
            empresa_id: r.get("empresa_id"),
            usuario_id: r.get("usuario_id"),
            acao: r.get("acao"),
            entidade: r.get("entidade"),
            entidade_id: r.get("entidade_id"),
            valor_anterior: r.get("valor_anterior"),
            valor_novo: r.get("valor_novo"),
            motivo: r.get("motivo"),
            criado_em: r.get("criado_em"),
        }).collect();

        Ok::<_, sqlx::Error>(eventos)
    }.await {
        Ok(lst) => (StatusCode::OK, Json(RespostaBase::ok("Eventos de auditoria carregados", lst))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(RespostaBase::<()>::falha("Erro ao obter auditoria", &AureonError::Interno(e.to_string())))).into_response(),
    }
}

/// GET /empresa/status-configuracao
pub async fn status_configuracao(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pool = match obter_pool(&state) {
        Ok(p) => p,
        Err(e) => return e,
    };

    match async {
        let id = garantir_empresa_padrao(pool).await?;
        garantir_registros_dependentes(pool, id).await?;

        let emp_row = sqlx::query("SELECT nome FROM empresas WHERE id = $1").bind(id).fetch_one(pool).await?;
        let conf_row = sqlx::query("SELECT status_empresa, idioma_padrao FROM configuracoes_empresa WHERE empresa_id = $1").bind(id).fetch_one(pool).await?;
        
        let end_row = sqlx::query("SELECT pais FROM empresas_enderecos WHERE empresa_id = $1").bind(id).fetch_one(pool).await?;

        let m_principal = sqlx::query(
            "SELECT m.codigo 
             FROM empresas_moedas em 
             JOIN moedas m ON em.moeda_id = m.id 
             WHERE em.empresa_id = $1 AND em.tipo_moeda = 'PRINCIPAL'"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .map(|r| r.get::<String, _>("codigo"))
        .unwrap_or_else(|| "BRL".to_string());

        let status: String = conf_row.get("status_empresa");

        let dto = StatusConfiguracaoDto {
            empresa_nome: emp_row.get("nome"),
            pais_fiscal: end_row.get("pais"),
            moeda_principal: m_principal,
            status: status.clone(),
            configurada: status == "ATIVA",
        };

        Ok::<_, sqlx::Error>(dto)
    }.await {
        Ok(dto) => (StatusCode::OK, Json(RespostaBase::ok("Status de configuração obtido", dto))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(RespostaBase::<()>::falha("Erro ao obter status", &AureonError::Interno(e.to_string())))).into_response(),
    }
}
