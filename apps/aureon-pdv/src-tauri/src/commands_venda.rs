use tauri::State;
use tracing::{info, error, warn};
use uuid::Uuid;
use serde_json::json;

use aureon_core::{dtos::*, RespostaBase};
use crate::estado::EstadoApp;
use crate::commands_caixa::outbox_inserir;
use crate::commands_estoque::processar_estorno_venda;

// ================================================================
// Command: iniciar_venda
// numero_venda = NULL ate finalizacao (regra financeira)
// ================================================================

#[tauri::command]
pub async fn iniciar_venda(
    sessao_caixa_id: String,
    usuario_id: String,
    tipo_venda: String,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<VendaResumoResp>, String> {
    info!(
        componente = "aureon-pdv::commands_venda",
        sessao_caixa_id = %sessao_caixa_id,
        tipo_venda = %tipo_venda,
        "Chamada: iniciar_venda"
    );

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let sessao_ok: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM sessoes_caixa WHERE id = ?1 AND status = 'ABERTO'",
        rusqlite::params![&sessao_caixa_id],
        |row| row.get(0),
    ).unwrap_or(false);

    if !sessao_ok {
        return Err("Sessao de caixa nao encontrada ou nao esta aberta.".into());
    }

    let venda_id = Uuid::new_v4().to_string();
    let agora = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // numero_venda = NULL ate finalizacao — regra financeira
    tx.execute(
        "INSERT INTO vendas (id, numero_venda, sessao_caixa_id, usuario_id, status, tipo_venda,
                             subtotal_minor, desconto_total_minor, acrescimo_total_minor,
                             total_minor, criado_em, atualizado_em)
         VALUES (?1, NULL, ?2, ?3, 'EM_ANDAMENTO', ?4, 0, 0, 0, 0, ?5, ?5)",
        rusqlite::params![
            &venda_id, &sessao_caixa_id, &usuario_id, &tipo_venda, &agora
        ],
    ).map_err(|e| {
        error!(componente = "aureon-pdv::commands_venda", erro = %e, "Erro ao inserir venda");
        e.to_string()
    })?;

    // Evento outbox: VENDA_INICIADA
    outbox_inserir(
        &tx,
        "VENDA_INICIADA",
        json!({
            "venda_id": &venda_id,
            "sessao_caixa_id": &sessao_caixa_id,
            "usuario_id": &usuario_id,
            "tipo_venda": &tipo_venda,
            "iniciado_em": &agora
        }),
    )?;

    tx.commit().map_err(|e| e.to_string())?;

    let resp = VendaResumoResp {
        id: venda_id,
        numero_venda: None,   // NULL — atribuido apenas na finalizacao
        status: "EM_ANDAMENTO".into(),
        tipo_venda,
        subtotal_minor: 0,
        desconto_total_minor: 0,
        acrescimo_total_minor: 0,
        total_minor: 0,
        total_itens: 0,
    };

    Ok(RespostaBase::ok("Venda iniciada", resp))
}

// ================================================================
// Command: buscar_produto_pdv
// Retorna preco_venda_minor (centavos) do cache local
// ================================================================

