use tauri::State;
use tracing::{info, error, warn};
use uuid::Uuid;

use aureon_core::{dtos::*, RespostaBase};
use crate::estado::EstadoApp;

/// Inicia uma nova venda. Garante numero sequencial unico via transacao.
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

    // Verificar se a sessao de caixa existe e esta ABERTO
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

    // Transacao: buscar e incrementar numero sequencial + criar venda atomicamente
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let numero_venda: i64 = tx.query_row(
        "SELECT proximo_numero FROM controle_numeracao WHERE id = 1",
        [],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE controle_numeracao SET proximo_numero = proximo_numero + 1, atualizado_em = ?1 WHERE id = 1",
        rusqlite::params![&agora],
    ).map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO vendas (id, numero_venda, sessao_caixa_id, usuario_id, status, tipo_venda,
                             subtotal, desconto_total, acrescimo_total, total, criado_em, atualizado_em)
         VALUES (?1, ?2, ?3, ?4, 'EM_ANDAMENTO', ?5, 0.0, 0.0, 0.0, 0.0, ?6, ?6)",
        rusqlite::params![
            &venda_id,
            numero_venda,
            &sessao_caixa_id,
            &usuario_id,
            &tipo_venda,
            &agora
        ],
    ).map_err(|e| {
        error!(componente = "aureon-pdv::commands_venda", erro = %e, "Erro ao inserir venda");
        e.to_string()
    })?;

    tx.commit().map_err(|e| e.to_string())?;

    let resp = VendaResumoResp {
        id: venda_id,
        numero_venda,
        status: "EM_ANDAMENTO".into(),
        tipo_venda,
        subtotal: 0.0,
        desconto_total: 0.0,
        acrescimo_total: 0.0,
        total: 0.0,
        total_itens: 0,
    };

    Ok(RespostaBase::ok("Venda iniciada", resp))
}

/// Busca um produto no cache local por codigo interno ou codigo de barras.
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

    let produto = conn.query_row(
        "SELECT p.produto_id, p.codigo, p.codigo_barras, p.nome, p.unidade_medida, p.ativo,
                COALESCE(pr.preco_venda, 0.0) as preco_venda
         FROM produtos_cache p
         LEFT JOIN produtos_precos_cache pr ON pr.produto_id = p.produto_id
         WHERE p.ativo = 1 AND (UPPER(p.codigo) = UPPER(?1) OR p.codigo_barras = ?1)
         LIMIT 1",
        rusqlite::params![&codigo],
        |row| {
            Ok(ProdutoPdvResp {
                produto_id:    row.get(0)?,
                codigo:        row.get(1)?,
                codigo_barras: row.get(2)?,
                nome:          row.get(3)?,
                unidade_medida: row.get(4)?,
                ativo:         row.get::<_, i32>(5)? == 1,
                preco_venda:   row.get(6)?,
            })
        },
    ).ok();

    Ok(RespostaBase::ok("Produto encontrado", produto))
}

/// Adiciona um item a uma venda em andamento e recalcula os totais.
#[tauri::command]
pub async fn adicionar_item_venda(
    venda_id: String,
    produto_id: String,
    descricao_produto: String,
    codigo_produto: Option<String>,
    codigo_barras: Option<String>,
    quantidade: f64,
    preco_unitario: f64,
    desconto_item: f64,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<VendaResumoResp>, String> {
    info!(
        componente = "aureon-pdv::commands_venda",
        venda_id = %venda_id,
        produto_id = %produto_id,
        quantidade,
        "Chamada: adicionar_item_venda"
    );

    if quantidade <= 0.0 {
        return Err("Quantidade deve ser maior que zero.".into());
    }
    if preco_unitario < 0.0 {
        return Err("Preco unitario nao pode ser negativo.".into());
    }

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    // Verificar se venda existe e esta EM_ANDAMENTO
    let venda_ok: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM vendas WHERE id = ?1 AND status = 'EM_ANDAMENTO'",
        rusqlite::params![&venda_id],
        |row| row.get(0),
    ).unwrap_or(false);

    if !venda_ok {
        return Err("Venda nao encontrada ou nao esta em andamento.".into());
    }

    let item_id = Uuid::new_v4().to_string();
    let agora = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let total_item = (quantidade * preco_unitario) - desconto_item;
    let total_item = total_item.max(0.0);

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO venda_itens (id, venda_id, produto_id, descricao_produto, codigo_produto,
                                  codigo_barras, quantidade, preco_unitario, desconto_item,
                                  acrescimo_item, total_item, cancelado, criado_em)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0.0, ?10, 0, ?11)",
        rusqlite::params![
            &item_id,
            &venda_id,
            &produto_id,
            &descricao_produto,
            &codigo_produto,
            &codigo_barras,
            quantidade,
            preco_unitario,
            desconto_item,
            total_item,
            &agora
        ],
    ).map_err(|e| {
        error!(componente = "aureon-pdv::commands_venda", erro = %e, "Erro ao inserir item");
        e.to_string()
    })?;

    // Recalcular totais da venda via SUM do banco (fonte de verdade)
    tx.execute(
        "UPDATE vendas SET
            subtotal = (SELECT COALESCE(SUM(total_item), 0.0) FROM venda_itens WHERE venda_id = ?1 AND cancelado = 0),
            total    = (SELECT COALESCE(SUM(total_item), 0.0) FROM venda_itens WHERE venda_id = ?1 AND cancelado = 0)
                       + acrescimo_total - desconto_total,
            atualizado_em = ?2
         WHERE id = ?1",
        rusqlite::params![&venda_id, &agora],
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    obter_resumo_venda(&conn, &venda_id)
}

