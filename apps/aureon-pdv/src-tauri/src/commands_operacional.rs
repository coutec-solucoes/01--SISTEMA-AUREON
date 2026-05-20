use std::sync::Arc;
use std::sync::Mutex;
use tauri::State;
use rusqlite::{params, Connection};
use aureon_core::dtos::*;
use aureon_core::RespostaBase;
use uuid::Uuid;
use chrono::Utc;
use crate::commands_caixa::outbox_inserir;

// --- Caixa: Movimentações ---

fn registrar_movimentacao_interna(
    conn: &mut Connection,
    req: CaixaMovimentacaoReq,
) -> Result<CaixaMovimentacaoResp, String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // 1. Validar se a sessão está aberta
    let status_sessao: String = tx
        .query_row(
            "SELECT status FROM sessoes_caixa WHERE id = ?1",
            params![req.sessao_caixa_id],
            |r| r.get(0),
        )
        .map_err(|_| "Sessão de caixa não encontrada".to_string())?;

    if status_sessao != "ABERTO" {
        return Err("A sessão de caixa não está aberta".to_string());
    }

    if req.valor_minor <= 0 {
        return Err("O valor deve ser maior que zero".to_string());
    }

    // 2. Inserir a movimentação
    let id = Uuid::new_v4().to_string();
    let agora = Utc::now().to_rfc3339();

    tx.execute(
        "INSERT INTO caixa_movimentacoes (
            id, sessao_caixa_id, usuario_id, tipo_movimentacao, moeda_codigo,
            valor_minor, motivo, funcionario_id, supervisor_id, autorizacao_id, criado_em
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            id, req.sessao_caixa_id, req.usuario_id, req.tipo_movimentacao, req.moeda_codigo,
            req.valor_minor, req.motivo, req.funcionario_id, req.supervisor_id, req.autorizacao_id, agora
        ],
    ).map_err(|e| e.to_string())?;

    let resp = CaixaMovimentacaoResp {
        id: id.clone(),
        sessao_caixa_id: req.sessao_caixa_id.clone(),
        usuario_id: req.usuario_id,
        tipo_movimentacao: req.tipo_movimentacao.clone(),
        moeda_codigo: req.moeda_codigo,
        valor_minor: req.valor_minor,
        motivo: req.motivo,
        funcionario_id: req.funcionario_id,
        supervisor_id: req.supervisor_id,
        autorizacao_id: req.autorizacao_id,
        cancelado: false,
        cancelado_em: None,
        usuario_cancelamento_id: None,
        motivo_cancelamento: None,
        criado_em: agora.clone(),
    };

    // 3. Outbox
    let evento_tipo = match req.tipo_movimentacao.as_str() {
        "SANGRIA" => "CAIXA_SANGRIA_REGISTRADA",
        "SUPRIMENTO" => "CAIXA_SUPRIMENTO_REGISTRADO",
        "VALE_FUNCIONARIO" => "CAIXA_VALE_FUNCIONARIO_REGISTRADO",
        _ => "CAIXA_MOVIMENTACAO_REGISTRADA"
    };

    outbox_inserir(
        &tx,
        evento_tipo,
        serde_json::to_value(&resp).unwrap(),
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(resp)
}

#[tauri::command]
pub async fn registrar_suprimento(
    db: State<'_, Arc<Mutex<Connection>>>,
    dto: CaixaMovimentacaoReq,
) -> Result<RespostaBase<CaixaMovimentacaoResp>, String> {
    let mut conn = db.lock().unwrap();
    let mut req = dto;
    req.tipo_movimentacao = "SUPRIMENTO".to_string();
    
    match registrar_movimentacao_interna(&mut conn, req) {
        Ok(resp) => Ok(RespostaBase::ok("", resp)),
        Err(e) => Ok(RespostaBase::falha_manual(e, "ERR_OP", "")),
    }
}

#[tauri::command]
pub async fn registrar_sangria(
    db: State<'_, Arc<Mutex<Connection>>>,
    dto: CaixaMovimentacaoReq,
) -> Result<RespostaBase<CaixaMovimentacaoResp>, String> {
    let mut conn = db.lock().unwrap();
    let mut req = dto;
    req.tipo_movimentacao = "SANGRIA".to_string();
    if req.motivo.is_none() || req.motivo.as_ref().unwrap().trim().is_empty() {
        return Ok(RespostaBase::falha_manual("Motivo é obrigatório para sangria", "ERR_OP", ""));
    }
    match registrar_movimentacao_interna(&mut conn, req) {
        Ok(resp) => Ok(RespostaBase::ok("", resp)),
        Err(e) => Ok(RespostaBase::falha_manual(e, "ERR_OP", "")),
    }
}