#[tauri::command]
pub async fn buscar_produto_pdv(
    codigo: String,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<Option<ProdutoPdvResp>>, String> {
    info!(
        componente = "aureon-pdv::commands_venda",
        codigo = %codigo,
        "Chamada: buscar_produto_pdv"
    );

    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    // preco_venda no cache e REAL — convertemos para minor unit (centavos)
    // usando ROUND(preco * 100) diretamente no SQLite para evitar float em Rust
    let produto = conn.query_row(
        "SELECT p.produto_id, p.codigo, p.codigo_barras, p.nome, p.unidade_medida, p.ativo,
                CAST(ROUND(COALESCE(pr.preco_venda, 0.0) * 100) AS INTEGER) as preco_venda_minor
         FROM produtos_cache p
         LEFT JOIN produtos_precos_cache pr ON pr.produto_id = p.produto_id
         WHERE p.ativo = 1 AND (UPPER(p.codigo) = UPPER(?1) OR p.codigo_barras = ?1)
         LIMIT 1",
        rusqlite::params![&codigo],
        |row| {
            Ok(ProdutoPdvResp {
                produto_id:       row.get(0)?,
                codigo:           row.get(1)?,
                codigo_barras:    row.get(2)?,
                nome:             row.get(3)?,
                unidade_medida:   row.get(4)?,
                ativo:            row.get::<_, i32>(5)? == 1,
                preco_venda_minor: row.get(6)?,
            })
        },
    ).ok();

    Ok(RespostaBase::ok("Produto encontrado", produto))
}

// ================================================================
// Command: adicionar_item_venda
// Todos os valores em minor unit (i64). Calculos inteiros puros.
// ================================================================

#[tauri::command]
pub async fn adicionar_item_venda(
    dto: AdicionarItemReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<VendaResumoResp>, String> {
    info!(
        componente = "aureon-pdv::commands_venda",
        venda_id = %dto.venda_id,
        produto_id = %dto.produto_id,
        quantidade = dto.quantidade_escala3,
        "Chamada: adicionar_item_venda"
    );

    if dto.quantidade_escala3 <= 0 {
        return Err("Quantidade deve ser maior que zero.".into());
    }
    if dto.preco_unitario_minor < 0 {
        return Err("Preco unitario nao pode ser negativo.".into());
    }
    if dto.desconto_item_minor < 0 {
        return Err("Desconto nao pode ser negativo.".into());
    }

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let venda_ok: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM vendas WHERE id = ?1 AND status = 'EM_ANDAMENTO'",
        rusqlite::params![&dto.venda_id],
        |row| row.get(0),
    ).unwrap_or(false);

    if !venda_ok {
        return Err("Venda nao encontrada ou nao esta em andamento.".into());
    }

    let item_id = Uuid::new_v4().to_string();
    let agora = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Calculo inteiro: (quantidade * preco) / 1000 - desconto
    // Divisao por 1000 porque quantidade esta em escala 3
    let bruto = dto.quantidade_escala3
        .checked_mul(dto.preco_unitario_minor)
        .ok_or("Overflow no calculo de preco")?
        / 1000;
    let total_item = (bruto - dto.desconto_item_minor).max(0);

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO venda_itens (id, venda_id, produto_id, descricao_produto, codigo_produto,
                                  codigo_barras, quantidade_escala3, preco_unitario_minor,
                                  desconto_item_minor, acrescimo_item_minor, total_item_minor,
                                  cancelado, criado_em)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, 0, ?11)",
        rusqlite::params![
            &item_id,
            &dto.venda_id,
            &dto.produto_id,
            &dto.descricao_produto,
            &dto.codigo_produto,
            &dto.codigo_barras,
            dto.quantidade_escala3,
            dto.preco_unitario_minor,
            dto.desconto_item_minor,
            total_item,
            &agora
        ],
    ).map_err(|e| {
        error!(componente = "aureon-pdv::commands_venda", erro = %e, "Erro ao inserir item");
        e.to_string()
    })?;

    // Recalcular totais via SUM inteiro no banco — fonte de verdade
    tx.execute(
        "UPDATE vendas SET
            subtotal_minor = (SELECT COALESCE(SUM(total_item_minor), 0)
                              FROM venda_itens WHERE venda_id = ?1 AND cancelado = 0),
            total_minor    = (SELECT COALESCE(SUM(total_item_minor), 0)
                              FROM venda_itens WHERE venda_id = ?1 AND cancelado = 0)
                             + acrescimo_total_minor - desconto_total_minor,
            atualizado_em  = ?2
         WHERE id = ?1",
        rusqlite::params![&dto.venda_id, &agora],
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    obter_resumo_venda(&conn, &dto.venda_id)
}

// ================================================================
// Command: cancelar_item_venda
// Requer usuario, motivo e opcionalmente supervisor
// ================================================================