/// Cancela um item especifico da venda e recalcula os totais.
#[tauri::command]
pub async fn cancelar_item_venda(
    item_id: String,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<VendaResumoResp>, String> {
    info!(
        componente = "aureon-pdv::commands_venda",
        item_id = %item_id,
        "Chamada: cancelar_item_venda"
    );

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    // Buscar venda_id do item
    let venda_id: String = conn.query_row(
        "SELECT venda_id FROM venda_itens WHERE id = ?1 AND cancelado = 0",
        rusqlite::params![&item_id],
        |row| row.get(0),
    ).map_err(|_| "Item nao encontrado ou ja cancelado.".to_string())?;

    let agora = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE venda_itens SET cancelado = 1, cancelado_em = ?1 WHERE id = ?2",
        rusqlite::params![&agora, &item_id],
    ).map_err(|e| e.to_string())?;

    // Recalcular totais
    tx.execute(
        "UPDATE vendas SET
            subtotal = (SELECT COALESCE(SUM(total_item), 0.0) FROM venda_itens WHERE venda_id = ?1 AND cancelado = 0),
            total    = (SELECT COALESCE(SUM(total_item), 0.0) FROM venda_itens WHERE venda_id = ?1 AND cancelado = 0)
                       + acrescimo_total - desconto_total,
            atualizado_em = ?2
         WHERE id = ?1",
        rusqlite::params![&venda_id, &agora],
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    warn!(
        componente = "aureon-pdv::commands_venda",
        item_id = %item_id,
        venda_id = %venda_id,
        "Item cancelado"
    );

    obter_resumo_venda(&conn, &venda_id)
}

/// Cancela uma venda inteira (todos os itens sao marcados como cancelados).
#[tauri::command]
pub async fn cancelar_venda(
    venda_id: String,
    motivo: Option<String>,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<bool>, String> {
    info!(
        componente = "aureon-pdv::commands_venda",
        venda_id = %venda_id,
        "Chamada: cancelar_venda"
    );

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let venda_ok: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM vendas WHERE id = ?1 AND status = 'EM_ANDAMENTO'",
        rusqlite::params![&venda_id],
        |row| row.get(0),
    ).unwrap_or(false);

    if !venda_ok {
        return Err("Venda nao encontrada ou nao pode ser cancelada.".into());
    }

    let agora = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // Cancelar todos os itens ativos
    tx.execute(
        "UPDATE venda_itens SET cancelado = 1, cancelado_em = ?1 WHERE venda_id = ?2 AND cancelado = 0",
        rusqlite::params![&agora, &venda_id],
    ).map_err(|e| e.to_string())?;

    // Marcar venda como cancelada
    let obs = motivo.unwrap_or_default();
    tx.execute(
        "UPDATE vendas SET status = 'CANCELADA', subtotal = 0.0, total = 0.0,
                           observacao = ?1, atualizado_em = ?2
         WHERE id = ?3",
        rusqlite::params![&obs, &agora, &venda_id],
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    warn!(
        componente = "aureon-pdv::commands_venda",
        venda_id = %venda_id,
        "Venda cancelada"
    );

    Ok(RespostaBase::ok("Venda cancelada", true))
}

/// Retorna detalhes completos da venda com todos os itens nao cancelados.
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
                quantidade, preco_unitario, desconto_item, total_item, cancelado, criado_em
         FROM venda_itens
         WHERE venda_id = ?1
         ORDER BY criado_em ASC"
    ).map_err(|e| e.to_string())?;

    let iter = stmt.query_map(rusqlite::params![&venda_id], |row| {
        Ok(VendaItemResp {
            id:               row.get(0)?,
            venda_id:         row.get(1)?,
            produto_id:       row.get(2)?,
            descricao_produto: row.get(3)?,
            codigo_produto:   row.get(4)?,
            quantidade:       row.get(5)?,
            preco_unitario:   row.get(6)?,
            desconto_item:    row.get(7)?,
            total_item:       row.get(8)?,
            cancelado:        row.get::<_, i32>(9)? == 1,
            criado_em:        row.get(10)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut itens = Vec::new();
    for item in iter {
        if let Ok(i) = item { itens.push(i); }
    }

    let detalhe = VendaDetalheResp { venda: resumo.dados.unwrap(), itens };

    Ok(RespostaBase::ok("Venda encontrada", detalhe))
}

// --- Funcao auxiliar interna ---

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
                v.subtotal, v.desconto_total, v.acrescimo_total, v.total,
                COUNT(CASE WHEN vi.cancelado = 0 THEN 1 END) as total_itens
         FROM vendas v
         LEFT JOIN venda_itens vi ON vi.venda_id = v.id
         WHERE v.id = ?1
         GROUP BY v.id",
        rusqlite::params![venda_id],
        |row| {
            Ok(VendaResumoResp {
                id:             row.get(0)?,
                numero_venda:   row.get(1)?,
                status:         row.get(2)?,
                tipo_venda:     row.get(3)?,
                subtotal:       row.get(4)?,
                desconto_total: row.get(5)?,
                acrescimo_total: row.get(6)?,
                total:          row.get(7)?,
                total_itens:    row.get(8)?,
            })
        },
    ).map_err(|e| format!("Venda nao encontrada: {e}"))?;

    Ok(RespostaBase::ok("Venda", resp))
}
