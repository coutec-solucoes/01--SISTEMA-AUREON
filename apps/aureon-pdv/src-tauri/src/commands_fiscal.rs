use tauri::State;
use aureon_core::{dtos::*, RespostaBase};
use crate::estado::EstadoApp;
use uuid::Uuid;
use chrono::Utc;

// ============================================================
// FASE 16 BLOCO 2 â€” COMMANDS FISCAIS (PARÃ‚METROS E ESTRUTURA)
// PROIBIDO: emissÃ£o, XML, DANFE, QR fiscal, transmissÃ£o.
// ============================================================

// --- CONFIGURAÃ‡ÃƒO DA EMPRESA ---

#[tauri::command]
pub async fn obter_configuracao_fiscal_empresa(
    estado: tauri::State<'_, EstadoApp>,
) -> Result<RespostaBase<FiscalEmpresaConfigResp>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let row = conn.query_row(
        "SELECT id, pais_fiscal, regime_fiscal, ambiente, forma_emissao,
                certificado_alias, certificado_caminho, configuracao_json
         FROM fiscal_empresa_cache LIMIT 1",
        [],
        |r| {
            Ok(FiscalEmpresaConfigResp {
                id: r.get(0)?,
                pais_fiscal: r.get(1)?,
                regime_fiscal: r.get(2)?,
                ambiente: r.get(3)?,
                forma_emissao: r.get(4)?,
                certificado_alias: r.get(5)?,
                certificado_caminho: r.get(6)?,
                configuracao_json: r.get(7)?,
            })
        },
    );

    match row {
        Ok(c) => Ok(RespostaBase::ok("Configuracao fiscal obtida.", c)),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            let default_config = FiscalEmpresaConfigResp {
                id: "".to_string(),
                pais_fiscal: "BR".to_string(),
                regime_fiscal: None,
                ambiente: "HOMOLOGACAO".to_string(),
                forma_emissao: "NORMAL".to_string(),
                certificado_alias: None,
                certificado_caminho: None,
                configuracao_json: None,
            };
            Ok(RespostaBase::ok("Nenhuma configuracao encontrada, retornando padrao.", default_config))
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn salvar_configuracao_fiscal_empresa(
    req: SalvarFiscalEmpresaConfigReq,
    estado: tauri::State<'_, EstadoApp>,
) -> Result<RespostaBase<String>, String> {
    if req.pais_fiscal != "BR" && req.pais_fiscal != "PY" {
        return Err("pais_fiscal deve ser BR ou PY.".into());
    }
    if req.ambiente != "HOMOLOGACAO" && req.ambiente != "PRODUCAO" {
        return Err("ambiente deve ser HOMOLOGACAO ou PRODUCAO.".into());
    }
    if req.forma_emissao != "NORMAL" && req.forma_emissao != "CONTINGENCIA_OFFLINE" {
        return Err("forma_emissao deve ser NORMAL ou CONTINGENCIA_OFFLINE.".into());
    }

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    tx.execute(
        "INSERT INTO fiscal_empresa_cache
            (id, pais_fiscal, regime_fiscal, ambiente, forma_emissao,
             certificado_alias, certificado_caminho, atualizado_em)
         VALUES ('config-fiscal-padrao', ?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET
            pais_fiscal = excluded.pais_fiscal,
            regime_fiscal = excluded.regime_fiscal,
            ambiente = excluded.ambiente,
            forma_emissao = excluded.forma_emissao,
            certificado_alias = excluded.certificado_alias,
            certificado_caminho = excluded.certificado_caminho,
            atualizado_em = excluded.atualizado_em",
        rusqlite::params![
            req.pais_fiscal,
            req.regime_fiscal,
            req.ambiente,
            req.forma_emissao,
            req.certificado_alias,
            req.certificado_caminho,
            agora
        ],
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(RespostaBase::ok("Configuracao fiscal salva.", "OK".to_string()))
}

// --- DICIONÃRIOS (SOMENTE LEITURA) ---

#[tauri::command]
pub async fn listar_fiscal_ncm(
    estado: tauri::State<'_, EstadoApp>,
) -> Result<RespostaBase<Vec<FiscalNcmResp>>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, codigo, descricao, ativo FROM fiscal_ncm_cache ORDER BY codigo")
        .map_err(|e| e.to_string())?;

    let lista: Vec<FiscalNcmResp> = stmt
        .query_map([], |r| {
            Ok(FiscalNcmResp {
                id: r.get(0)?,
                codigo: r.get(1)?,
                descricao: r.get(2)?,
                ativo: r.get::<_, i32>(3)? == 1,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(RespostaBase::ok("", lista))
}

#[tauri::command]
pub async fn listar_fiscal_cfop(
    estado: tauri::State<'_, EstadoApp>,
) -> Result<RespostaBase<Vec<FiscalCfopResp>>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, codigo, descricao, tipo_operacao, ativo FROM fiscal_cfop_cache ORDER BY codigo")
        .map_err(|e| e.to_string())?;

    let lista: Vec<FiscalCfopResp> = stmt
        .query_map([], |r| {
            Ok(FiscalCfopResp {
                id: r.get(0)?,
                codigo: r.get(1)?,
                descricao: r.get(2)?,
                tipo_operacao: r.get(3)?,
                ativo: r.get::<_, i32>(4)? == 1,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(RespostaBase::ok("", lista))
}

#[tauri::command]
pub async fn listar_fiscal_cst_csosn(
    estado: tauri::State<'_, EstadoApp>,
) -> Result<RespostaBase<Vec<FiscalCstCsosnResp>>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, codigo, tipo, descricao, ativo FROM fiscal_cst_csosn_cache ORDER BY tipo, codigo")
        .map_err(|e| e.to_string())?;

    let lista: Vec<FiscalCstCsosnResp> = stmt
        .query_map([], |r| {
            Ok(FiscalCstCsosnResp {
                id: r.get(0)?,
                codigo: r.get(1)?,
                tipo: r.get(2)?,
                descricao: r.get(3)?,
                ativo: r.get::<_, i32>(4)? == 1,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(RespostaBase::ok("", lista))
}

#[tauri::command]
pub async fn listar_fiscal_iva(
    estado: tauri::State<'_, EstadoApp>,
) -> Result<RespostaBase<Vec<FiscalIvaResp>>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, codigo, descricao, aliquota_escala6, ativo FROM fiscal_iva_cache ORDER BY codigo")
        .map_err(|e| e.to_string())?;

    let lista: Vec<FiscalIvaResp> = stmt
        .query_map([], |r| {
            Ok(FiscalIvaResp {
                id: r.get(0)?,
                codigo: r.get(1)?,
                descricao: r.get(2)?,
                aliquota_escala6: r.get(3)?,
                ativo: r.get::<_, i32>(4)? == 1,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(RespostaBase::ok("", lista))
}

#[tauri::command]
pub async fn listar_fiscal_regras_tributarias(
    estado: tauri::State<'_, EstadoApp>,
) -> Result<RespostaBase<Vec<FiscalRegraTributariaResp>>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, pais_fiscal, tipo_operacao, uf_origem, uf_destino,
                    ncm_id, cfop_id, cst_csosn_id, iva_id,
                    aliquota_icms_escala6, aliquota_pis_escala6, aliquota_cofins_escala6,
                    aliquota_iva_escala6, reducao_base_escala6, ativo
             FROM fiscal_regras_tributarias_cache ORDER BY pais_fiscal, tipo_operacao",
        )
        .map_err(|e| e.to_string())?;

    let lista: Vec<FiscalRegraTributariaResp> = stmt
        .query_map([], |r| {
            Ok(FiscalRegraTributariaResp {
                id: r.get(0)?,
                pais_fiscal: r.get(1)?,
                tipo_operacao: r.get(2)?,
                uf_origem: r.get(3)?,
                uf_destino: r.get(4)?,
                ncm_id: r.get(5)?,
                cfop_id: r.get(6)?,
                cst_csosn_id: r.get(7)?,
                iva_id: r.get(8)?,
                aliquota_icms_escala6: r.get(9)?,
                aliquota_pis_escala6: r.get(10)?,
                aliquota_cofins_escala6: r.get(11)?,
                aliquota_iva_escala6: r.get(12)?,
                reducao_base_escala6: r.get(13)?,
                ativo: r.get::<_, i32>(14)? == 1,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(RespostaBase::ok("", lista))
}

// --- MANUTENÃ‡ÃƒO ESTRUTURAL ---

#[tauri::command]
pub async fn salvar_fiscal_iva(
    req: SalvarFiscalIvaReq,
    estado: tauri::State<'_, EstadoApp>,
) -> Result<RespostaBase<String>, String> {
    if req.aliquota_escala6 < 0 {
        return Err("aliquota_escala6 nao pode ser negativa.".into());
    }
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let id = format!("iva-{}", req.codigo);

    tx.execute(
        "INSERT INTO fiscal_iva_cache (id, codigo, descricao, aliquota_escala6, ativo, atualizado_em)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(id) DO UPDATE SET
            descricao = excluded.descricao,
            aliquota_escala6 = excluded.aliquota_escala6,
            ativo = excluded.ativo,
            atualizado_em = excluded.atualizado_em",
        rusqlite::params![
            id, req.codigo, req.descricao, req.aliquota_escala6,
            if req.ativo { 1 } else { 0 },
            agora
        ],
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(RespostaBase::ok("IVA salvo.", id))
}

#[tauri::command]
pub async fn salvar_regra_tributaria(
    req: SalvarFiscalRegraTributariaReq,
    estado: tauri::State<'_, EstadoApp>,
) -> Result<RespostaBase<String>, String> {
    if req.pais_fiscal != "BR" && req.pais_fiscal != "PY" {
        return Err("pais_fiscal deve ser BR ou PY.".into());
    }
    if req.aliquota_icms_escala6 < 0
        || req.aliquota_pis_escala6 < 0
        || req.aliquota_cofins_escala6 < 0
        || req.aliquota_iva_escala6 < 0
        || req.reducao_base_escala6 < 0
    {
        return Err("Aliquotas nao podem ser negativas.".into());
    }

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let id = Uuid::new_v4().to_string();

    tx.execute(
        "INSERT INTO fiscal_regras_tributarias_cache
            (id, pais_fiscal, tipo_operacao, uf_origem, uf_destino,
             ncm_id, cfop_id, cst_csosn_id, iva_id,
             aliquota_icms_escala6, aliquota_pis_escala6, aliquota_cofins_escala6,
             aliquota_iva_escala6, reducao_base_escala6, ativo, atualizado_em)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
        rusqlite::params![
            id, req.pais_fiscal, req.tipo_operacao,
            req.uf_origem, req.uf_destino,
            req.ncm_id, req.cfop_id, req.cst_csosn_id, req.iva_id,
            req.aliquota_icms_escala6, req.aliquota_pis_escala6,
            req.aliquota_cofins_escala6, req.aliquota_iva_escala6,
            req.reducao_base_escala6, if req.ativo { 1 } else { 0 },
            agora
        ],
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(RespostaBase::ok("Regra tributaria salva.", id))
}

#[tauri::command]
pub async fn vincular_fiscal_produto(
    req: VincularFiscalProdutoReq,
    estado: tauri::State<'_, EstadoApp>,
) -> Result<RespostaBase<String>, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let existe: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM produtos_cache WHERE id = ?1",
            rusqlite::params![req.produto_id],
            |r| r.get(0),
        )
        .unwrap_or(false);

    if !existe {
        return Err("Produto nao encontrado.".into());
    }

    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    conn.execute(
        "UPDATE produtos_cache SET
            ncm_id = ?1, iva_id = ?2, cst_csosn_id = ?3,
            cfop_padrao_id = ?4, origem_mercadoria = ?5, fiscal_atualizado_em = ?6
         WHERE id = ?7",
        rusqlite::params![
            req.ncm_id, req.iva_id, req.cst_csosn_id,
            req.cfop_padrao_id, req.origem_mercadoria,
            agora, req.produto_id
        ],
    ).map_err(|e| e.to_string())?;

    Ok(RespostaBase::ok("Vinculos fiscais do produto atualizados.", "OK".to_string()))
}

// --- LOGS FISCAIS (SOMENTE LEITURA) ---

#[tauri::command]
pub async fn listar_fiscal_eventos_logs(
    estado: tauri::State<'_, EstadoApp>,
) -> Result<RespostaBase<Vec<FiscalEventoLogResp>>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, venda_id, tipo_evento, origem, payload_preview, mensagem, criado_em
             FROM fiscal_eventos_logs ORDER BY criado_em DESC LIMIT 100",
        )
        .map_err(|e| e.to_string())?;

    let lista: Vec<FiscalEventoLogResp> = stmt
        .query_map([], |r| {
            Ok(FiscalEventoLogResp {
                id: r.get(0)?,
                venda_id: r.get(1)?,
                tipo_evento: r.get(2)?,
                origem: r.get(3)?,
                payload_preview: r.get(4)?,
                mensagem: r.get(5)?,
                criado_em: r.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(RespostaBase::ok("", lista))
}