#[tauri::command]
pub async fn registrar_vale_funcionario(
    db: State<'_, Arc<Mutex<Connection>>>,
    dto: CaixaMovimentacaoReq,
) -> Result<RespostaBase<CaixaMovimentacaoResp>, String> {
    let mut conn = db.lock().unwrap();
    let mut req = dto;
    req.tipo_movimentacao = "VALE_FUNCIONARIO".to_string();
    if req.motivo.is_none() || req.motivo.as_ref().unwrap().trim().is_empty() {
        return Ok(RespostaBase::falha_manual("Motivo é obrigatório para vale funcionário", "ERR_OP", ""));
    }
    match registrar_movimentacao_interna(&mut conn, req) {
        Ok(resp) => Ok(RespostaBase::ok("", resp)),
        Err(e) => Ok(RespostaBase::falha_manual(e, "ERR_OP", "")),
    }
}

#[tauri::command]
pub async fn cancelar_movimentacao_caixa(
    db: State<'_, Arc<Mutex<Connection>>>,
    dto: CancelarMovimentacaoReq,
) -> Result<RespostaBase<bool>, String> {
    let mut conn = db.lock().unwrap();
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    if dto.motivo_cancelamento.trim().is_empty() {
        return Ok(RespostaBase::falha_manual("Motivo do cancelamento é obrigatório", "ERR_OP", ""));
    }

    let id = dto.movimentacao_id.clone();
    let agora = Utc::now().to_rfc3339();

    let rows = tx.execute(
        "UPDATE caixa_movimentacoes 
         SET cancelado = 1, cancelado_em = ?1, usuario_cancelamento_id = ?2, motivo_cancelamento = ?3,
             supervisor_id = COALESCE(supervisor_id, ?4), autorizacao_id = COALESCE(autorizacao_id, ?5)
         WHERE id = ?6 AND cancelado = 0",
        params![agora, dto.usuario_cancelamento_id, dto.motivo_cancelamento, dto.supervisor_id, dto.autorizacao_id, id],
    ).map_err(|e| e.to_string())?;

    if rows == 0 {
        return Ok(RespostaBase::falha_manual("Movimentação não encontrada ou já cancelada", "ERR_OP", ""));
    }

    outbox_inserir(
        &tx,
        "CAIXA_MOVIMENTACAO_CANCELADA",
        serde_json::json!({"id": id, "cancelado_em": agora, "motivo": dto.motivo_cancelamento}),
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(RespostaBase::ok("", true))
}

#[tauri::command]
pub async fn listar_movimentacoes_caixa(
    db: State<'_, Arc<Mutex<Connection>>>,
    sessao_caixa_id: String,
) -> Result<RespostaBase<Vec<CaixaMovimentacaoResp>>, String> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, sessao_caixa_id, usuario_id, tipo_movimentacao, moeda_codigo, valor_minor,
                motivo, funcionario_id, supervisor_id, autorizacao_id, cancelado, cancelado_em,
                usuario_cancelamento_id, motivo_cancelamento, criado_em
         FROM caixa_movimentacoes WHERE sessao_caixa_id = ?1 ORDER BY criado_em DESC"
    ).map_err(|e| e.to_string())?;

    let iter = stmt.query_map(params![sessao_caixa_id], |row| {
        Ok(CaixaMovimentacaoResp {
            id: row.get(0)?,
            sessao_caixa_id: row.get(1)?,
            usuario_id: row.get(2)?,
            tipo_movimentacao: row.get(3)?,
            moeda_codigo: row.get(4)?,
            valor_minor: row.get(5)?,
            motivo: row.get(6)?,
            funcionario_id: row.get(7)?,
            supervisor_id: row.get(8)?,
            autorizacao_id: row.get(9)?,
            cancelado: row.get(10)?,
            cancelado_em: row.get(11)?,
            usuario_cancelamento_id: row.get(12)?,
            motivo_cancelamento: row.get(13)?,
            criado_em: row.get(14)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut lista = Vec::new();
    for i in iter {
        if let Ok(m) = i { lista.push(m); }
    }
    Ok(RespostaBase::ok("", lista))
}

#[tauri::command]
pub async fn obter_resumo_caixa(
    db: State<'_, Arc<Mutex<Connection>>>,
    sessao_caixa_id: String,
) -> Result<RespostaBase<serde_json::Value>, String> {
    let conn = db.lock().unwrap();
    
    // Simplificado para retornar um map de totais por moeda para exibição
    let mut stmt = conn.prepare(
        "SELECT moeda_codigo, 
                SUM(CASE WHEN tipo_movimentacao = 'SUPRIMENTO' THEN valor_minor ELSE 0 END) as total_suprimentos,
                SUM(CASE WHEN tipo_movimentacao IN ('SANGRIA', 'VALE_FUNCIONARIO') THEN valor_minor ELSE 0 END) as total_sangrias
         FROM caixa_movimentacoes 
         WHERE sessao_caixa_id = ?1 AND cancelado = 0
         GROUP BY moeda_codigo"
    ).map_err(|e| e.to_string())?;

    let mut map_moedas = serde_json::Map::new();

    let _ = stmt.query_map(params![sessao_caixa_id], |row| {
        let moeda: String = row.get(0)?;
        let suprimentos: i64 = row.get(1)?;
        let sangrias: i64 = row.get(2)?;
        map_moedas.insert(moeda, serde_json::json!({
            "total_suprimentos_minor": suprimentos,
            "total_sangrias_minor": sangrias
        }));
        Ok(())
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>();

    Ok(RespostaBase::ok("", serde_json::Value::Object(map_moedas)))
}

// --- Supervisor ---

#[tauri::command]
pub async fn solicitar_autorizacao_supervisor(
    db: State<'_, Arc<Mutex<Connection>>>,
    terminal_id: String,
    dto: SolicitarAutorizacaoReq,
) -> Result<RespostaBase<AutorizacaoResp>, String> {
    // A validação real usaria um cache local e hash. Como não temos supervisores_cache populado ainda,
    // vamos simular a validação falhando apenas se pin for vazio.
    let mut conn = db.lock().unwrap();
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    // Simulação: "1234" aprova, qualquer outro nega
    let aprovado = dto.pin_supervisor == "1234";
    // Como não temos tabela de supervisores real, fingimos que o ID do supervisor é algo fixo
    let supervisor_id = if aprovado { "SUP-001".to_string() } else { "SUP-UNKNOWN".to_string() };

    let id = Uuid::new_v4().to_string();
    let agora = Utc::now().to_rfc3339();

    tx.execute(
        "INSERT INTO supervisor_autorizacoes_local (
            id, operacao, usuario_solicitante_id, supervisor_id, motivo, aprovado,
            criado_em, terminal_id, sessao_caixa_id, entidade_tipo, entidade_id
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            id, dto.operacao, dto.usuario_solicitante_id, supervisor_id, dto.motivo, aprovado,
            agora, terminal_id, dto.sessao_caixa_id, dto.entidade_tipo, dto.entidade_id
        ],
    ).map_err(|e| e.to_string())?;

    let resp = AutorizacaoResp {
        id: id.clone(),
        operacao: dto.operacao,
        usuario_solicitante_id: dto.usuario_solicitante_id,
        supervisor_id,
        aprovado,
        motivo: dto.motivo,
        criado_em: agora.clone(),
    };

    let evento = if aprovado { "SUPERVISOR_AUTORIZACAO_APROVADA" } else { "SUPERVISOR_AUTORIZACAO_NEGADA" };
    outbox_inserir(
        &tx,
        evento,
        serde_json::to_value(&resp).unwrap(),
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    if aprovado {
        Ok(RespostaBase::ok("", resp))
    } else {
        Ok(RespostaBase::falha_manual("PIN inválido ou supervisor não autorizado", "ERR_OP", ""))
    }
}

#[tauri::command]
pub async fn validar_autorizacao_supervisor() -> Result<RespostaBase<bool>, String> {
    Ok(RespostaBase::ok("", true))
}

#[tauri::command]
pub async fn listar_autorizacoes_local() -> Result<RespostaBase<Vec<AutorizacaoResp>>, String> {
    // Stub
    Ok(RespostaBase::ok("", vec![]))
}

// --- Historico e Reimpressão ---

#[tauri::command]
pub async fn listar_vendas_pdv(
    db: State<'_, Arc<Mutex<Connection>>>,
    limite: Option<i32>,
) -> Result<RespostaBase<Vec<VendaResumoResp>>, String> {
    let conn = db.lock().unwrap();
    let lim = limite.unwrap_or(50);

    let mut stmt = conn.prepare(
        "SELECT id, numero_venda, status, tipo_venda, subtotal_minor, desconto_total_minor, 
                acrescimo_total_minor, total_minor 
         FROM vendas ORDER BY criado_em DESC LIMIT ?1"
    ).map_err(|e| e.to_string())?;

    let iter = stmt.query_map([lim], |row| {
        Ok(VendaResumoResp {
            id: row.get(0)?,
            numero_venda: row.get(1)?,
            status: row.get(2)?,
            tipo_venda: row.get(3)?,
            subtotal_minor: row.get(4)?,
            desconto_total_minor: row.get(5)?,
            acrescimo_total_minor: row.get(6)?,
            total_minor: row.get(7)?,
            total_itens: 0,
        })
    }).map_err(|e| e.to_string())?;

    let mut lista = Vec::new();
    for i in iter {
        if let Ok(v) = i { lista.push(v); }
    }
    Ok(RespostaBase::ok("", lista))
}

#[tauri::command]
pub async fn buscar_venda_por_numero(
    db: State<'_, Arc<Mutex<Connection>>>,
    numero: i64,
) -> Result<RespostaBase<VendaResumoResp>, String> {
    let conn = db.lock().unwrap();
    let res = conn.query_row(
        "SELECT id, numero_venda, status, tipo_venda, subtotal_minor, desconto_total_minor, 
                acrescimo_total_minor, total_minor 
         FROM vendas WHERE numero_venda = ?1 LIMIT 1",
        [numero],
        |row| {
            Ok(VendaResumoResp {
                id: row.get(0)?,
                numero_venda: row.get(1)?,
                status: row.get(2)?,
                tipo_venda: row.get(3)?,
                subtotal_minor: row.get(4)?,
                desconto_total_minor: row.get(5)?,
                acrescimo_total_minor: row.get(6)?,
                total_minor: row.get(7)?,
                total_itens: 0,
            })
        }
    );

    match res {
        Ok(v) => Ok(RespostaBase::ok("", v)),
        Err(_) => Ok(RespostaBase::falha_manual("Venda não encontrada", "ERR_OP", "")),
    }
}

#[tauri::command]
pub async fn gerar_comprovante_nao_fiscal(
    db: State<'_, Arc<Mutex<Connection>>>,
    venda_id: String,
) -> Result<RespostaBase<String>, String> {
    let conn = db.lock().unwrap();
    
    // Obter cabeçalho
    let total_minor: i64 = conn.query_row(
        "SELECT total_minor FROM vendas WHERE id = ?1",
        params![venda_id],
        |r| r.get(0)
    ).map_err(|_| "Venda não encontrada".to_string())?;

    let txt = format!(
        "================================================\n\
         COMPROVANTE NÃO FISCAL\n\
         ================================================\n\
         VENDA ID: {}\n\
         TOTAL: BRL {:.2}\n\
         ================================================\n\
         * SEM VALOR FISCAL *\n\
         ================================================",
         &venda_id[0..8],
         (total_minor as f64) / 100.0
    );

    Ok(RespostaBase::ok("", txt))
}

#[tauri::command]
pub async fn registrar_reimpressao_comprovante(
    db: State<'_, Arc<Mutex<Connection>>>,
    req: ReimpressaoReq,
) -> Result<RespostaBase<bool>, String> {
    let mut conn = db.lock().unwrap();
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    if req.motivo.is_none() || req.motivo.as_ref().unwrap().trim().is_empty() {
        return Ok(RespostaBase::falha_manual("Motivo da reimpressão é obrigatório", "ERR_OP", ""));
    }

    let id = Uuid::new_v4().to_string();
    let agora = Utc::now().to_rfc3339();

    tx.execute(
        "INSERT INTO vendas_reimpressoes (id, venda_id, usuario_id, motivo, supervisor_id, criado_em)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, req.venda_id, req.usuario_id, req.motivo, req.supervisor_id, agora],
    ).map_err(|e| e.to_string())?;

    outbox_inserir(
        &tx,
        "COMPROVANTE_REIMPRESSO",
        serde_json::json!({"id": id, "venda_id": req.venda_id, "motivo": req.motivo}),
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(RespostaBase::ok("", true))
}
