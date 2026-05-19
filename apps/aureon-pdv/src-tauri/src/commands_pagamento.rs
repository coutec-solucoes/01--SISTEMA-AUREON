use tauri::State;
use tracing::{info, error};
use uuid::Uuid;

use aureon_core::{dtos::*, RespostaBase};
use crate::estado::EstadoApp;

/// Registra um pagamento para uma venda, com suporte a multiplas moedas.
/// Usa cotacoes_cache para converter o valor informado para a moeda principal.
#[tauri::command]
pub async fn registrar_pagamento(
    dto: RegistrarPagamentoReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<PagamentoResp>, String> {
    info!(
        componente = "aureon-pdv::commands_pagamento",
        venda_id = %dto.venda_id,
        forma = %dto.forma_pagamento,
        moeda = %dto.moeda_codigo,
        valor = dto.valor_informado,
        "Chamada: registrar_pagamento"
    );

    if dto.valor_informado <= 0.0 {
        return Err("Valor do pagamento deve ser maior que zero.".into());
    }

    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    // Verificar se venda existe e esta EM_ANDAMENTO
    let venda_ok: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM vendas WHERE id = ?1 AND status = 'EM_ANDAMENTO'",
        rusqlite::params![&dto.venda_id],
        |row| row.get(0),
    ).unwrap_or(false);

    if !venda_ok {
        return Err("Venda nao encontrada ou nao esta em andamento.".into());
    }

    // Buscar moeda principal da empresa (principal = 1)
    let moeda_principal: String = conn.query_row(
        "SELECT codigo FROM moedas_cache WHERE principal = 1 LIMIT 1",
        [],
        |row| row.get(0),
    ).unwrap_or_else(|_| "BRL".to_string());

    // Calcular taxa de cambio
    // Se a moeda do pagamento ja eh a principal, taxa = 1.0
    let taxa_cambio: f64 = if dto.moeda_codigo.to_uppercase() == moeda_principal.to_uppercase() {
        1.0
    } else {
        // Buscar cotacao: quanto vale 1 unidade de dto.moeda_codigo em moeda principal
        conn.query_row(
            "SELECT taxa FROM cotacoes_cache
             WHERE UPPER(moeda_origem) = UPPER(?1) AND UPPER(moeda_destino) = UPPER(?2)
             ORDER BY data_cotacao DESC LIMIT 1",
            rusqlite::params![&dto.moeda_codigo, &moeda_principal],
            |row| row.get(0),
        ).map_err(|_| format!(
            "Cotacao nao encontrada para {} -> {}. Atualize as cotacoes antes de usar esta moeda.",
            dto.moeda_codigo, moeda_principal
        ))?
    };

    let valor_convertido = dto.valor_informado * taxa_cambio;
    let agora = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let pag_id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO venda_pagamentos (id, venda_id, forma_pagamento, moeda_codigo,
                                       valor_informado, valor_convertido, taxa_cambio, troco, criado_em)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0.0, ?8)",
        rusqlite::params![
            &pag_id,
            &dto.venda_id,
            &dto.forma_pagamento,
            &dto.moeda_codigo,
            dto.valor_informado,
            valor_convertido,
            taxa_cambio,
            &agora
        ],
    ).map_err(|e| {
        error!(componente = "aureon-pdv::commands_pagamento", erro = %e, "Erro ao inserir pagamento");
        e.to_string()
    })?;

    let resp = PagamentoResp {
        id:              pag_id,
        venda_id:        dto.venda_id,
        forma_pagamento: dto.forma_pagamento,
        moeda_codigo:    dto.moeda_codigo,
        valor_informado: dto.valor_informado,
        valor_convertido,
        taxa_cambio,
        troco: 0.0,
        criado_em: agora,
    };

    Ok(RespostaBase::ok("Pagamento registrado", resp))
}

