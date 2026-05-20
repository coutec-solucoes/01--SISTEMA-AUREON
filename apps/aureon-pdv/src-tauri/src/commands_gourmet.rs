use tauri::State;
use rusqlite::{params, Connection};
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;
use tracing::info;

use aureon_core::dtos::*;
use aureon_core::RespostaBase;
use crate::estado::EstadoApp;
use crate::commands_caixa::inserir_outbox;

// ============================================================================
// AUXILIARES
// ============================================================================

/// Verifica se existe uma sessão de caixa aberta
fn verificar_caixa_aberto(conn: &Connection) -> Result<String, String> {
    let sessao_id: Option<String> = conn
        .query_row(
            "SELECT id FROM sessoes_caixa WHERE status = 'ABERTO' LIMIT 1",
            [],
            |r| r.get(0),
        )
        .ok();

    match sessao_id {
        Some(id) => Ok(id),
        None => Err("Não há nenhuma sessão de caixa aberta. Operação bloqueada.".to_string()),
    }
}

// ============================================================================
// MESAS COMMANDS
// ============================================================================

#[tauri::command]
pub async fn listar_mesas_pdv(
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<Vec<MesaOperacionalResp>>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT 
                mc.id as cache_id,
                mc.numero as mesa_numero,
                mc.nome as cache_nome,
                mo.id as op_id,
                mo.nome_exibicao,
                mo.cliente_nome_informal,
                mo.cliente_id,
                mo.status as op_status,
                mo.usuario_abertura_id,
                mo.sessao_caixa_id,
                mo.observacao,
                mo.aberta_em,
                COALESCE((
                    SELECT SUM(gi.total_item_minor) 
                    FROM gourmet_itens gi 
                    WHERE gi.origem_tipo = 'MESA' 
                      AND gi.origem_id = mo.id 
                      AND gi.cancelado = 0 
                      AND gi.status != 'TRANSFERIDO'
                ), 0) as total_consumo_minor
            FROM mesas_cache mc
            LEFT JOIN mesas_operacionais mo ON mc.numero = mo.mesa_numero AND mo.status IN ('ABERTA', 'RESERVADA', 'BLOQUEADA')
            WHERE mc.ativo = 1
            ORDER BY mc.numero",
        )
        .map_err(|e| e.to_string())?;

    let iter = stmt
        .query_map([], |row| {
            let op_id: Option<String> = row.get(3)?;
            let status: String = match &op_id {
                Some(_) => row.get(7)?,
                None => "LIVRE".to_string(),
            };

            Ok(MesaOperacionalResp {
                id: op_id,
                mesa_id: row.get(0)?,
                mesa_numero: row.get(1)?,
                nome_exibicao: row.get::<_, Option<String>>(4)?.unwrap_or_else(|| row.get::<_, String>(2).unwrap_or_default()),
                cliente_nome_informal: row.get(5)?,
                cliente_id: row.get(6)?,
                status,
                usuario_abertura_id: row.get(8)?,
                sessao_caixa_id: row.get(9)?,
                observacao: row.get(10)?,
                aberta_em: row.get(11)?,
                total_consumo_minor: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut mesas = Vec::new();
    for m in iter {
        if let Ok(mesa) = m {
            mesas.push(mesa);
        }
    }

    Ok(RespostaBase::ok("Mesas listadas com sucesso", mesas))
}

#[tauri::command]
pub async fn abrir_mesa(
    dto: AbrirMesaReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<MesaOperacionalResp>, String> {
    info!(componente = "aureon-pdv::commands_gourmet", mesa = dto.mesa_numero, "Chamada: abrir_mesa");

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    
    // 1. Validar caixa aberto
    verificar_caixa_aberto(&conn)?;

    // 2. Validar se a mesa existe no cache e está ativa
    let mesa_existe: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM mesas_cache WHERE numero = ?1 AND ativo = 1",
            params![dto.mesa_numero],
            |r| r.get(0),
        )
        .unwrap_or(false);

    if !mesa_existe {
        return Err("Mesa não cadastrada ou inativa.".to_string());
    }

    // 3. Validar se mesa já está ativa (status IN ABERTA, RESERVADA, BLOQUEADA)
    let mesa_ativa: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM mesas_operacionais WHERE mesa_numero = ?1 AND status IN ('ABERTA', 'RESERVADA', 'BLOQUEADA')",
            params![dto.mesa_numero],
            |r| r.get(0),
        )
        .unwrap_or(false);

    if mesa_ativa {
        return Err("Mesa já está aberta, reservada ou bloqueada.".to_string());
    }

    let id = Uuid::new_v4().to_string();
    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO mesas_operacionais (
            id, mesa_numero, nome_exibicao, cliente_nome_informal, cliente_id,
            status, usuario_abertura_id, sessao_caixa_id, observacao, aberta_em
        ) VALUES (?1, ?2, ?3, ?4, ?5, 'ABERTA', ?6, ?7, ?8, ?9)",
        params![
            &id,
            dto.mesa_numero,
            &dto.nome_exibicao,
            &dto.cliente_nome_informal,
            &dto.cliente_id,
            &dto.usuario_id,
            &dto.sessao_caixa_id,
            &dto.observacao,
            &agora
        ],
    ).map_err(|e| e.to_string())?;

    // Inserir outbox
    inserir_outbox(
        &tx,
        "MESA_ABERTA",
        json!({
            "id": &id,
            "mesa_numero": dto.mesa_numero,
            "nome_exibicao": &dto.nome_exibicao,
            "cliente_id": &dto.cliente_id,
            "usuario_id": &dto.usuario_id,
            "sessao_caixa_id": &dto.sessao_caixa_id,
            "aberta_em": &agora
        }),
    )?;

    tx.commit().map_err(|e| e.to_string())?;

    let resp = MesaOperacionalResp {
        id: Some(id),
        mesa_id: "".to_string(), // opcional no retorno direto
        mesa_numero: dto.mesa_numero,
        nome_exibicao: dto.nome_exibicao,
        cliente_nome_informal: dto.cliente_nome_informal,
        cliente_id: dto.cliente_id,
        status: "ABERTA".to_string(),
        usuario_abertura_id: Some(dto.usuario_id),
        sessao_caixa_id: Some(dto.sessao_caixa_id),
        observacao: dto.observacao,
        aberta_em: Some(agora),
        total_consumo_minor: 0,
    };

    Ok(RespostaBase::ok("Mesa aberta com sucesso", resp))
}

#[tauri::command]
pub async fn reservar_mesa(
    dto: ReservarMesaReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<MesaOperacionalResp>, String> {
    info!(componente = "aureon-pdv::commands_gourmet", mesa = dto.mesa_numero, "Chamada: reservar_mesa");

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    verificar_caixa_aberto(&conn)?;

    // Validar se mesa já está ativa
    let mesa_ativa: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM mesas_operacionais WHERE mesa_numero = ?1 AND status IN ('ABERTA', 'RESERVADA', 'BLOQUEADA')",
            params![dto.mesa_numero],
            |r| r.get(0),
        )
        .unwrap_or(false);

    if mesa_ativa {
        return Err("Mesa já está ativa ou ocupada.".to_string());
    }

    let id = Uuid::new_v4().to_string();
    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO mesas_operacionais (
            id, mesa_numero, nome_exibicao, cliente_nome_informal, cliente_id,
            status, usuario_abertura_id, sessao_caixa_id, observacao, aberta_em
        ) VALUES (?1, ?2, ?3, ?4, ?5, 'RESERVADA', ?6, ?7, ?8, ?9)",
        params![
            &id,
            dto.mesa_numero,
            &dto.nome_exibicao,
            &dto.cliente_nome_informal,
            &dto.cliente_id,
            &dto.usuario_id,
            &dto.sessao_caixa_id,
            &dto.observacao,
            &agora
        ],
    ).map_err(|e| e.to_string())?;

    inserir_outbox(
        &tx,
        "MESA_RESERVADA",
        json!({
            "id": &id,
            "mesa_numero": dto.mesa_numero,
            "nome_exibicao": &dto.nome_exibicao,
            "cliente_id": &dto.cliente_id,
            "usuario_id": &dto.usuario_id,
            "sessao_caixa_id": &dto.sessao_caixa_id,
            "reservada_em": &agora
        }),
    )?;

    tx.commit().map_err(|e| e.to_string())?;

    let resp = MesaOperacionalResp {
        id: Some(id),
        mesa_id: "".to_string(),
        mesa_numero: dto.mesa_numero,
        nome_exibicao: dto.nome_exibicao,
        cliente_nome_informal: dto.cliente_nome_informal,
        cliente_id: dto.cliente_id,
        status: "RESERVADA".to_string(),
        usuario_abertura_id: Some(dto.usuario_id),
        sessao_caixa_id: Some(dto.sessao_caixa_id),
        observacao: dto.observacao,
        aberta_em: Some(agora),
        total_consumo_minor: 0,
    };

    Ok(RespostaBase::ok("Mesa reservada com sucesso", resp))
}

