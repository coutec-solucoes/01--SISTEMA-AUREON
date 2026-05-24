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

// --- CONSULTA INDIVIDUAL ---

#[tauri::command]
pub async fn obter_regra_tributaria(
    id: String,
    estado: tauri::State<'_, EstadoApp>,
) -> Result<RespostaBase<Option<FiscalRegraTributariaResp>>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let row = conn.query_row(
        "SELECT id, pais_fiscal, tipo_operacao, uf_origem, uf_destino,
                ncm_id, cfop_id, cst_csosn_id, iva_id,
                aliquota_icms_escala6, aliquota_pis_escala6, aliquota_cofins_escala6,
                aliquota_iva_escala6, reducao_base_escala6, ativo
         FROM fiscal_regras_tributarias_cache WHERE id = ?1",
        rusqlite::params![id],
        |r| {
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
        },
    );

    match row {
        Ok(regra) => Ok(RespostaBase::ok("Regra tributaria encontrada.", Some(regra))),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            Ok(RespostaBase::ok("Regra tributaria nao encontrada.", None))
        }
        Err(e) => Err(e.to_string()),
    }
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
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

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

// ============================================================
// FASE 16 BLOCO 3 — ESPELHO FISCAL (PREVIEW TÉCNICO)
// PROIBIDO: emissão, XML, DANFE, KuDE, QR fiscal, transmissão,
//           float/f64/double, numeração oficial, certificado.
// ============================================================

// --- VALIDAÇÃO CADASTRAL FISCAL ---