/// Calcula o troco com base nos pagamentos ja registrados para a venda.
#[tauri::command]
pub async fn calcular_troco(
    venda_id: String,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<TrocoResp>, String> {
    info!(
        componente = "aureon-pdv::commands_pagamento",
        venda_id = %venda_id,
        "Chamada: calcular_troco"
    );

    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let total_venda: f64 = conn.query_row(
        "SELECT total FROM vendas WHERE id = ?1",
        rusqlite::params![&venda_id],
        |row| row.get(0),
    ).map_err(|_| "Venda nao encontrada.".to_string())?;

    let total_pago: f64 = conn.query_row(
        "SELECT COALESCE(SUM(valor_convertido), 0.0) FROM venda_pagamentos WHERE venda_id = ?1",
        rusqlite::params![&venda_id],
        |row| row.get(0),
    ).unwrap_or(0.0);

    let troco = total_pago - total_venda;

    let resp = TrocoResp {
        total_venda,
        total_pago,
        troco: troco.max(0.0),
        quitado: total_pago >= total_venda,
    };

    Ok(RespostaBase::ok("Troco calculado", resp))
}

/// Finaliza a venda, exigindo que o pagamento total cubra o valor da venda.
#[tauri::command]
pub async fn finalizar_venda(
    venda_id: String,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<VendaResumoResp>, String> {
    info!(
        componente = "aureon-pdv::commands_pagamento",
        venda_id = %venda_id,
        "Chamada: finalizar_venda"
    );

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    // Verificar se venda esta EM_ANDAMENTO
    let (total_venda, total_itens): (f64, i64) = conn.query_row(
        "SELECT v.total, COUNT(CASE WHEN vi.cancelado = 0 THEN 1 END)
         FROM vendas v
         LEFT JOIN venda_itens vi ON vi.venda_id = v.id
         WHERE v.id = ?1 AND v.status = 'EM_ANDAMENTO'
         GROUP BY v.id",
        rusqlite::params![&venda_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).map_err(|_| "Venda nao encontrada ou nao esta em andamento.".to_string())?;

    if total_itens == 0 {
        return Err("Nao eh possivel finalizar uma venda sem itens.".into());
    }

    // Verificar se pagamento cobre o total
    let total_pago: f64 = conn.query_row(
        "SELECT COALESCE(SUM(valor_convertido), 0.0) FROM venda_pagamentos WHERE venda_id = ?1",
        rusqlite::params![&venda_id],
        |row| row.get(0),
    ).unwrap_or(0.0);

    // Tolerancia de 1 centavo para arredondamentos de ponto flutuante
    if total_pago < total_venda - 0.01 {
        return Err(format!(
            "Pagamento insuficiente. Total: {:.2} | Pago: {:.2} | Faltam: {:.2}",
            total_venda,
            total_pago,
            total_venda - total_pago
        ));
    }

    let agora = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE vendas SET status = 'FINALIZADA', finalizado_em = ?1, atualizado_em = ?1
         WHERE id = ?2",
        rusqlite::params![&agora, &venda_id],
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    // Retornar resumo final
    let resp = conn.query_row(
        "SELECT v.id, v.numero_venda, v.status, v.tipo_venda,
                v.subtotal, v.desconto_total, v.acrescimo_total, v.total,
                COUNT(CASE WHEN vi.cancelado = 0 THEN 1 END) as total_itens
         FROM vendas v
         LEFT JOIN venda_itens vi ON vi.venda_id = v.id
         WHERE v.id = ?1
         GROUP BY v.id",
        rusqlite::params![&venda_id],
        |row| {
            Ok(VendaResumoResp {
                id:              row.get(0)?,
                numero_venda:    row.get(1)?,
                status:          row.get(2)?,
                tipo_venda:      row.get(3)?,
                subtotal:        row.get(4)?,
                desconto_total:  row.get(5)?,
                acrescimo_total: row.get(6)?,
                total:           row.get(7)?,
                total_itens:     row.get(8)?,
            })
        },
    ).map_err(|e| e.to_string())?;

    Ok(RespostaBase::ok("Venda finalizada com sucesso", resp))
}

/// Lista todos os pagamentos registrados para uma venda.
#[tauri::command]
pub async fn listar_pagamentos_venda(
    venda_id: String,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<Vec<PagamentoResp>>, String> {
    info!(
        componente = "aureon-pdv::commands_pagamento",
        venda_id = %venda_id,
        "Chamada: listar_pagamentos_venda"
    );

    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id, venda_id, forma_pagamento, moeda_codigo,
                valor_informado, valor_convertido, taxa_cambio, troco, criado_em
         FROM venda_pagamentos
         WHERE venda_id = ?1
         ORDER BY criado_em ASC"
    ).map_err(|e| e.to_string())?;

    let iter = stmt.query_map(rusqlite::params![&venda_id], |row| {
        Ok(PagamentoResp {
            id:              row.get(0)?,
            venda_id:        row.get(1)?,
            forma_pagamento: row.get(2)?,
            moeda_codigo:    row.get(3)?,
            valor_informado: row.get(4)?,
            valor_convertido: row.get(5)?,
            taxa_cambio:     row.get(6)?,
            troco:           row.get(7)?,
            criado_em:       row.get(8)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut pagamentos = Vec::new();
    for p in iter {
        if let Ok(val) = p { pagamentos.push(val); }
    }

    Ok(RespostaBase::ok("Pagamentos da venda", pagamentos))
}