#[tauri::command]
pub async fn cancelar_item_venda(
    dto: CancelarItemReq,
    req_sup: Option<aureon_core::dtos::AutorizarOperacaoSupervisorReq>,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<VendaResumoResp>, String> {
    info!(
        componente = "aureon-pdv::commands_venda",
        item_id = %dto.item_id,
        usuario = %dto.usuario_cancelamento_id,
        "Chamada: cancelar_item_venda"
    );

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    // GUARDA DE PERMISSÃO / SUPERVISOR
    crate::commands_seguranca::garantir_permissao_ou_supervisor(
        &conn, 
        "ITEM_CANCELAR", 
        Some(&dto.item_id), 
        Some("commands_venda::cancelar_item_venda"), 
        Some(&dto.motivo_cancelamento), 
        req_sup.as_ref()
    )?;

    let venda_id: String = conn.query_row(
        "SELECT venda_id FROM venda_itens WHERE id = ?1 AND cancelado = 0",
        rusqlite::params![&dto.item_id],
        |row| row.get(0),
    ).map_err(|_| "Item nao encontrado ou ja cancelado.".to_string())?;

    let agora = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE venda_itens
         SET cancelado = 1,
             cancelado_em = ?1,
             usuario_cancelamento_id = ?2,
             motivo_cancelamento = ?3,
             supervisor_id = ?4,
             autorizacao_id = ?5
         WHERE id = ?6",
        rusqlite::params![
            &agora,
            &dto.usuario_cancelamento_id,
            &dto.motivo_cancelamento,
            &dto.supervisor_id,
            &dto.autorizacao_id,
            &dto.item_id
        ],
    ).map_err(|e| e.to_string())?;

    // Recalcular totais
    tx.execute(
        "UPDATE vendas SET
            subtotal_minor = (SELECT COALESCE(SUM(total_item_minor), 0)
                              FROM venda_itens WHERE venda_id = ?1 AND cancelado = 0),
            total_minor    = (SELECT COALESCE(SUM(total_item_minor), 0)
                              FROM venda_itens WHERE venda_id = ?1 AND cancelado = 0)
                             + acrescimo_total_minor - desconto_total_minor,
            atualizado_em  = ?2
         WHERE id = ?1",
        rusqlite::params![&venda_id, &agora],
    ).map_err(|e| e.to_string())?;

    // Evento outbox: ITEM_CANCELADO
    outbox_inserir(
        &tx,
        "ITEM_CANCELADO",
        json!({
            "item_id": &dto.item_id,
            "venda_id": &venda_id,
            "usuario_cancelamento_id": &dto.usuario_cancelamento_id,
            "motivo": &dto.motivo_cancelamento,
            "supervisor_id": &dto.supervisor_id,
            "cancelado_em": &agora
        }),
    )?;

    tx.commit().map_err(|e| e.to_string())?;

    warn!(
        componente = "aureon-pdv::commands_venda",
        item_id = %dto.item_id,
        venda_id = %venda_id,
        usuario = %dto.usuario_cancelamento_id,
        "Item cancelado"
    );

    obter_resumo_venda(&conn, &venda_id)
}

// ================================================================
// Command: cancelar_venda
// Requer usuario, motivo e opcionalmente supervisor
// ================================================================