#[tauri::command]
pub async fn validar_dados_cadastrais_fiscais(
    venda_id: String,
    estado: tauri::State<'_, EstadoApp>,
) -> Result<RespostaBase<ValidacaoFiscalResp>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let mut itens: Vec<ValidacaoFiscalItemResp> = Vec::new();
    let mut total_erros: i32 = 0;
    let mut total_avisos: i32 = 0;

    // 1. Valida configuração da empresa
    let empresa_row: Result<(String, String, String), _> = conn.query_row(
        "SELECT pais_fiscal, ambiente, forma_emissao FROM fiscal_empresa_cache LIMIT 1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    );

    let (pais_fiscal, ambiente) = match empresa_row {
        Ok((pais, amb, _)) => {
            if pais != "BR" && pais != "PY" {
                itens.push(ValidacaoFiscalItemResp {
                    entidade: "empresa".into(),
                    nivel: "ERRO".into(),
                    mensagem: format!("pais_fiscal invalido: {}", pais),
                });
                total_erros += 1;
            } else {
                itens.push(ValidacaoFiscalItemResp {
                    entidade: "empresa".into(),
                    nivel: "OK".into(),
                    mensagem: format!("Empresa fiscal configurada: pais={} ambiente={}", pais, amb),
                });
            }
            (pais, amb)
        }
        Err(_) => {
            itens.push(ValidacaoFiscalItemResp {
                entidade: "empresa".into(),
                nivel: "ERRO".into(),
                mensagem: "fiscal_empresa_cache nao configurado.".into(),
            });
            total_erros += 1;
            ("".into(), "".into())
        }
    };

    // 2. Valida produtos da venda
    let mut stmt_itens = conn
        .prepare(
            "SELECT vi.id, vi.produto_id, vi.descricao_produto, vi.cancelado,
                    p.ncm_id, p.iva_id, p.cst_csosn_id, p.cfop_padrao_id
             FROM venda_itens vi
             LEFT JOIN produtos_cache p ON p.id = vi.produto_id
             WHERE vi.venda_id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let rows: Vec<(String, String, String, bool, Option<String>, Option<String>, Option<String>, Option<String>)> = stmt_itens
        .query_map(rusqlite::params![venda_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i32>(3)? == 1,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, Option<String>>(6)?,
                r.get::<_, Option<String>>(7)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    if rows.is_empty() {
        itens.push(ValidacaoFiscalItemResp {
            entidade: "venda".into(),
            nivel: "ERRO".into(),
            mensagem: format!("Venda {} nao encontrada ou sem itens.", venda_id),
        });
        total_erros += 1;
    }

    for (item_id, prod_id, desc, cancelado, ncm_id, iva_id, cst_id, cfop_id) in &rows {
        if *cancelado {
            continue;
        }
        let entidade = format!("produto:{}", prod_id);
        if pais_fiscal == "BR" {
            if ncm_id.is_none() {
                itens.push(ValidacaoFiscalItemResp {
                    entidade: entidade.clone(),
                    nivel: "AVISO".into(),
                    mensagem: format!("Item '{}' sem NCM vinculado (item_id={}).", desc, item_id),
                });
                total_avisos += 1;
            }
            if cfop_id.is_none() {
                itens.push(ValidacaoFiscalItemResp {
                    entidade: entidade.clone(),
                    nivel: "AVISO".into(),
                    mensagem: format!("Item '{}' sem CFOP padrao vinculado.", desc),
                });
                total_avisos += 1;
            }
            if cst_id.is_none() {
                itens.push(ValidacaoFiscalItemResp {
                    entidade: entidade.clone(),
                    nivel: "AVISO".into(),
                    mensagem: format!("Item '{}' sem CST/CSOSN vinculado.", desc),
                });
                total_avisos += 1;
            }
            if ncm_id.is_some() && cfop_id.is_some() {
                itens.push(ValidacaoFiscalItemResp {
                    entidade: entidade.clone(),
                    nivel: "OK".into(),
                    mensagem: format!("Item '{}' com NCM e CFOP para BR.", desc),
                });
            }
        } else if pais_fiscal == "PY" {
            if iva_id.is_none() {
                itens.push(ValidacaoFiscalItemResp {
                    entidade: entidade.clone(),
                    nivel: "AVISO".into(),
                    mensagem: format!("Item '{}' sem IVA vinculado (item_id={}).", desc, item_id),
                });
                total_avisos += 1;
            } else {
                itens.push(ValidacaoFiscalItemResp {
                    entidade: entidade.clone(),
                    nivel: "OK".into(),
                    mensagem: format!("Item '{}' com IVA para PY.", desc),
                });
            }
        }
    }

    let valido = total_erros == 0;

    // Registra log
    let log_id = Uuid::new_v4().to_string();
    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let _ = conn.execute(
        "INSERT INTO fiscal_eventos_logs (id, venda_id, tipo_evento, origem, payload_preview, mensagem, criado_em)
         VALUES (?1, ?2, 'VALIDACAO_FISCAL_EXECUTADA', 'PDV', NULL, ?3, ?4)",
        rusqlite::params![
            log_id,
            venda_id,
            format!("erros={} avisos={} valido={}", total_erros, total_avisos, valido),
            agora
        ],
    );

    Ok(RespostaBase::ok(
        if valido { "Validacao fiscal concluida sem erros." } else { "Validacao fiscal com erros." },
        ValidacaoFiscalResp {
            valido,
            pais_fiscal: if pais_fiscal.is_empty() { None } else { Some(pais_fiscal) },
            ambiente: if ambiente.is_empty() { None } else { Some(ambiente) },
            total_erros,
            total_avisos,
            itens,
        },
    ))
}

// --- CÁLCULO DO ESPELHO FISCAL ---

#[tauri::command]
pub async fn calcular_espelho_fiscal_venda(
    req: CalcularEspelhoFiscalVendaReq,
    estado: tauri::State<'_, EstadoApp>,
) -> Result<RespostaBase<EspelhoFiscalVendaResp>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // 1. Carrega configuração fiscal da empresa
    let empresa_row: Result<(String, String, String), _> = conn.query_row(
        "SELECT pais_fiscal, ambiente, forma_emissao FROM fiscal_empresa_cache LIMIT 1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    );
    let (pais_fiscal, ambiente, _) = empresa_row.map_err(|_| {
        "fiscal_empresa_cache nao configurado. Configure a empresa fiscal antes de calcular.".to_string()
    })?;

    // 2. Verifica se a venda existe
    let venda_existe: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM vendas WHERE id = ?1",
            rusqlite::params![req.venda_id],
            |r| r.get(0),
        )
        .unwrap_or(false);
    if !venda_existe {
        return Err(format!("Venda {} nao encontrada.", req.venda_id));
    }

    // 3. Carrega itens ativos da venda com dados do produto
    struct ItemDados {
        item_id: String,
        produto_id: String,
        descricao: String,
        total_item_minor: i64,
        ncm_id: Option<String>,
        iva_id: Option<String>,
        cst_csosn_id: Option<String>,
        cfop_padrao_id: Option<String>,
    }

    let mut stmt = conn
        .prepare(
            "SELECT vi.id, vi.produto_id, vi.descricao_produto, vi.total_item_minor,
                    p.ncm_id, p.iva_id, p.cst_csosn_id, p.cfop_padrao_id
             FROM venda_itens vi
             LEFT JOIN produtos_cache p ON p.id = vi.produto_id
             WHERE vi.venda_id = ?1 AND (vi.cancelado IS NULL OR vi.cancelado = 0)",
        )
        .map_err(|e| e.to_string())?;

    let itens_dados: Vec<ItemDados> = stmt
        .query_map(rusqlite::params![req.venda_id], |r| {
            Ok(ItemDados {
                item_id: r.get(0)?,
                produto_id: r.get(1)?,
                descricao: r.get(2)?,
                total_item_minor: r.get(3)?,
                ncm_id: r.get(4)?,
                iva_id: r.get(5)?,
                cst_csosn_id: r.get(6)?,
                cfop_padrao_id: r.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    // 4. Tipo de operação para busca de regra
    let tipo_operacao = req.tipo_operacao.clone().unwrap_or_else(|| "VENDA_BALCAO".to_string());

    // 5. Calcula espelho fiscal por item
    let mut itens_espelho: Vec<EspelhoFiscalItemResp> = Vec::new();
    let mut total_base_minor: i64 = 0;
    let mut total_imposto_minor: i64 = 0;
    let mut alertas: Vec<String> = Vec::new();

    for item in &itens_dados {
        let base_minor = item.total_item_minor;

        // Tenta buscar regra tributária por pais_fiscal + tipo_operacao
        let regra_row: Result<(Option<String>, Option<String>, Option<String>, Option<String>, i64), _> = conn.query_row(
            "SELECT cfop_id, cst_csosn_id, iva_id, ncm_id, aliquota_iva_escala6
             FROM fiscal_regras_tributarias_cache
             WHERE pais_fiscal = ?1 AND tipo_operacao = ?2 AND ativo = 1
             LIMIT 1",
            rusqlite::params![pais_fiscal, tipo_operacao],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get::<_, i64>(4)?)),
        );

        let (cfop_id, cst_csosn_id, iva_id, ncm_id, aliquota_escala6, origem_regra) = match regra_row {
            Ok((cfop, cst, iva, ncm, aliq)) => {
                // Prioriza IDs da regra, cai em vínculo do produto se NULL
                (
                    cfop.or_else(|| item.cfop_padrao_id.clone()),
                    cst.or_else(|| item.cst_csosn_id.clone()),
                    iva.or_else(|| item.iva_id.clone()),
                    ncm.or_else(|| item.ncm_id.clone()),
                    aliq,
                    "REGRA_TRIBUTARIA".to_string(),
                )
            }
            Err(_) => {
                // Fallback: usa apenas vínculo direto do produto
                // Para BR: sem alíquota única — alíquota = 0 como preview sem regra
                // Para PY: busca alíquota do iva_id vinculado ao produto
                let aliq_fallback = if pais_fiscal == "PY" {
                    if let Some(ref iva) = item.iva_id {
                        conn.query_row(
                            "SELECT aliquota_escala6 FROM fiscal_iva_cache WHERE id = ?1",
                            rusqlite::params![iva],
                            |r| r.get::<_, i64>(0),
                        ).unwrap_or(0)
                    } else {
                        0
                    }
                } else {
                    0
                };
                (
                    item.cfop_padrao_id.clone(),
                    item.cst_csosn_id.clone(),
                    item.iva_id.clone(),
                    item.ncm_id.clone(),
                    aliq_fallback,
                    "VINCULO_PRODUTO".to_string(),
                )
            }
        };

        // Calculo: imposto_minor = base_minor * aliquota_escala6 / 1_000_000
        // Sem float — divisão inteira
        let imposto_minor: i64 = base_minor * aliquota_escala6 / 1_000_000;

        if ncm_id.is_none() && pais_fiscal == "BR" {
            alertas.push(format!("Item '{}' sem NCM — preview parcial.", item.descricao));
        }
        if iva_id.is_none() && pais_fiscal == "PY" {
            alertas.push(format!("Item '{}' sem IVA — preview sem imposto.", item.descricao));
        }

        total_base_minor += base_minor;
        total_imposto_minor += imposto_minor;

        itens_espelho.push(EspelhoFiscalItemResp {
            venda_item_id: item.item_id.clone(),
            produto_id: item.produto_id.clone(),
            descricao_produto: item.descricao.clone(),
            ncm_id: ncm_id.clone(),
            cfop_id: cfop_id.clone(),
            cst_csosn_id: cst_csosn_id.clone(),
            iva_id: iva_id.clone(),
            base_minor,
            aliquota_escala6,
            imposto_minor,
            origem_regra: origem_regra.clone(),
        });

        // Salva preview no item (UPDATE — sem alterar preço, estoque, pagamento, financeiro)
        let item_preview_json = format!(
            "{{\"origem_regra\":\"{}\",\"aliquota_escala6\":{},\"base_minor\":{},\"imposto_minor\":{}}}",
            origem_regra, aliquota_escala6, base_minor, imposto_minor
        );
        let _ = conn.execute(
            "UPDATE venda_itens SET
                fiscal_cfop_id = ?1,
                fiscal_cst_csosn_id = ?2,
                fiscal_iva_id = ?3,
                fiscal_ncm_id = ?4,
                fiscal_base_minor = ?5,
                fiscal_aliquota_escala6 = ?6,
                fiscal_imposto_minor = ?7,
                fiscal_preview_json = ?8
             WHERE id = ?9",
            rusqlite::params![
                cfop_id, cst_csosn_id, iva_id, ncm_id,
                base_minor, aliquota_escala6, imposto_minor,
                item_preview_json,
                item.item_id
            ],
        );
    }

    // 6. Define modelo de preview e status
    let modelo_preview = if pais_fiscal == "BR" { "NFC-E_BR" } else { "SIFEN_PY" }.to_string();
    let status_preparacao = if alertas.is_empty() {
        "PREVIEW_OK"
    } else if total_imposto_minor > 0 {
        "PREVIEW_COM_ALERTAS"
    } else {
        "PREVIEW_INCOMPLETO"
    }.to_string();

    // 7. Salva preview na venda (só campos fiscais)
    let venda_preview_json = format!(
        "{{\"pais_fiscal\":\"{}\",\"ambiente\":\"{}\",\"total_base_minor\":{},\"total_imposto_minor\":{},\"itens\":{}}}",
        pais_fiscal, ambiente, total_base_minor, total_imposto_minor, itens_espelho.len()
    );
    conn.execute(
        "UPDATE vendas SET
            fiscal_pais = ?1,
            fiscal_ambiente = ?2,
            fiscal_modelo_preview = ?3,
            fiscal_status_preparacao = ?4,
            fiscal_total_base_minor = ?5,
            fiscal_total_imposto_minor = ?6,
            fiscal_preview_json = ?7,
            fiscal_calculado_em = ?8
         WHERE id = ?9",
        rusqlite::params![
            pais_fiscal, ambiente, modelo_preview, status_preparacao,
            total_base_minor, total_imposto_minor,
            venda_preview_json, agora,
            req.venda_id
        ],
    ).map_err(|e| e.to_string())?;

    // 8. Registra log fiscal técnico
    let log_id = Uuid::new_v4().to_string();
    let tipo_log = if alertas.is_empty() {
        "ESPELHO_FISCAL_CALCULADO"
    } else {
        "ESPELHO_FISCAL_COM_ALERTAS"
    };
    let _ = conn.execute(
        "INSERT INTO fiscal_eventos_logs (id, venda_id, tipo_evento, origem, payload_preview, mensagem, criado_em)
         VALUES (?1, ?2, ?3, 'PDV', ?4, ?5, ?6)",
        rusqlite::params![
            log_id,
            req.venda_id,
            tipo_log,
            venda_preview_json,
            format!("status={} base={} imposto={} alertas={}", status_preparacao, total_base_minor, total_imposto_minor, alertas.len()),
            agora
        ],
    );

    Ok(RespostaBase::ok(
        "Espelho fiscal calculado. Preview tecnico sem validade fiscal oficial.",
        EspelhoFiscalVendaResp {
            venda_id: req.venda_id,
            pais_fiscal,
            ambiente,
            modelo_preview,
            status_preparacao,
            total_base_minor,
            total_imposto_minor,
            calculado_em: agora,
            itens: itens_espelho,
            alertas,
        },
    ))
}

// --- OBTER ESPELHO FISCAL JÁ CALCULADO ---

#[tauri::command]
pub async fn obter_espelho_fiscal_venda(
    req: ObterEspelhoFiscalVendaReq,
    estado: tauri::State<'_, EstadoApp>,
) -> Result<RespostaBase<Option<EspelhoFiscalVendaResp>>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    // Lê preview salvo na venda
    let venda_row: Result<(String, String, String, String, i64, i64, Option<String>, String), _> =
        conn.query_row(
            "SELECT fiscal_pais, fiscal_ambiente, fiscal_modelo_preview, fiscal_status_preparacao,
                    fiscal_total_base_minor, fiscal_total_imposto_minor,
                    fiscal_preview_json, fiscal_calculado_em
             FROM vendas WHERE id = ?1 AND fiscal_status_preparacao IS NOT NULL",
            rusqlite::params![req.venda_id],
            |r| {
                Ok((
                    r.get::<_, Option<String>>(0)?.unwrap_or_default(),
                    r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    r.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    r.get::<_, Option<i64>>(4)?.unwrap_or(0),
                    r.get::<_, Option<i64>>(5)?.unwrap_or(0),
                    r.get::<_, Option<String>>(6)?,
                    r.get::<_, Option<String>>(7)?.unwrap_or_default(),
                ))
            },
        );

    let (pais_fiscal, ambiente, modelo_preview, status_preparacao, total_base_minor, total_imposto_minor, _, calculado_em) =
        match venda_row {
            Ok(v) => v,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                return Ok(RespostaBase::ok("Espelho fiscal nao calculado para esta venda.", None));
            }
            Err(e) => return Err(e.to_string()),
        };

    // Lê itens com preview fiscal
    let mut stmt = conn
        .prepare(
            "SELECT vi.id, vi.produto_id, vi.descricao_produto,
                    vi.fiscal_ncm_id, vi.fiscal_cfop_id, vi.fiscal_cst_csosn_id, vi.fiscal_iva_id,
                    vi.fiscal_base_minor, vi.fiscal_aliquota_escala6, vi.fiscal_imposto_minor,
                    vi.fiscal_preview_json
             FROM venda_itens vi
             WHERE vi.venda_id = ?1 AND (vi.cancelado IS NULL OR vi.cancelado = 0)
               AND vi.fiscal_base_minor IS NOT NULL",
        )
        .map_err(|e| e.to_string())?;

    let itens: Vec<EspelhoFiscalItemResp> = stmt
        .query_map(rusqlite::params![req.venda_id], |r| {
            let origem_regra = r.get::<_, Option<String>>(10)?
                .unwrap_or_else(|| "VINCULO_PRODUTO".to_string());
            Ok(EspelhoFiscalItemResp {
                venda_item_id: r.get(0)?,
                produto_id: r.get(1)?,
                descricao_produto: r.get(2)?,
                ncm_id: r.get(3)?,
                cfop_id: r.get(4)?,
                cst_csosn_id: r.get(5)?,
                iva_id: r.get(6)?,
                base_minor: r.get::<_, Option<i64>>(7)?.unwrap_or(0),
                aliquota_escala6: r.get::<_, Option<i64>>(8)?.unwrap_or(0),
                imposto_minor: r.get::<_, Option<i64>>(9)?.unwrap_or(0),
                origem_regra,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(RespostaBase::ok(
        "Espelho fiscal obtido.",
        Some(EspelhoFiscalVendaResp {
            venda_id: req.venda_id,
            pais_fiscal,
            ambiente,
            modelo_preview,
            status_preparacao,
            total_base_minor,
            total_imposto_minor,
            calculado_em,
            itens,
            alertas: vec![],
        }),
    ))
}

// --- LIMPAR ESPELHO FISCAL ---

#[tauri::command]
pub async fn limpar_espelho_fiscal_venda(
    venda_id: String,
    estado: tauri::State<'_, EstadoApp>,
) -> Result<RespostaBase<String>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Limpa somente campos fiscais de preview — não altera total, preço, estoque ou pagamento
    conn.execute(
        "UPDATE vendas SET
            fiscal_pais = NULL,
            fiscal_ambiente = NULL,
            fiscal_modelo_preview = NULL,
            fiscal_status_preparacao = NULL,
            fiscal_total_base_minor = 0,
            fiscal_total_imposto_minor = 0,
            fiscal_preview_json = NULL,
            fiscal_calculado_em = NULL
         WHERE id = ?1",
        rusqlite::params![venda_id],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE venda_itens SET
            fiscal_cfop_id = NULL,
            fiscal_cst_csosn_id = NULL,
            fiscal_iva_id = NULL,
            fiscal_ncm_id = NULL,
            fiscal_base_minor = 0,
            fiscal_aliquota_escala6 = 0,
            fiscal_imposto_minor = 0,
            fiscal_preview_json = NULL
         WHERE venda_id = ?1",
        rusqlite::params![venda_id],
    ).map_err(|e| e.to_string())?;

    // Log técnico
    let log_id = Uuid::new_v4().to_string();
    let _ = conn.execute(
        "INSERT INTO fiscal_eventos_logs (id, venda_id, tipo_evento, origem, payload_preview, mensagem, criado_em)
         VALUES (?1, ?2, 'ESPELHO_FISCAL_LIMPO', 'PDV', NULL, 'Preview fiscal resetado.', ?3)",
        rusqlite::params![log_id, venda_id, agora],
    );

    Ok(RespostaBase::ok("Espelho fiscal limpo. Nenhum dado fiscal oficial foi alterado.", "OK".to_string()))
}