#[tauri::command]
pub async fn bloquear_mesa(
    dto: BloquearMesaReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<MesaOperacionalResp>, String> {
    info!(componente = "aureon-pdv::commands_gourmet", mesa = dto.mesa_numero, "Chamada: bloquear_mesa");

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    verificar_caixa_aberto(&conn)?;

    let active_op: Option<(String, String)> = conn
        .query_row(
            "SELECT id, status FROM mesas_operacionais WHERE mesa_numero = ?1 AND status IN ('ABERTA', 'RESERVADA', 'BLOQUEADA')",
            params![dto.mesa_numero],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok();

    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let op_id = match active_op {
        Some((id, status)) => {
            if status == "BLOQUEADA" {
                return Err("Mesa já está bloqueada.".to_string());
            }
            tx.execute(
                "UPDATE mesas_operacionais SET status = 'BLOQUEADA' WHERE id = ?1",
                params![&id],
            ).map_err(|e| e.to_string())?;
            id
        }
        None => {
            let id = Uuid::new_v4().to_string();
            tx.execute(
                "INSERT INTO mesas_operacionais (
                    id, mesa_numero, nome_exibicao, status, usuario_abertura_id, sessao_caixa_id, aberta_em
                ) VALUES (?1, ?2, 'Mesa ' || ?2, 'BLOQUEADA', ?3, ?4, ?5)",
                params![&id, dto.mesa_numero, &dto.usuario_id, &dto.sessao_caixa_id, &agora],
            ).map_err(|e| e.to_string())?;
            id
        }
    };

    inserir_outbox(
        &tx,
        "MESA_BLOQUEADA",
        json!({
            "id": &op_id,
            "mesa_numero": dto.mesa_numero,
            "bloqueada_em": &agora,
            "usuario_id": dto.usuario_id
        }),
    )?;

    tx.commit().map_err(|e| e.to_string())?;

    // Obter dados atualizados
    let resp = conn.query_row(
        "SELECT id, mesa_numero, nome_exibicao, cliente_nome_informal, cliente_id, status, usuario_abertura_id, sessao_caixa_id, observacao, aberta_em
         FROM mesas_operacionais WHERE id = ?1",
        params![&op_id],
        |r| {
            Ok(MesaOperacionalResp {
                id: Some(r.get(0)?),
                mesa_id: "".to_string(),
                mesa_numero: r.get(1)?,
                nome_exibicao: r.get(2)?,
                cliente_nome_informal: r.get(3)?,
                cliente_id: r.get(4)?,
                status: r.get(5)?,
                usuario_abertura_id: r.get(6)?,
                sessao_caixa_id: r.get(7)?,
                observacao: r.get(8)?,
                aberta_em: r.get(9)?,
                total_consumo_minor: 0,
            })
        }
    ).map_err(|e| e.to_string())?;

    Ok(RespostaBase::ok("Mesa bloqueada com sucesso", resp))
}

#[tauri::command]
pub async fn cancelar_mesa(
    dto: CancelarMesaReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<String>, String> {
    info!(componente = "aureon-pdv::commands_gourmet", mesa = dto.mesa_numero, "Chamada: cancelar_mesa");

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let op: Option<(String, String)> = conn
        .query_row(
            "SELECT id, status FROM mesas_operacionais WHERE mesa_numero = ?1 AND status IN ('ABERTA', 'RESERVADA', 'BLOQUEADA')",
            params![dto.mesa_numero],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok();

    let (op_id, _) = match op {
        Some(x) => x,
        None => return Err("Nenhum ciclo operacional ativo encontrado para esta mesa.".to_string()),
    };

    if dto.motivo_cancelamento.trim().is_empty() {
        return Err("Justificativa de cancelamento é obrigatória.".to_string());
    }

    // Regra 16: Se houver itens ativos, exige supervisor_id
    let tem_itens_ativos: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM gourmet_itens WHERE origem_tipo = 'MESA' AND origem_id = ?1 AND cancelado = 0 AND status != 'TRANSFERIDO'",
            params![&op_id],
            |r| r.get(0),
        )
        .unwrap_or(false);

    if tem_itens_ativos && dto.supervisor_id.is_none() {
        return Err("A mesa possui itens de consumo ativos. É necessária autorização de supervisor para cancelar.".to_string());
    }

    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // Atualiza status da mesa
    tx.execute(
        "UPDATE mesas_operacionais 
         SET status = 'CANCELADA',
             cancelada_em = ?1,
             usuario_cancelamento_id = ?2,
             motivo_cancelamento = ?3,
             supervisor_id = ?4,
             autorizacao_id = ?5
         WHERE id = ?6",
        params![
            &agora,
            &dto.usuario_cancelamento_id,
            &dto.motivo_cancelamento,
            &dto.supervisor_id,
            &dto.autorizacao_id,
            &op_id
        ],
    ).map_err(|e| e.to_string())?;

    // Cancela todos os itens ativos da mesa
    tx.execute(
        "UPDATE gourmet_itens
         SET status = 'CANCELADO',
             cancelado = 1,
             cancelado_em = ?1,
             usuario_cancelamento_id = ?2,
             motivo_cancelamento = ?3,
             supervisor_id = ?4,
             autorizacao_id = ?5
         WHERE origem_tipo = 'MESA' AND origem_id = ?6 AND cancelado = 0 AND status != 'TRANSFERIDO'",
        params![
            &agora,
            &dto.usuario_cancelamento_id,
            &dto.motivo_cancelamento,
            &dto.supervisor_id,
            &dto.autorizacao_id,
            &op_id
        ],
    ).map_err(|e| e.to_string())?;

    // Inserir outbox da mesa cancelada
    inserir_outbox(
        &tx,
        "MESA_CANCELADA",
        json!({
            "id": &op_id,
            "mesa_numero": dto.mesa_numero,
            "cancelada_em": &agora,
            "usuario_cancelamento_id": &dto.usuario_cancelamento_id,
            "motivo": &dto.motivo_cancelamento,
            "supervisor_id": &dto.supervisor_id
        }),
    )?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(RespostaBase::ok("Mesa cancelada com sucesso", op_id))
}

#[tauri::command]
pub async fn obter_mesa(
    mesa_numero: i32,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<MesaDetalheResp>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    // Buscar no cache
    let cache: Option<(String, String)> = conn
        .query_row(
            "SELECT id, nome FROM mesas_cache WHERE numero = ?1 AND ativo = 1",
            params![mesa_numero],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok();

    let (cache_id, cache_nome) = match cache {
        Some(x) => x,
        None => return Err(format!("Mesa {mesa_numero} não cadastrada ou inativa")),
    };

    // Buscar ciclo ativo
    let op = conn.query_row(
        "SELECT id, nome_exibicao, cliente_nome_informal, cliente_id, status, usuario_abertura_id, sessao_caixa_id, observacao, aberta_em
         FROM mesas_operacionais 
         WHERE mesa_numero = ?1 AND status IN ('ABERTA', 'RESERVADA', 'BLOQUEADA')",
        params![mesa_numero],
        |r| {
            Ok(MesaOperacionalResp {
                id: Some(r.get(0)?),
                mesa_id: cache_id.clone(),
                mesa_numero,
                nome_exibicao: r.get(1)?,
                cliente_nome_informal: r.get(2)?,
                cliente_id: r.get(3)?,
                status: r.get(4)?,
                usuario_abertura_id: r.get(5)?,
                sessao_caixa_id: r.get(6)?,
                observacao: r.get(7)?,
                aberta_em: r.get(8)?,
                total_consumo_minor: 0,
            })
        }
    ).ok();

    let mesa = match op {
        Some(mut m) => {
            // Calcular consumo
            let total: i64 = conn.query_row(
                "SELECT COALESCE(SUM(total_item_minor), 0) FROM gourmet_itens 
                 WHERE origem_tipo = 'MESA' AND origem_id = ?1 AND cancelado = 0 AND status != 'TRANSFERIDO'",
                params![m.id],
                |r| r.get(0)
            ).unwrap_or(0);
            m.total_consumo_minor = total;
            m
        }
        None => MesaOperacionalResp {
            id: None,
            mesa_id: cache_id,
            mesa_numero,
            nome_exibicao: cache_nome,
            cliente_nome_informal: None,
            cliente_id: None,
            status: "LIVRE".to_string(),
            usuario_abertura_id: None,
            sessao_caixa_id: None,
            observacao: None,
            aberta_em: None,
            total_consumo_minor: 0,
        }
    };

    // Obter itens ativos se houver ciclo
    let mut itens = Vec::new();
    if let Some(op_id) = &mesa.id {
        let mut stmt = conn.prepare(
            "SELECT id, origem_tipo, origem_id, produto_id, descricao_produto, codigo_produto,
                    quantidade_escala3, preco_unitario_minor, desconto_item_minor, acrescimo_item_minor,
                    total_item_minor, observacao_producao, local_producao_id, status, enviado_producao,
                    enviado_producao_em, cancelado, cancelado_em, motivo_cancelamento, supervisor_id,
                    autorizacao_id, criado_em
             FROM gourmet_itens 
             WHERE origem_tipo = 'MESA' AND origem_id = ?1 AND cancelado = 0 AND status != 'TRANSFERIDO'
             ORDER BY criado_em ASC"
        ).map_err(|e| e.to_string())?;

        let iter = stmt.query_map(params![op_id], |r| {
            Ok(GourmetItemResp {
                id: r.get(0)?,
                origem_tipo: r.get(1)?,
                origem_id: r.get(2)?,
                produto_id: r.get(3)?,
                descricao_produto: r.get(4)?,
                codigo_produto: r.get(5)?,
                quantidade_escala3: r.get(6)?,
                preco_unitario_minor: r.get(7)?,
                desconto_item_minor: r.get(8)?,
                acrescimo_item_minor: r.get(9)?,
                total_item_minor: r.get(10)?,
                observacao_producao: r.get(11)?,
                local_producao_id: r.get(12)?,
                status: r.get(13)?,
                enviado_producao: r.get::<_, i32>(14)? == 1,
                enviado_producao_em: r.get(15)?,
                cancelado: r.get::<_, i32>(16)? == 1,
                cancelado_em: r.get(17)?,
                motivo_cancelamento: r.get(18)?,
                supervisor_id: r.get(19)?,
                autorizacao_id: r.get(20)?,
                criado_em: r.get(21)?,
            })
        }).map_err(|e| e.to_string())?;

        for i in iter {
            if let Ok(item) = i {
                itens.push(item);
            }
        }
    }

    Ok(RespostaBase::ok("Mesa carregada com sucesso", MesaDetalheResp { mesa, itens }))
}

// ============================================================================
// COMANDAS COMMANDS
// ============================================================================

#[tauri::command]
pub async fn listar_comandas_pdv(
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<Vec<ComandaOperacionalResp>>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT 
                cc.id as cache_id,
                cc.numero as numero_comanda,
                co.id as op_id,
                co.codigo_barras_qr,
                co.cliente_nome_informal,
                co.cliente_id,
                co.status as op_status,
                co.usuario_abertura_id,
                co.sessao_caixa_id,
                co.observacao,
                co.aberta_em,
                COALESCE((
                    SELECT SUM(gi.total_item_minor) 
                    FROM gourmet_itens gi 
                    WHERE gi.origem_tipo = 'COMANDA' 
                      AND gi.origem_id = co.id 
                      AND gi.cancelado = 0 
                      AND gi.status != 'TRANSFERIDO'
                ), 0) as total_consumo_minor
            FROM comandas_cache cc
            LEFT JOIN comandas_operacionais co ON cc.numero = co.numero_comanda AND co.status IN ('ABERTA', 'BLOQUEADA')
            WHERE cc.ativo = 1
            ORDER BY cc.numero",
        )
        .map_err(|e| e.to_string())?;

    let iter = stmt
        .query_map([], |row| {
            let op_id: Option<String> = row.get(2)?;
            let status: String = match &op_id {
                Some(_) => row.get(6)?,
                None => "LIVRE".to_string(),
            };

            Ok(ComandaOperacionalResp {
                id: op_id,
                comanda_id: row.get(0)?,
                numero_comanda: row.get(1)?,
                codigo_barras_qr: row.get(3)?,
                cliente_nome_informal: row.get(4)?,
                cliente_id: row.get(5)?,
                status,
                usuario_abertura_id: row.get(7)?,
                sessao_caixa_id: row.get(8)?,
                observacao: row.get(9)?,
                aberta_em: row.get(10)?,
                total_consumo_minor: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut comandas = Vec::new();
    for c in iter {
        if let Ok(comanda) = c {
            comandas.push(comanda);
        }
    }

    Ok(RespostaBase::ok("Comandas listadas com sucesso", comandas))
}

#[tauri::command]
pub async fn abrir_comanda(
    dto: AbrirComandaReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<ComandaOperacionalResp>, String> {
    info!(componente = "aureon-pdv::commands_gourmet", comanda = dto.numero_comanda, "Chamada: abrir_comanda");

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    verificar_caixa_aberto(&conn)?;

    // Validar se comanda existe no cache e está ativa
    let existe: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM comandas_cache WHERE numero = ?1 AND ativo = 1",
            params![dto.numero_comanda],
            |r| r.get(0),
        )
        .unwrap_or(false);

    if !existe {
        return Err("Comanda não cadastrada ou inativa.".to_string());
    }

    // Validar se já está ativa
    let ativa: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM comandas_operacionais WHERE numero_comanda = ?1 AND status IN ('ABERTA', 'BLOQUEADA')",
            params![dto.numero_comanda],
            |r| r.get(0),
        )
        .unwrap_or(false);

    if ativa {
        return Err("Comanda já está aberta ou bloqueada.".to_string());
    }

    let id = Uuid::new_v4().to_string();
    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO comandas_operacionais (
            id, numero_comanda, codigo_barras_qr, cliente_nome_informal, cliente_id,
            status, usuario_abertura_id, sessao_caixa_id, observacao, aberta_em
        ) VALUES (?1, ?2, ?3, ?4, ?5, 'ABERTA', ?6, ?7, ?8, ?9)",
        params![
            &id,
            dto.numero_comanda,
            &dto.codigo_barras_qr,
            &dto.cliente_nome_informal,
            &dto.cliente_id,
            &dto.usuario_id,
            &dto.sessao_caixa_id,
            &dto.observacao,
            &agora
        ],
    ).map_err(|e| e.to_string())?;

    inserir_outbox(
        &tx,
        "COMANDA_ABERTA",
        json!({
            "id": &id,
            "numero_comanda": dto.numero_comanda,
            "cliente_id": &dto.cliente_id,
            "usuario_id": &dto.usuario_id,
            "sessao_caixa_id": &dto.sessao_caixa_id,
            "aberta_em": &agora
        }),
    )?;

    tx.commit().map_err(|e| e.to_string())?;

    let resp = ComandaOperacionalResp {
        id: Some(id),
        comanda_id: "".to_string(),
        numero_comanda: dto.numero_comanda,
        codigo_barras_qr: dto.codigo_barras_qr,
        cliente_nome_informal: dto.cliente_nome_informal,
        cliente_id: dto.cliente_id,
        status: "ABERTA".to_string(),
        usuario_abertura_id: Some(dto.usuario_id),
        sessao_caixa_id: Some(dto.sessao_caixa_id),
        observacao: dto.observacao,
        aberta_em: Some(agora),
        total_consumo_minor: 0,
    };

    Ok(RespostaBase::ok("Comanda aberta com sucesso", resp))
}

#[tauri::command]
pub async fn bloquear_comanda(
    dto: BloquearComandaReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<ComandaOperacionalResp>, String> {
    info!(componente = "aureon-pdv::commands_gourmet", comanda = dto.numero_comanda, "Chamada: bloquear_comanda");

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    verificar_caixa_aberto(&conn)?;

    let active_op: Option<(String, String)> = conn
        .query_row(
            "SELECT id, status FROM comandas_operacionais WHERE numero_comanda = ?1 AND status IN ('ABERTA', 'BLOQUEADA')",
            params![dto.numero_comanda],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok();

    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let op_id = match active_op {
        Some((id, status)) => {
            if status == "BLOQUEADA" {
                return Err("Comanda já está bloqueada.".to_string());
            }
            tx.execute(
                "UPDATE comandas_operacionais SET status = 'BLOQUEADA' WHERE id = ?1",
                params![&id],
            ).map_err(|e| e.to_string())?;
            id
        }
        None => {
            let id = Uuid::new_v4().to_string();
            tx.execute(
                "INSERT INTO comandas_operacionais (
                    id, numero_comanda, status, usuario_abertura_id, sessao_caixa_id, aberta_em
                ) VALUES (?1, ?2, 'BLOQUEADA', ?3, ?4, ?5)",
                params![&id, dto.numero_comanda, &dto.usuario_id, &dto.sessao_caixa_id, &agora],
            ).map_err(|e| e.to_string())?;
            id
        }
    };

    inserir_outbox(
        &tx,
        "COMANDA_BLOQUEADA",
        json!({
            "id": &op_id,
            "numero_comanda": dto.numero_comanda,
            "bloqueada_em": &agora,
            "usuario_id": dto.usuario_id
        }),
    )?;

    tx.commit().map_err(|e| e.to_string())?;

    let resp = conn.query_row(
        "SELECT id, numero_comanda, codigo_barras_qr, cliente_nome_informal, cliente_id, status, usuario_abertura_id, sessao_caixa_id, observacao, aberta_em
         FROM comandas_operacionais WHERE id = ?1",
        params![&op_id],
        |r| {
            Ok(ComandaOperacionalResp {
                id: Some(r.get(0)?),
                comanda_id: "".to_string(),
                numero_comanda: r.get(1)?,
                codigo_barras_qr: r.get(2)?,
                cliente_nome_informal: r.get(3)?,
                cliente_id: r.get(4)?,
                status: r.get(5)?,
                usuario_abertura_id: r.get(6)?,
                sessao_caixa_id: r.get(7)?,
                observacao: r.get(8)?,
                aberta_em: r.get(9)?,
                total_consumo_minor: 0,
            })
        }
    ).map_err(|e| e.to_string())?;

    Ok(RespostaBase::ok("Comanda bloqueada com sucesso", resp))
}

#[tauri::command]
pub async fn cancelar_comanda(
    dto: CancelarComandaReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<String>, String> {
    info!(componente = "aureon-pdv::commands_gourmet", comanda = dto.numero_comanda, "Chamada: cancelar_comanda");

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let op: Option<(String, String)> = conn
        .query_row(
            "SELECT id, status FROM comandas_operacionais WHERE numero_comanda = ?1 AND status IN ('ABERTA', 'BLOQUEADA')",
            params![dto.numero_comanda],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok();

    let (op_id, _) = match op {
        Some(x) => x,
        None => return Err("Nenhum ciclo operacional ativo encontrado para esta comanda.".to_string()),
    };

    if dto.motivo_cancelamento.trim().is_empty() {
        return Err("Justificativa de cancelamento é obrigatória.".to_string());
    }

    // Regra 16: Se houver itens ativos, exige supervisor
    let tem_itens_ativos: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM gourmet_itens WHERE origem_tipo = 'COMANDA' AND origem_id = ?1 AND cancelado = 0 AND status != 'TRANSFERIDO'",
            params![&op_id],
            |r| r.get(0),
        )
        .unwrap_or(false);

    if tem_itens_ativos && dto.supervisor_id.is_none() {
        return Err("A comanda possui itens de consumo ativos. É necessária autorização de supervisor para cancelar.".to_string());
    }

    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // Atualiza status da comanda
    tx.execute(
        "UPDATE comandas_operacionais 
         SET status = 'CANCELADA',
             cancelada_em = ?1,
             usuario_cancelamento_id = ?2,
             motivo_cancelamento = ?3,
             supervisor_id = ?4,
             autorizacao_id = ?5
         WHERE id = ?6",
        params![
            &agora,
            &dto.usuario_cancelamento_id,
            &dto.motivo_cancelamento,
            &dto.supervisor_id,
            &dto.autorizacao_id,
            &op_id
        ],
    ).map_err(|e| e.to_string())?;

    // Cancela todos os itens ativos da comanda
    tx.execute(
        "UPDATE gourmet_itens
         SET status = 'CANCELADO',
             cancelado = 1,
             cancelado_em = ?1,
             usuario_cancelamento_id = ?2,
             motivo_cancelamento = ?3,
             supervisor_id = ?4,
             autorizacao_id = ?5
         WHERE origem_tipo = 'COMANDA' AND origem_id = ?6 AND cancelado = 0 AND status != 'TRANSFERIDO'",
        params![
            &agora,
            &dto.usuario_cancelamento_id,
            &dto.motivo_cancelamento,
            &dto.supervisor_id,
            &dto.autorizacao_id,
            &op_id
        ],
    ).map_err(|e| e.to_string())?;

    inserir_outbox(
        &tx,
        "COMANDA_CANCELADA",
        json!({
            "id": &op_id,
            "numero_comanda": dto.numero_comanda,
            "cancelada_em": &agora,
            "usuario_cancelamento_id": &dto.usuario_cancelamento_id,
            "motivo": &dto.motivo_cancelamento,
            "supervisor_id": &dto.supervisor_id
        }),
    )?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(RespostaBase::ok("Comanda cancelada com sucesso", op_id))
}

#[tauri::command]
pub async fn obter_comanda(
    numero_comanda: i32,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<ComandaDetalheResp>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    // Buscar no cache
    let cache_id: String = conn
        .query_row(
            "SELECT id FROM comandas_cache WHERE numero = ?1 AND ativo = 1",
            params![numero_comanda],
            |r| r.get(0),
        )
        .map_err(|_| format!("Comanda {numero_comanda} não cadastrada ou inativa"))?;

    // Buscar ciclo ativo
    let op = conn.query_row(
        "SELECT id, codigo_barras_qr, cliente_nome_informal, cliente_id, status, usuario_abertura_id, sessao_caixa_id, observacao, aberta_em
         FROM comandas_operacionais 
         WHERE numero_comanda = ?1 AND status IN ('ABERTA', 'BLOQUEADA')",
        params![numero_comanda],
        |r| {
            Ok(ComandaOperacionalResp {
                id: Some(r.get(0)?),
                comanda_id: cache_id.clone(),
                numero_comanda,
                codigo_barras_qr: r.get(1)?,
                cliente_nome_informal: r.get(2)?,
                cliente_id: r.get(3)?,
                status: r.get(4)?,
                usuario_abertura_id: r.get(5)?,
                sessao_caixa_id: r.get(6)?,
                observacao: r.get(7)?,
                aberta_em: r.get(8)?,
                total_consumo_minor: 0,
            })
        }
    ).ok();

    let comanda = match op {
        Some(mut c) => {
            let total: i64 = conn.query_row(
                "SELECT COALESCE(SUM(total_item_minor), 0) FROM gourmet_itens 
                 WHERE origem_tipo = 'COMANDA' AND origem_id = ?1 AND cancelado = 0 AND status != 'TRANSFERIDO'",
                params![c.id],
                |r| r.get(0)
            ).unwrap_or(0);
            c.total_consumo_minor = total;
            c
        }
        None => ComandaOperacionalResp {
            id: None,
            comanda_id: cache_id,
            numero_comanda,
            codigo_barras_qr: None,
            cliente_nome_informal: None,
            cliente_id: None,
            status: "LIVRE".to_string(),
            usuario_abertura_id: None,
            sessao_caixa_id: None,
            observacao: None,
            aberta_em: None,
            total_consumo_minor: 0,
        }
    };

    let mut itens = Vec::new();
    if let Some(op_id) = &comanda.id {
        let mut stmt = conn.prepare(
            "SELECT id, origem_tipo, origem_id, produto_id, descricao_produto, codigo_produto,
                    quantidade_escala3, preco_unitario_minor, desconto_item_minor, acrescimo_item_minor,
                    total_item_minor, observacao_producao, local_producao_id, status, enviado_producao,
                    enviado_producao_em, cancelado, cancelado_em, motivo_cancelamento, supervisor_id,
                    autorizacao_id, criado_em
             FROM gourmet_itens 
             WHERE origem_tipo = 'COMANDA' AND origem_id = ?1 AND cancelado = 0 AND status != 'TRANSFERIDO'
             ORDER BY criado_em ASC"
        ).map_err(|e| e.to_string())?;

        let iter = stmt.query_map(params![op_id], |r| {
            Ok(GourmetItemResp {
                id: r.get(0)?,
                origem_tipo: r.get(1)?,
                origem_id: r.get(2)?,
                produto_id: r.get(3)?,
                descricao_produto: r.get(4)?,
                codigo_produto: r.get(5)?,
                quantidade_escala3: r.get(6)?,
                preco_unitario_minor: r.get(7)?,
                desconto_item_minor: r.get(8)?,
                acrescimo_item_minor: r.get(9)?,
                total_item_minor: r.get(10)?,
                observacao_producao: r.get(11)?,
                local_producao_id: r.get(12)?,
                status: r.get(13)?,
                enviado_producao: r.get::<_, i32>(14)? == 1,
                enviado_producao_em: r.get(15)?,
                cancelado: r.get::<_, i32>(16)? == 1,
                cancelado_em: r.get(17)?,
                motivo_cancelamento: r.get(18)?,
                supervisor_id: r.get(19)?,
                autorizacao_id: r.get(20)?,
                criado_em: r.get(21)?,
            })
        }).map_err(|e| e.to_string())?;

        for i in iter {
            if let Ok(item) = i {
                itens.push(item);
            }
        }
    }

    Ok(RespostaBase::ok("Comanda carregada com sucesso", ComandaDetalheResp { comanda, itens }))
}

// ============================================================================
// ITEMS COMMANDS
// ============================================================================

fn adicionar_item_gourmet_interno(
    conn: &mut Connection,
    dto: AdicionarItemGourmetReq,
) -> Result<GourmetItemResp, String> {
    if dto.quantidade_escala3 <= 0 {
        return Err("Quantidade deve ser maior que zero.".to_string());
    }
    if dto.preco_unitario_minor < 0 {
        return Err("Preço unitário não pode ser negativo.".to_string());
    }
    if dto.desconto_item_minor < 0 {
        return Err("Desconto não pode ser negativo.".to_string());
    }
    if dto.acrescimo_item_minor < 0 {
        return Err("Acréscimo não pode ser negativo.".to_string());
    }

    // 1. Verificar se a origem é ativa
    let op_status: String = match dto.origem_tipo.as_str() {
        "MESA" => conn.query_row(
            "SELECT status FROM mesas_operacionais WHERE id = ?1 AND status IN ('ABERTA', 'RESERVADA', 'BLOQUEADA')",
            params![dto.origem_id],
            |r| r.get(0)
        ).map_err(|_| "Mesa de origem não está ativa ou não existe.".to_string())?,
        "COMANDA" => conn.query_row(
            "SELECT status FROM comandas_operacionais WHERE id = ?1 AND status IN ('ABERTA', 'BLOQUEADA')",
            params![dto.origem_id],
            |r| r.get(0)
        ).map_err(|_| "Comanda de origem não está ativa ou não existe.".to_string())?,
        _ => return Err("Tipo de origem inválido. Use MESA ou COMANDA.".to_string()),
    };

    if op_status == "BLOQUEADA" {
        return Err("A origem está bloqueada. Não é possível adicionar novos itens.".to_string());
    }

    // 2. Verificar se o produto existe no cache e está ativo
    let (descricao, codigo): (String, String) = conn.query_row(
        "SELECT nome, codigo FROM produtos_cache WHERE produto_id = ?1 AND ativo = 1",
        params![dto.produto_id],
        |r| Ok((r.get(0)?, r.get(1)?))
    ).map_err(|_| "Produto não encontrado no catálogo ou inativo.".to_string())?;

    // 3. Cálculo matemático seguro de inteiros (Rule 15)
    // total = (quantidade * preco) / 1000 - desconto + acrescimo
    let bruto = dto.quantidade_escala3
        .checked_mul(dto.preco_unitario_minor)
        .ok_or("Overflow no cálculo de preço unitário e quantidade")?
        / 1000;
    
    let total = bruto
        .checked_sub(dto.desconto_item_minor)
        .and_then(|val| val.checked_add(dto.acrescimo_item_minor))
        .ok_or("Overflow no cálculo de totais do item")?;

    let total_final = total.max(0);

    let id = Uuid::new_v4().to_string();
    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO gourmet_itens (
            id, origem_tipo, origem_id, produto_id, descricao_produto, codigo_produto,
            quantidade_escala3, preco_unitario_minor, desconto_item_minor, acrescimo_item_minor,
            total_item_minor, observacao_producao, local_producao_id, status, enviado_producao, criado_em
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 'PENDENTE', 0, ?14)",
        params![
            &id,
            &dto.origem_tipo,
            &dto.origem_id,
            &dto.produto_id,
            &descricao,
            &codigo,
            dto.quantidade_escala3,
            dto.preco_unitario_minor,
            dto.desconto_item_minor,
            dto.acrescimo_item_minor,
            total_final,
            &dto.observacao_producao,
            &dto.local_producao_id,
            &agora
        ],
    ).map_err(|e| e.to_string())?;

    // Inserir outbox
    let event_name = match dto.origem_tipo.as_str() {
        "MESA" => "MESA_ITEM_ADICIONADO",
        _ => "COMANDA_ITEM_ADICIONADO",
    };

    inserir_outbox(
        &tx,
        event_name,
        json!({
            "item_id": &id,
            "origem_tipo": &dto.origem_tipo,
            "origem_id": &dto.origem_id,
            "produto_id": &dto.produto_id,
            "quantidade_escala3": dto.quantidade_escala3,
            "preco_unitario_minor": dto.preco_unitario_minor,
            "total_item_minor": total_final,
            "criado_em": &agora
        }),
    )?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(GourmetItemResp {
        id,
        origem_tipo: dto.origem_tipo,
        origem_id: dto.origem_id,
        produto_id: dto.produto_id,
        descricao_produto: descricao,
        codigo_produto: codigo,
        quantidade_escala3: dto.quantidade_escala3,
        preco_unitario_minor: dto.preco_unitario_minor,
        desconto_item_minor: dto.desconto_item_minor,
        acrescimo_item_minor: dto.acrescimo_item_minor,
        total_item_minor: total_final,
        observacao_producao: dto.observacao_producao,
        local_producao_id: dto.local_producao_id,
        status: "PENDENTE".to_string(),
        enviado_producao: false,
        enviado_producao_em: None,
        cancelado: false,
        cancelado_em: None,
        motivo_cancelamento: None,
        supervisor_id: None,
        autorizacao_id: None,
        criado_em: agora,
    })
}

fn cancelar_item_gourmet_interno(
    conn: &mut Connection,
    dto: CancelarItemGourmetReq,
) -> Result<String, String> {
    if dto.motivo_cancelamento.trim().is_empty() {
        return Err("Justificativa de cancelamento é obrigatória.".to_string());
    }

    // Buscar item
    let item: Option<(String, String, i32)> = conn
        .query_row(
            "SELECT origem_tipo, origem_id, enviado_producao FROM gourmet_itens WHERE id = ?1 AND cancelado = 0 AND status != 'TRANSFERIDO'",
            params![dto.item_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .ok();

    let (origem_tipo, origem_id, enviado_producao) = match item {
        Some(x) => x,
        None => return Err("Item não encontrado, já cancelado ou já transferido.".to_string()),
    };

    // Validar se exige supervisor (exige se já foi enviado para a produção)
    if enviado_producao == 1 && dto.supervisor_id.is_none() {
        return Err("Item já foi enviado para a produção. É necessária autorização de supervisor para cancelar.".to_string());
    }

    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE gourmet_itens
         SET status = 'CANCELADO',
             cancelado = 1,
             cancelado_em = ?1,
             usuario_cancelamento_id = ?2,
             motivo_cancelamento = ?3,
             supervisor_id = ?4,
             autorizacao_id = ?5
         WHERE id = ?6",
        params![
            &agora,
            &dto.usuario_cancelamento_id,
            &dto.motivo_cancelamento,
            &dto.supervisor_id,
            &dto.autorizacao_id,
            &dto.item_id
        ],
    ).map_err(|e| e.to_string())?;

    let event_name = match origem_tipo.as_str() {
        "MESA" => "MESA_ITEM_CANCELADO",
        _ => "COMANDA_ITEM_CANCELADO",
    };

    inserir_outbox(
        &tx,
        event_name,
        json!({
            "item_id": &dto.item_id,
            "origem_tipo": &origem_tipo,
            "origem_id": &origem_id,
            "usuario_cancelamento_id": &dto.usuario_cancelamento_id,
            "motivo": &dto.motivo_cancelamento,
            "supervisor_id": &dto.supervisor_id,
            "enviado_producao": enviado_producao == 1,
            "cancelado_em": &agora
        }),
    )?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(dto.item_id)
}

#[tauri::command]
pub async fn adicionar_item_mesa(
    dto: AdicionarItemGourmetReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<GourmetItemResp>, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let resp = adicionar_item_gourmet_interno(&mut conn, dto)?;
    Ok(RespostaBase::ok("Item adicionado à mesa com sucesso", resp))
}

#[tauri::command]
pub async fn cancelar_item_mesa(
    dto: CancelarItemGourmetReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<String>, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let item_id = cancelar_item_gourmet_interno(&mut conn, dto)?;
    Ok(RespostaBase::ok("Item de mesa cancelado com sucesso", item_id))
}

#[tauri::command]
pub async fn adicionar_item_comanda(
    dto: AdicionarItemGourmetReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<GourmetItemResp>, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let resp = adicionar_item_gourmet_interno(&mut conn, dto)?;
    Ok(RespostaBase::ok("Item adicionado à comanda com sucesso", resp))
}

#[tauri::command]
pub async fn cancelar_item_comanda(
    dto: CancelarItemGourmetReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<String>, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let item_id = cancelar_item_gourmet_interno(&mut conn, dto)?;
    Ok(RespostaBase::ok("Item de comanda cancelado com sucesso", item_id))
}