#[tauri::command]
pub async fn cancelar_venda(
    dto: CancelarVendaReq,
    req_sup: Option<aureon_core::dtos::AutorizarOperacaoSupervisorReq>,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<bool>, String> {
    info!(
        componente = "aureon-pdv::commands_venda",
        venda_id = %dto.venda_id,
        usuario = %dto.usuario_cancelamento_id,
        "Chamada: cancelar_venda"
    );

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    // GUARDA DE PERMISSÃO / SUPERVISOR
    crate::commands_seguranca::garantir_permissao_ou_supervisor(
        &conn, 
        "VENDA_CANCELAR", 
        Some(&dto.venda_id), 
        Some("commands_venda::cancelar_venda"), 
        Some(&dto.motivo_cancelamento), 
        req_sup.as_ref()
    )?;

    let status_venda: String = conn.query_row(
        "SELECT status FROM vendas WHERE id = ?1 AND status IN ('EM_ANDAMENTO', 'FINALIZADA')",
        rusqlite::params![&dto.venda_id],
        |row| row.get(0),
    ).map_err(|_| "Venda nao encontrada ou ja esta cancelada.".to_string())?;

    let agora = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // Cancelar todos os itens ativos com rastreabilidade
    tx.execute(
        "UPDATE venda_itens
         SET cancelado = 1,
             cancelado_em = ?1,
             usuario_cancelamento_id = ?2,
             motivo_cancelamento = ?3,
             supervisor_id = ?4,
             autorizacao_id = ?5
         WHERE venda_id = ?6 AND cancelado = 0",
        rusqlite::params![
            &agora,
            &dto.usuario_cancelamento_id,
            &dto.motivo_cancelamento,
            &dto.supervisor_id,
            &dto.autorizacao_id,
            &dto.venda_id
        ],
    ).map_err(|e| e.to_string())?;

    // Cancelar venda com todos os campos de rastreamento
    tx.execute(
        "UPDATE vendas SET
             status = 'CANCELADA',
             subtotal_minor = 0,
             total_minor = 0,
             cancelado_em = ?1,
             usuario_cancelamento_id = ?2,
             motivo_cancelamento = ?3,
             supervisor_id = ?4,
             autorizacao_id = ?5,
             atualizado_em = ?1
         WHERE id = ?6",
        rusqlite::params![
            &agora,
            &dto.usuario_cancelamento_id,
            &dto.motivo_cancelamento,
            &dto.supervisor_id,
            &dto.autorizacao_id,
            &dto.venda_id
        ],
    ).map_err(|e| e.to_string())?;

    // Evento outbox: VENDA_CANCELADA
    outbox_inserir(
        &tx,
        "VENDA_CANCELADA",
        json!({
            "venda_id": &dto.venda_id,
            "usuario_cancelamento_id": &dto.usuario_cancelamento_id,
            "motivo": &dto.motivo_cancelamento,
            "supervisor_id": &dto.supervisor_id,
            "cancelado_em": &agora
        }),
    )?;

    // FASE 11 - ESTOQUE: Estornar itens se a venda estava FINALIZADA
    if status_venda == "FINALIZADA" {
        processar_estorno_venda(&tx, &dto.venda_id, &dto.usuario_cancelamento_id).map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;

    warn!(
        componente = "aureon-pdv::commands_venda",
        venda_id = %dto.venda_id,
        usuario = %dto.usuario_cancelamento_id,
        "Venda cancelada"
    );

    Ok(RespostaBase::ok("Venda cancelada", true))
}

// ================================================================
// Command: obter_venda
// ================================================================

#[tauri::command]
pub async fn obter_venda(
    venda_id: String,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<VendaDetalheResp>, String> {
    info!(
        componente = "aureon-pdv::commands_venda",
        venda_id = %venda_id,
        "Chamada: obter_venda"
    );

    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let resumo = obter_resumo_venda_conn(&conn, &venda_id)?;

    let mut stmt = conn.prepare(
        "SELECT id, venda_id, produto_id, descricao_produto, codigo_produto,
                quantidade_escala3, preco_unitario_minor, desconto_item_minor,
                total_item_minor, cancelado, cancelado_em, motivo_cancelamento, criado_em
         FROM venda_itens WHERE venda_id = ?1 ORDER BY criado_em ASC"
    ).map_err(|e| e.to_string())?;

    let iter = stmt.query_map(rusqlite::params![&venda_id], |row| {
        Ok(VendaItemResp {
            id:                  row.get(0)?,
            venda_id:            row.get(1)?,
            produto_id:          row.get(2)?,
            descricao_produto:   row.get(3)?,
            codigo_produto:      row.get(4)?,
            quantidade_escala3:  row.get(5)?,
            preco_unitario_minor: row.get(6)?,
            desconto_item_minor: row.get(7)?,
            total_item_minor:    row.get(8)?,
            cancelado:           row.get::<_, i32>(9)? == 1,
            cancelado_em:        row.get(10)?,
            motivo_cancelamento: row.get(11)?,
            criado_em:           row.get(12)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut itens = Vec::new();
    for item in iter {
        if let Ok(i) = item { itens.push(i); }
    }

    let detalhe = VendaDetalheResp { venda: resumo.dados.unwrap(), itens };
    Ok(RespostaBase::ok("Venda encontrada", detalhe))
}

// ================================================================
// Funcao auxiliar: resumo da venda
// ================================================================

fn obter_resumo_venda(
    conn: &rusqlite::Connection,
    venda_id: &str,
) -> Result<RespostaBase<VendaResumoResp>, String> {
    obter_resumo_venda_conn(conn, venda_id)
}

fn obter_resumo_venda_conn(
    conn: &rusqlite::Connection,
    venda_id: &str,
) -> Result<RespostaBase<VendaResumoResp>, String> {
    let resp = conn.query_row(
        "SELECT v.id, v.numero_venda, v.status, v.tipo_venda,
                v.subtotal_minor, v.desconto_total_minor, v.acrescimo_total_minor, v.total_minor,
                COUNT(CASE WHEN vi.cancelado = 0 THEN 1 END) as total_itens
         FROM vendas v
         LEFT JOIN venda_itens vi ON vi.venda_id = v.id
         WHERE v.id = ?1
         GROUP BY v.id",
        rusqlite::params![venda_id],
        |row| {
            Ok(VendaResumoResp {
                id:                   row.get(0)?,
                numero_venda:         row.get(1)?,  // Option<i64> — NULL enquanto em andamento
                status:               row.get(2)?,
                tipo_venda:           row.get(3)?,
                subtotal_minor:       row.get(4)?,
                desconto_total_minor: row.get(5)?,
                acrescimo_total_minor: row.get(6)?,
                total_minor:          row.get(7)?,
                total_itens:          row.get(8)?,
            })
        },
    ).map_err(|e| format!("Venda nao encontrada: {e}"))?;

    Ok(RespostaBase::ok("Venda", resp))
}
