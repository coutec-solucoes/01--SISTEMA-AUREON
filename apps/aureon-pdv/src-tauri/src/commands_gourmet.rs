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
        return Err("A origem esta bloqueada. Nao e possivel adicionar novos itens.".to_string());
    }

    // Ressalva 2: bloquear novos lancamentos se a origem ja tem venda EM_ANDAMENTO
    let venda_em_andamento: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM vendas WHERE origem_tipo = ?1 AND origem_id = ?2 AND status = 'EM_ANDAMENTO'",
        params![&dto.origem_tipo, &dto.origem_id],
        |r| r.get(0),
    ).unwrap_or(false);
    if venda_em_andamento {
        return Err(format!(
            "{} ja possui venda EM_ANDAMENTO. Finalize ou cancele o pagamento antes de adicionar itens.",
            dto.origem_tipo
        ));
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
        return Err("Justificativa de cancelamento e obrigatorio.".to_string());
    }

    // Buscar item: origem_tipo, origem_id, enviado_producao
    let item: Option<(String, String, i32)> = conn
        .query_row(
            "SELECT origem_tipo, origem_id, enviado_producao FROM gourmet_itens WHERE id = ?1 AND cancelado = 0 AND status != 'TRANSFERIDO'",
            params![dto.item_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .ok();

    let (origem_tipo, origem_id, enviado_producao) = match item {
        Some(x) => x,
        None => return Err("Item nao encontrado, ja cancelado ou ja transferido.".to_string()),
    };

    // Ressalva 2: bloquear cancelamento se a origem ja tem venda EM_ANDAMENTO
    let venda_em_andamento: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM vendas WHERE origem_tipo = ?1 AND origem_id = ?2 AND status = 'EM_ANDAMENTO'",
        params![&origem_tipo, &origem_id],
        |r| r.get(0),
    ).unwrap_or(false);
    if venda_em_andamento {
        return Err(format!(
            "{} ja possui uma venda EM_ANDAMENTO vinculada. Cancele ou finalize o pagamento antes de alterar itens.",
            origem_tipo
        ));
    }

    // Validar se exige supervisor (exige se ja foi enviado para a producao)
    if enviado_producao == 1 && dto.supervisor_id.is_none() {
        return Err("Item ja foi enviado para a producao. E necessaria autorizacao de supervisor para cancelar.".to_string());
    }

    // Ressalva 3: buscar envio de producao vinculado para gerar evento de cancelamento
    let envio_producao_id: Option<String> = if enviado_producao == 1 {
        conn.query_row(
            "SELECT envio_id FROM producao_envios_itens WHERE item_id = ?1 LIMIT 1",
            params![&dto.item_id],
            |r| r.get(0),
        ).ok()
    } else {
        None
    };

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

    // Se estava em producao, marcar producao_envios_itens como cancelamento
    if enviado_producao == 1 {
        tx.execute(
            "UPDATE producao_envios_itens SET cancelamento = 1 WHERE item_id = ?1",
            params![&dto.item_id],
        ).map_err(|e| e.to_string())?;
    }

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

    // Ressalva 3: eventos especificos de cancelamento de producao
    if enviado_producao == 1 {
        inserir_outbox(
            &tx,
            "ITEM_CANCELAMENTO_ENVIADO_PRODUCAO",
            json!({
                "item_id": &dto.item_id,
                "origem_tipo": &origem_tipo,
                "origem_id": &origem_id,
                "envio_id": &envio_producao_id,
                "usuario_cancelamento_id": &dto.usuario_cancelamento_id,
                "supervisor_id": &dto.supervisor_id,
                "motivo": &dto.motivo_cancelamento,
                "cancelado_em": &agora
            }),
        )?;

        if let Some(ref env_id) = envio_producao_id {
            inserir_outbox(
                &tx,
                "PRODUCAO_CANCELAMENTO_GERADO",
                json!({
                    "envio_id": env_id,
                    "item_id": &dto.item_id,
                    "origem_tipo": &origem_tipo,
                    "origem_id": &origem_id,
                    "usuario_cancelamento_id": &dto.usuario_cancelamento_id,
                    "cancelado_em": &agora
                }),
            )?;
        }
    }

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

// ============================================================================
// TRANSFERÊNCIAS — BLOCO 3
// ============================================================================

/// Helper: valida que o destino está ativo e é recebível (somente ABERTA)
fn validar_destino_transferencia(conn: &Connection, destino_tipo: &str, destino_id: &str) -> Result<(), String> {
    let status: String = match destino_tipo {
        "MESA" => conn.query_row(
            "SELECT status FROM mesas_operacionais WHERE id = ?1 AND status IN ('ABERTA', 'RESERVADA', 'BLOQUEADA')",
            params![destino_id],
            |r| r.get(0),
        ).map_err(|_| "Mesa de destino não existe ou não está ativa.".to_string())?,
        "COMANDA" => conn.query_row(
            "SELECT status FROM comandas_operacionais WHERE id = ?1 AND status IN ('ABERTA', 'BLOQUEADA')",
            params![destino_id],
            |r| r.get(0),
        ).map_err(|_| "Comanda de destino não existe ou não está ativa.".to_string())?,
        _ => return Err("Tipo de destino inválido. Use MESA ou COMANDA.".to_string()),
    };

    if status != "ABERTA" {
        return Err(format!(
            "O destino ({destino_tipo}) está com status '{status}'. Somente destinos ABERTOS podem receber transferências."
        ));
    }
    Ok(())
}

/// Helper: valida que a origem está ABERTA e retorna o status
fn validar_origem_aberta(conn: &Connection, origem_tipo: &str, origem_id: &str) -> Result<(), String> {
    let status: String = match origem_tipo {
        "MESA" => conn.query_row(
            "SELECT status FROM mesas_operacionais WHERE id = ?1 AND status IN ('ABERTA', 'RESERVADA', 'BLOQUEADA')",
            params![origem_id],
            |r| r.get(0),
        ).map_err(|_| "Mesa de origem não encontrada ou não está ativa.".to_string())?,
        "COMANDA" => conn.query_row(
            "SELECT status FROM comandas_operacionais WHERE id = ?1 AND status IN ('ABERTA', 'BLOQUEADA')",
            params![origem_id],
            |r| r.get(0),
        ).map_err(|_| "Comanda de origem não encontrada ou não está ativa.".to_string())?,
        _ => return Err("Tipo de origem inválido. Use MESA ou COMANDA.".to_string()),
    };
    if status != "ABERTA" {
        return Err(format!("Origem ({origem_tipo}) está com status '{status}'. Somente origens ABERTAS podem ser transferidas."));
    }
    Ok(())
}

/// Helper interno para criar item espelho no destino a partir de um item de origem
fn criar_item_destino_interno(
    tx: &rusqlite::Transaction,
    item_origem_id: &str,
    destino_tipo: &str,
    destino_id: &str,
    quantidade_escala3: i64,
) -> Result<(String, i64, i64, i64, i64, String, String, Option<String>, String), String> {
    // Buscar dados do item de origem
    let (produto_id, descricao, codigo, preco_minor, desc_minor, acr_minor, obs, local_prod): (String, String, String, i64, i64, i64, Option<String>, String) = tx.query_row(
        "SELECT produto_id, descricao_produto, codigo_produto, preco_unitario_minor, desconto_item_minor, acrescimo_item_minor, observacao_producao, local_producao_id
         FROM gourmet_itens WHERE id = ?1 AND cancelado = 0 AND status != 'TRANSFERIDO'",
        params![item_origem_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?))
    ).map_err(|_| format!("Item de origem '{item_origem_id}' não encontrado ou já cancelado/transferido."))?;

    // Calcular proporcional: (quantidade_nova * preco) / 1000 - desconto_proporcional
    let total_novo = quantidade_escala3
        .checked_mul(preco_minor)
        .ok_or("Overflow no cálculo do item de destino")?
        / 1000;
    let total_novo = (total_novo - desc_minor + acr_minor).max(0);

    let novo_id = Uuid::new_v4().to_string();
    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    tx.execute(
        "INSERT INTO gourmet_itens (
            id, origem_tipo, origem_id, produto_id, descricao_produto, codigo_produto,
            quantidade_escala3, preco_unitario_minor, desconto_item_minor, acrescimo_item_minor,
            total_item_minor, observacao_producao, local_producao_id, status, enviado_producao, criado_em
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 'PENDENTE', 0, ?14)",
        params![
            &novo_id, destino_tipo, destino_id, &produto_id, &descricao, &codigo,
            quantidade_escala3, preco_minor, desc_minor, acr_minor,
            total_novo, &obs, &local_prod, &agora
        ],
    ).map_err(|e| e.to_string())?;

    Ok((novo_id, quantidade_escala3, preco_minor, desc_minor, acr_minor, produto_id, descricao, obs, agora))
}

#[tauri::command]
pub async fn transferir_mesa_total(
    dto: TransferirTotalReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<TransferenciaResp>, String> {
    info!(componente = "aureon-pdv::commands_gourmet", origem_id = %dto.origem_id, "Chamada: transferir_mesa_total");

    if dto.motivo.trim().is_empty() {
        return Err("Motivo de transferência é obrigatório.".to_string());
    }
    if dto.origem_id == dto.destino_id {
        return Err("Origem e destino não podem ser iguais.".to_string());
    }

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    validar_origem_aberta(&conn, "MESA", &dto.origem_id)?;
    validar_destino_transferencia(&conn, &dto.destino_tipo, &dto.destino_id)?;

    // Buscar itens ativos da mesa de origem — escopo fechado para liberar borrow antes de conn.transaction()
    let itens_origem: Vec<(String, i64)> = {
        let mut stmt = conn.prepare(
            "SELECT id, quantidade_escala3 FROM gourmet_itens
             WHERE origem_tipo = 'MESA' AND origem_id = ?1 AND cancelado = 0 AND status NOT IN ('TRANSFERIDO', 'CANCELADO')"
        ).map_err(|e| e.to_string())?;
        let x = stmt.query_map(params![&dto.origem_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        }).map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
        x
    };

    if itens_origem.is_empty() {
        return Err("A mesa de origem não possui itens ativos para transferir.".to_string());
    }

    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let transferencia_id = Uuid::new_v4().to_string();
    let total_itens = itens_origem.len() as i64;

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // Registrar transferência
    tx.execute(
        "INSERT INTO gourmet_transferencias (id, origem_tipo, origem_id, destino_tipo, destino_id, usuario_id, motivo, transferencia_total, criado_em)
         VALUES (?1, 'MESA', ?2, ?3, ?4, ?5, ?6, 1, ?7)",
        params![&transferencia_id, &dto.origem_id, &dto.destino_tipo, &dto.destino_id, &dto.usuario_id, &dto.motivo, &agora],
    ).map_err(|e| e.to_string())?;

    // Para cada item: criar novo no destino, marcar origem como TRANSFERIDO, registrar rastreabilidade
    for (item_id, qtd) in &itens_origem {
        let (novo_id, ..) = criar_item_destino_interno(&tx, item_id, &dto.destino_tipo, &dto.destino_id, *qtd)?;

        // Marcar item de origem como TRANSFERIDO
        tx.execute(
            "UPDATE gourmet_itens SET status = 'TRANSFERIDO' WHERE id = ?1",
            params![item_id],
        ).map_err(|e| e.to_string())?;

        // Rastreabilidade
        tx.execute(
            "INSERT INTO gourmet_transferencias_itens (transferencia_id, item_origem_id, quantidade_transferida_escala3, item_destino_id)
             VALUES (?1, ?2, ?3, ?4)",
            params![&transferencia_id, item_id, qtd, &novo_id],
        ).map_err(|e| e.to_string())?;
    }

    // Marcar mesa de origem como FECHADA (consumo migrado)
    tx.execute(
        "UPDATE mesas_operacionais SET status = 'FECHADA', fechada_em = ?1 WHERE id = ?2",
        params![&agora, &dto.origem_id],
    ).map_err(|e| e.to_string())?;

    inserir_outbox(&tx, "MESA_TRANSFERIDA", json!({
        "transferencia_id": &transferencia_id,
        "origem_tipo": "MESA",
        "origem_id": &dto.origem_id,
        "destino_tipo": &dto.destino_tipo,
        "destino_id": &dto.destino_id,
        "usuario_id": &dto.usuario_id,
        "motivo": &dto.motivo,
        "total_itens": total_itens,
        "transferencia_total": true,
        "criado_em": &agora
    }))?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(RespostaBase::ok("Mesa transferida com sucesso", TransferenciaResp {
        transferencia_id,
        origem_tipo: "MESA".to_string(),
        origem_id: dto.origem_id,
        destino_tipo: dto.destino_tipo,
        destino_id: dto.destino_id,
        total: true,
        itens_transferidos: total_itens,
        criado_em: agora,
    }))
}

#[tauri::command]
pub async fn transferir_itens_mesa(
    dto: TransferirItensReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<TransferenciaResp>, String> {
    info!(componente = "aureon-pdv::commands_gourmet", "Chamada: transferir_itens_mesa");

    if dto.motivo.trim().is_empty() {
        return Err("Motivo de transferência é obrigatório.".to_string());
    }
    if dto.itens.is_empty() {
        return Err("Selecione ao menos um item para transferir.".to_string());
    }
    if dto.origem_id == dto.destino_id {
        return Err("Origem e destino não podem ser iguais.".to_string());
    }

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    validar_destino_transferencia(&conn, &dto.destino_tipo, &dto.destino_id)?;

    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let transferencia_id = Uuid::new_v4().to_string();

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO gourmet_transferencias (id, origem_tipo, origem_id, destino_tipo, destino_id, usuario_id, motivo, transferencia_total, criado_em)
         VALUES (?1, 'MESA', ?2, ?3, ?4, ?5, ?6, 0, ?7)",
        params![&transferencia_id, &dto.origem_id, &dto.destino_tipo, &dto.destino_id, &dto.usuario_id, &dto.motivo, &agora],
    ).map_err(|e| e.to_string())?;

    let mut total_itens = 0i64;

    for item_req in &dto.itens {
        if item_req.quantidade_escala3 <= 0 {
            return Err(format!("Quantidade do item '{}' deve ser maior que zero.", item_req.item_origem_id));
        }

        // Validar que o item existe e pegar quantidade disponível
        let qtd_disponivel: i64 = tx.query_row(
            "SELECT quantidade_escala3 FROM gourmet_itens WHERE id = ?1 AND origem_tipo = 'MESA' AND origem_id = ?2 AND cancelado = 0 AND status NOT IN ('TRANSFERIDO', 'CANCELADO')",
            params![&item_req.item_origem_id, &dto.origem_id],
            |r| r.get(0),
        ).map_err(|_| format!("Item '{}' não encontrado, cancelado ou já transferido.", item_req.item_origem_id))?;

        if item_req.quantidade_escala3 > qtd_disponivel {
            return Err(format!(
                "Quantidade a transferir ({}) excede o disponível no item ({}).",
                item_req.quantidade_escala3, qtd_disponivel
            ));
        }

        let (novo_id, ..) = criar_item_destino_interno(
            &tx, &item_req.item_origem_id,
            &dto.destino_tipo, &dto.destino_id,
            item_req.quantidade_escala3
        )?;

        // Transferência total do item ou parcial?
        if item_req.quantidade_escala3 == qtd_disponivel {
            // Marcar o item de origem como TRANSFERIDO
            tx.execute(
                "UPDATE gourmet_itens SET status = 'TRANSFERIDO' WHERE id = ?1",
                params![&item_req.item_origem_id],
            ).map_err(|e| e.to_string())?;
        } else {
            // Reduzir a quantidade do item de origem
            let nova_qtd = qtd_disponivel - item_req.quantidade_escala3;
            // Recalcular total do item de origem com a nova quantidade
            let preco_minor: i64 = tx.query_row(
                "SELECT preco_unitario_minor FROM gourmet_itens WHERE id = ?1",
                params![&item_req.item_origem_id],
                |r| r.get(0)
            ).unwrap_or(0);
            let novo_total = (nova_qtd * preco_minor) / 1000;
            tx.execute(
                "UPDATE gourmet_itens SET quantidade_escala3 = ?1, total_item_minor = ?2 WHERE id = ?3",
                params![nova_qtd, novo_total, &item_req.item_origem_id],
            ).map_err(|e| e.to_string())?;
        }

        // Rastreabilidade
        tx.execute(
            "INSERT INTO gourmet_transferencias_itens (transferencia_id, item_origem_id, quantidade_transferida_escala3, item_destino_id)
             VALUES (?1, ?2, ?3, ?4)",
            params![&transferencia_id, &item_req.item_origem_id, item_req.quantidade_escala3, &novo_id],
        ).map_err(|e| e.to_string())?;

        total_itens += 1;
    }

    inserir_outbox(&tx, "MESA_ITENS_TRANSFERIDOS", json!({
        "transferencia_id": &transferencia_id,
        "origem_tipo": "MESA",
        "origem_id": &dto.origem_id,
        "destino_tipo": &dto.destino_tipo,
        "destino_id": &dto.destino_id,
        "usuario_id": &dto.usuario_id,
        "itens_transferidos": total_itens,
        "criado_em": &agora
    }))?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(RespostaBase::ok("Itens de mesa transferidos com sucesso", TransferenciaResp {
        transferencia_id,
        origem_tipo: "MESA".to_string(),
        origem_id: dto.origem_id,
        destino_tipo: dto.destino_tipo,
        destino_id: dto.destino_id,
        total: false,
        itens_transferidos: total_itens,
        criado_em: agora,
    }))
}

#[tauri::command]
pub async fn transferir_comanda_total(
    dto: TransferirTotalReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<TransferenciaResp>, String> {
    info!(componente = "aureon-pdv::commands_gourmet", "Chamada: transferir_comanda_total");

    if dto.motivo.trim().is_empty() {
        return Err("Motivo de transferência é obrigatório.".to_string());
    }
    if dto.origem_id == dto.destino_id {
        return Err("Origem e destino não podem ser iguais.".to_string());
    }

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    validar_origem_aberta(&conn, "COMANDA", &dto.origem_id)?;
    validar_destino_transferencia(&conn, &dto.destino_tipo, &dto.destino_id)?;

    let itens_origem: Vec<(String, i64)> = {
        let mut stmt = conn.prepare(
            "SELECT id, quantidade_escala3 FROM gourmet_itens
             WHERE origem_tipo = 'COMANDA' AND origem_id = ?1 AND cancelado = 0 AND status NOT IN ('TRANSFERIDO', 'CANCELADO')"
        ).map_err(|e| e.to_string())?;
        let x = stmt.query_map(params![&dto.origem_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        }).map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
        x
    };

    if itens_origem.is_empty() {
        return Err("A comanda de origem não possui itens ativos para transferir.".to_string());
    }

    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let transferencia_id = Uuid::new_v4().to_string();
    let total_itens = itens_origem.len() as i64;

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO gourmet_transferencias (id, origem_tipo, origem_id, destino_tipo, destino_id, usuario_id, motivo, transferencia_total, criado_em)
         VALUES (?1, 'COMANDA', ?2, ?3, ?4, ?5, ?6, 1, ?7)",
        params![&transferencia_id, &dto.origem_id, &dto.destino_tipo, &dto.destino_id, &dto.usuario_id, &dto.motivo, &agora],
    ).map_err(|e| e.to_string())?;

    for (item_id, qtd) in &itens_origem {
        let (novo_id, ..) = criar_item_destino_interno(&tx, item_id, &dto.destino_tipo, &dto.destino_id, *qtd)?;
        tx.execute("UPDATE gourmet_itens SET status = 'TRANSFERIDO' WHERE id = ?1", params![item_id]).map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO gourmet_transferencias_itens (transferencia_id, item_origem_id, quantidade_transferida_escala3, item_destino_id) VALUES (?1, ?2, ?3, ?4)",
            params![&transferencia_id, item_id, qtd, &novo_id],
        ).map_err(|e| e.to_string())?;
    }

    tx.execute(
        "UPDATE comandas_operacionais SET status = 'FECHADA', fechada_em = ?1 WHERE id = ?2",
        params![&agora, &dto.origem_id],
    ).map_err(|e| e.to_string())?;

    inserir_outbox(&tx, "COMANDA_TRANSFERIDA", json!({
        "transferencia_id": &transferencia_id,
        "origem_tipo": "COMANDA",
        "origem_id": &dto.origem_id,
        "destino_tipo": &dto.destino_tipo,
        "destino_id": &dto.destino_id,
        "usuario_id": &dto.usuario_id,
        "motivo": &dto.motivo,
        "total_itens": total_itens,
        "transferencia_total": true,
        "criado_em": &agora
    }))?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(RespostaBase::ok("Comanda transferida com sucesso", TransferenciaResp {
        transferencia_id,
        origem_tipo: "COMANDA".to_string(),
        origem_id: dto.origem_id,
        destino_tipo: dto.destino_tipo,
        destino_id: dto.destino_id,
        total: true,
        itens_transferidos: total_itens,
        criado_em: agora,
    }))
}

#[tauri::command]
pub async fn transferir_itens_comanda(
    dto: TransferirItensReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<TransferenciaResp>, String> {
    info!(componente = "aureon-pdv::commands_gourmet", "Chamada: transferir_itens_comanda");

    if dto.motivo.trim().is_empty() {
        return Err("Motivo de transferência é obrigatório.".to_string());
    }
    if dto.itens.is_empty() {
        return Err("Selecione ao menos um item para transferir.".to_string());
    }
    if dto.origem_id == dto.destino_id {
        return Err("Origem e destino não podem ser iguais.".to_string());
    }

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    validar_destino_transferencia(&conn, &dto.destino_tipo, &dto.destino_id)?;

    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let transferencia_id = Uuid::new_v4().to_string();

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO gourmet_transferencias (id, origem_tipo, origem_id, destino_tipo, destino_id, usuario_id, motivo, transferencia_total, criado_em)
         VALUES (?1, 'COMANDA', ?2, ?3, ?4, ?5, ?6, 0, ?7)",
        params![&transferencia_id, &dto.origem_id, &dto.destino_tipo, &dto.destino_id, &dto.usuario_id, &dto.motivo, &agora],
    ).map_err(|e| e.to_string())?;

    let mut total_itens = 0i64;

    for item_req in &dto.itens {
        if item_req.quantidade_escala3 <= 0 {
            return Err(format!("Quantidade do item '{}' deve ser maior que zero.", item_req.item_origem_id));
        }

        let qtd_disponivel: i64 = tx.query_row(
            "SELECT quantidade_escala3 FROM gourmet_itens WHERE id = ?1 AND origem_tipo = 'COMANDA' AND origem_id = ?2 AND cancelado = 0 AND status NOT IN ('TRANSFERIDO', 'CANCELADO')",
            params![&item_req.item_origem_id, &dto.origem_id],
            |r| r.get(0),
        ).map_err(|_| format!("Item '{}' não encontrado, cancelado ou já transferido.", item_req.item_origem_id))?;

        if item_req.quantidade_escala3 > qtd_disponivel {
            return Err(format!(
                "Quantidade a transferir ({}) excede o disponível ({}).",
                item_req.quantidade_escala3, qtd_disponivel
            ));
        }

        let (novo_id, ..) = criar_item_destino_interno(&tx, &item_req.item_origem_id, &dto.destino_tipo, &dto.destino_id, item_req.quantidade_escala3)?;

        if item_req.quantidade_escala3 == qtd_disponivel {
            tx.execute("UPDATE gourmet_itens SET status = 'TRANSFERIDO' WHERE id = ?1", params![&item_req.item_origem_id]).map_err(|e| e.to_string())?;
        } else {
            let nova_qtd = qtd_disponivel - item_req.quantidade_escala3;
            let preco_minor: i64 = tx.query_row(
                "SELECT preco_unitario_minor FROM gourmet_itens WHERE id = ?1",
                params![&item_req.item_origem_id], |r| r.get(0)
            ).unwrap_or(0);
            let novo_total = (nova_qtd * preco_minor) / 1000;
            tx.execute(
                "UPDATE gourmet_itens SET quantidade_escala3 = ?1, total_item_minor = ?2 WHERE id = ?3",
                params![nova_qtd, novo_total, &item_req.item_origem_id],
            ).map_err(|e| e.to_string())?;
        }

        tx.execute(
            "INSERT INTO gourmet_transferencias_itens (transferencia_id, item_origem_id, quantidade_transferida_escala3, item_destino_id) VALUES (?1, ?2, ?3, ?4)",
            params![&transferencia_id, &item_req.item_origem_id, item_req.quantidade_escala3, &novo_id],
        ).map_err(|e| e.to_string())?;

        total_itens += 1;
    }

    inserir_outbox(&tx, "COMANDA_ITENS_TRANSFERIDOS", json!({
        "transferencia_id": &transferencia_id,
        "origem_tipo": "COMANDA",
        "origem_id": &dto.origem_id,
        "destino_tipo": &dto.destino_tipo,
        "destino_id": &dto.destino_id,
        "usuario_id": &dto.usuario_id,
        "itens_transferidos": total_itens,
        "criado_em": &agora
    }))?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(RespostaBase::ok("Itens de comanda transferidos com sucesso", TransferenciaResp {
        transferencia_id,
        origem_tipo: "COMANDA".to_string(),
        origem_id: dto.origem_id,
        destino_tipo: dto.destino_tipo,
        destino_id: dto.destino_id,
        total: false,
        itens_transferidos: total_itens,
        criado_em: agora,
    }))
}

// ============================================================================
// PRODUÇÃO — BLOCO 3
// ============================================================================

/// Gera o texto TXT de produção para um setor
fn gerar_texto_envio(
    origem_tipo: &str,
    origem_id: &str,
    setor_id: &str,
    itens: &[ProducaoEnvioItemResp],
    agora: &str,
) -> String {
    use std::fmt::Write;
    let mut txt = String::new();
    let _ = writeln!(txt, "================================");
    let _ = writeln!(txt, "  PEDIDO DE PRODUÇÃO");
    let _ = writeln!(txt, "================================");
    let _ = writeln!(txt, "Origem:  {} {}", origem_tipo, &origem_id[..8.min(origem_id.len())]);
    let _ = writeln!(txt, "Setor:   {}", setor_id);
    let _ = writeln!(txt, "Data/Hr: {}", agora);
    let _ = writeln!(txt, "--------------------------------");
    for item in itens {
        let qtd_f = item.quantidade_escala3 as f64 / 1000.0;
        let _ = writeln!(txt, "{} x {}", qtd_f, item.descricao_produto);
        if let Some(obs) = &item.observacao_producao {
            if !obs.is_empty() {
                let _ = writeln!(txt, "  OBS: {}", obs);
            }
        }
        if item.cancelamento {
            let _ = writeln!(txt, "  *** CANCELADO ***");
        }
    }
    let _ = writeln!(txt, "================================");
    txt
}

#[tauri::command]
pub async fn enviar_itens_producao(
    dto: EnviarProducaoReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<Vec<ProducaoEnvioResp>>, String> {
    info!(componente = "aureon-pdv::commands_gourmet", "Chamada: enviar_itens_producao");

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    // Buscar itens PENDENTES — escopo fechado para liberar borrow antes de conn.transaction()
    let itens_pendentes: Vec<(String, String, String, i64, Option<String>, String)> = {
        let mut stmt = conn.prepare(
            "SELECT id, produto_id, descricao_produto, quantidade_escala3, observacao_producao, local_producao_id
             FROM gourmet_itens
             WHERE origem_tipo = ?1 AND origem_id = ?2 AND status = 'PENDENTE' AND cancelado = 0
             ORDER BY local_producao_id, criado_em"
        ).map_err(|e| e.to_string())?;
        let x = stmt.query_map(
            params![&dto.origem_tipo, &dto.origem_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?))
        ).map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
        x
    };

    if itens_pendentes.is_empty() {
        return Err("Não há itens PENDENTES para enviar à produção.".to_string());
    }

    // Agrupar por setor
    let mut por_setor: std::collections::HashMap<String, Vec<(String, String, String, i64, Option<String>)>> = std::collections::HashMap::new();
    for (item_id, produto_id, descricao, qtd, obs, setor_id) in &itens_pendentes {
        por_setor.entry(setor_id.clone()).or_default().push((
            item_id.clone(), produto_id.clone(), descricao.clone(), *qtd, obs.clone()
        ));
    }

    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut envios_resp: Vec<ProducaoEnvioResp> = Vec::new();

    for (setor_id, itens_setor) in &por_setor {
        let envio_id = Uuid::new_v4().to_string();

        tx.execute(
            "INSERT INTO producao_envios (id, origem_tipo, origem_id, setor_producao_id, usuario_id, status, criado_em)
             VALUES (?1, ?2, ?3, ?4, ?5, 'GERADO', ?6)",
            params![&envio_id, &dto.origem_tipo, &dto.origem_id, setor_id, &dto.usuario_id, &agora],
        ).map_err(|e| e.to_string())?;

        let mut itens_resp: Vec<ProducaoEnvioItemResp> = Vec::new();

        for (item_id, produto_id, descricao, qtd, obs) in itens_setor {
            tx.execute(
                "INSERT INTO producao_envios_itens (envio_id, item_id, produto_id, descricao_produto, quantidade_escala3, observacao_producao, cancelamento, criado_em)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7)",
                params![&envio_id, item_id, produto_id, descricao, qtd, obs, &agora],
            ).map_err(|e| e.to_string())?;

            // Marcar item como enviado para produção
            tx.execute(
                "UPDATE gourmet_itens SET status = 'ENVIADO_PRODUCAO', enviado_producao = 1, enviado_producao_em = ?1 WHERE id = ?2",
                params![&agora, item_id],
            ).map_err(|e| e.to_string())?;

            itens_resp.push(ProducaoEnvioItemResp {
                item_id: item_id.clone(),
                produto_id: produto_id.clone(),
                descricao_produto: descricao.clone(),
                quantidade_escala3: *qtd,
                observacao_producao: obs.clone(),
                cancelamento: false,
            });
        }

        let texto = gerar_texto_envio(&dto.origem_tipo, &dto.origem_id, setor_id, &itens_resp, &agora);

        inserir_outbox(&tx, "PRODUCAO_ENVIO_GERADO", json!({
            "envio_id": &envio_id,
            "origem_tipo": &dto.origem_tipo,
            "origem_id": &dto.origem_id,
            "setor_producao_id": setor_id,
            "usuario_id": &dto.usuario_id,
            "total_itens": itens_resp.len(),
            "criado_em": &agora
        }))?;

        // Outbox por cada item enviado
        for item in &itens_resp {
            inserir_outbox(&tx, "ITEM_ENVIADO_PRODUCAO", json!({
                "envio_id": &envio_id,
                "item_id": &item.item_id,
                "origem_tipo": &dto.origem_tipo,
                "origem_id": &dto.origem_id,
                "setor_producao_id": setor_id,
                "enviado_em": &agora
            }))?;
        }

        envios_resp.push(ProducaoEnvioResp {
            id: envio_id,
            origem_tipo: dto.origem_tipo.clone(),
            origem_id: dto.origem_id.clone(),
            setor_producao_id: setor_id.clone(),
            usuario_id: dto.usuario_id.clone(),
            status: "GERADO".to_string(),
            texto_producao: texto,
            itens: itens_resp,
            criado_em: agora.clone(),
        });
    }

    tx.commit().map_err(|e| e.to_string())?;

    Ok(RespostaBase::ok(
        &format!("{} envio(s) de produção gerado(s) com sucesso", envios_resp.len()),
        envios_resp,
    ))
}

#[tauri::command]
pub async fn gerar_texto_producao(
    envio_id: String,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<String>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let (origem_tipo, origem_id, setor_id, criado_em): (String, String, String, String) = conn.query_row(
        "SELECT origem_tipo, origem_id, setor_producao_id, criado_em FROM producao_envios WHERE id = ?1",
        params![&envio_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
    ).map_err(|_| format!("Envio de produção '{envio_id}' não encontrado."))?;

    let mut stmt = conn.prepare(
        "SELECT item_id, produto_id, descricao_produto, quantidade_escala3, observacao_producao, cancelamento
         FROM producao_envios_itens WHERE envio_id = ?1 ORDER BY rowid"
    ).map_err(|e| e.to_string())?;

    let itens: Vec<ProducaoEnvioItemResp> = stmt.query_map(params![&envio_id], |r| {
        Ok(ProducaoEnvioItemResp {
            item_id: r.get(0)?,
            produto_id: r.get(1)?,
            descricao_produto: r.get(2)?,
            quantidade_escala3: r.get(3)?,
            observacao_producao: r.get(4)?,
            cancelamento: r.get::<_, i32>(5)? == 1,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    let texto = gerar_texto_envio(&origem_tipo, &origem_id, &setor_id, &itens, &criado_em);

    Ok(RespostaBase::ok("Texto de produção gerado", texto))
}

#[tauri::command]
pub async fn reimprimir_envio_producao(
    dto: ReimpressaoProducaoReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<String>, String> {
    info!(componente = "aureon-pdv::commands_gourmet", envio_id = %dto.envio_id, "Chamada: reimprimir_envio_producao");

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let existe: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM producao_envios WHERE id = ?1",
        params![&dto.envio_id],
        |r| r.get(0),
    ).unwrap_or(false);

    if !existe {
        return Err(format!("Envio de produção '{}' não encontrado.", dto.envio_id));
    }

    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE producao_envios SET reimpresso_em = ?1, status = 'IMPRESSO_PREPARADO' WHERE id = ?2",
        params![&agora, &dto.envio_id],
    ).map_err(|e| e.to_string())?;

    inserir_outbox(&tx, "PRODUCAO_REIMPRESSAO_GERADA", json!({
        "envio_id": &dto.envio_id,
        "usuario_id": &dto.usuario_id,
        "reimpresso_em": &agora
    }))?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(RespostaBase::ok("Envio de produção registrado como reimpresso", dto.envio_id))
}

#[tauri::command]
pub async fn listar_todos_envios_producao(
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<Vec<ProducaoEnvioResp>>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id, origem_tipo, origem_id, setor_producao_id, usuario_id, status, criado_em
         FROM producao_envios
         ORDER BY criado_em DESC LIMIT 50"
    ).map_err(|e| e.to_string())?;

    let envios: Vec<(String, String, String, String, String, String, String)> = stmt.query_map(
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?))
    ).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    let mut result = Vec::new();
    for e in envios {
        result.push(ProducaoEnvioResp {
            id: e.0,
            origem_tipo: e.1,
            origem_id: e.2,
            setor_producao_id: e.3,
            usuario_id: e.4,
            status: e.5,
            texto_producao: String::new(), // Não carregado na lista geral
            itens: Vec::new(),             // Não carregado na lista geral
            criado_em: e.6,
        });
    }

    Ok(RespostaBase::ok("Todos envios de produção listados", result))
}

#[tauri::command]
pub async fn listar_envios_producao(
    origem_tipo: String,
    origem_id: String,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<Vec<ProducaoEnvioResp>>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id, origem_tipo, origem_id, setor_producao_id, usuario_id, status, criado_em
         FROM producao_envios
         WHERE origem_tipo = ?1 AND origem_id = ?2
         ORDER BY criado_em DESC"
    ).map_err(|e| e.to_string())?;

    let envios: Vec<(String, String, String, String, String, String, String)> = stmt.query_map(
        params![&origem_tipo, &origem_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?))
    ).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    let mut resultado = Vec::new();

    for (env_id, ot, oi, setor, usuario, status, criado) in envios {
        let mut stmt2 = conn.prepare(
            "SELECT item_id, produto_id, descricao_produto, quantidade_escala3, observacao_producao, cancelamento
             FROM producao_envios_itens WHERE envio_id = ?1 ORDER BY rowid"
        ).map_err(|e| e.to_string())?;

        let itens: Vec<ProducaoEnvioItemResp> = stmt2.query_map(params![&env_id], |r| {
            Ok(ProducaoEnvioItemResp {
                item_id: r.get(0)?,
                produto_id: r.get(1)?,
                descricao_produto: r.get(2)?,
                quantidade_escala3: r.get(3)?,
                observacao_producao: r.get(4)?,
                cancelamento: r.get::<_, i32>(5)? == 1,
            })
        }).map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

        let texto = gerar_texto_envio(&ot, &oi, &setor, &itens, &criado);

        resultado.push(ProducaoEnvioResp {
            id: env_id,
            origem_tipo: ot,
            origem_id: oi,
            setor_producao_id: setor,
            usuario_id: usuario,
            status,
            texto_producao: texto,
            itens,
            criado_em: criado,
        });
    }

    Ok(RespostaBase::ok("Envios de produção listados", resultado))
}

// ============================================================================
// FECHAMENTO EM VENDA — BLOCO 3
// ============================================================================

/// Helper interno para fechar mesa ou comanda em venda
/// Cria a venda EM_ANDAMENTO sem numero_venda, copia os itens de consumo.
/// Status da origem: mesa/comanda permanece ABERTA — o status FECHADA só é
/// aplicado quando o pagamento é finalizado (via `finalizar_venda` da Fase 7).
/// Isso evita inconsistência de dupla conversão: uma mesa/comanda já tem
/// a venda em andamento sendo mapeada por `vendas.origem_id`.
fn fechar_em_venda_interno(
    conn: &mut rusqlite::Connection,
    origem_tipo: &str,
    origem_id: &str,
    usuario_id: &str,
    sessao_caixa_id: &str,
) -> Result<FechamentoEmVendaResp, String> {
    // 1. Validar caixa aberto
    let sessao_ok: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM sessoes_caixa WHERE id = ?1 AND status = 'ABERTO'",
        params![sessao_caixa_id],
        |r| r.get(0),
    ).unwrap_or(false);
    if !sessao_ok {
        return Err("Sessão de caixa não encontrada ou não está aberta.".to_string());
    }

    // 2. Validar origem ativa
    let status_origem: String = match origem_tipo {
        "MESA" => conn.query_row(
            "SELECT status FROM mesas_operacionais WHERE id = ?1 AND status IN ('ABERTA', 'RESERVADA', 'BLOQUEADA')",
            params![origem_id],
            |r| r.get(0),
        ).map_err(|_| "Mesa não encontrada ou não está ativa.".to_string())?,
        "COMANDA" => conn.query_row(
            "SELECT status FROM comandas_operacionais WHERE id = ?1 AND status IN ('ABERTA', 'BLOQUEADA')",
            params![origem_id],
            |r| r.get(0),
        ).map_err(|_| "Comanda não encontrada ou não está ativa.".to_string())?,
        _ => return Err("Tipo de origem inválido.".to_string()),
    };

    if status_origem == "BLOQUEADA" {
        return Err(format!(
            "A {origem_tipo} está BLOQUEADA. Desbloqueie antes de fechar em venda."
        ));
    }

    // 3. Verificar se já existe uma venda EM_ANDAMENTO para esta origem
    // Evita dupla conversão
    let ja_em_venda: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM vendas WHERE origem_tipo = ?1 AND origem_id = ?2 AND status = 'EM_ANDAMENTO'",
        params![origem_tipo, origem_id],
        |r| r.get(0),
    ).unwrap_or(false);
    if ja_em_venda {
        return Err(format!(
            "{origem_tipo} já possui uma venda EM_ANDAMENTO vinculada. Finalize ou cancele antes de gerar outra."
        ));
    }

    // 4. Buscar itens ativos para copiar — escopo fechado para liberar borrow antes de conn.transaction()
    let itens_consumo: Vec<(String, String, String, Option<String>, i64, i64, i64, i64, i64)> = {
        let mut stmt = conn.prepare(
            "SELECT id, produto_id, descricao_produto, codigo_produto,
                    quantidade_escala3, preco_unitario_minor, desconto_item_minor,
                    acrescimo_item_minor, total_item_minor
             FROM gourmet_itens
             WHERE origem_tipo = ?1 AND origem_id = ?2
               AND cancelado = 0
               AND status NOT IN ('CANCELADO', 'TRANSFERIDO')
             ORDER BY criado_em ASC"
        ).map_err(|e| e.to_string())?;
        let x = stmt.query_map(
            params![origem_tipo, origem_id],
            |r| Ok((
                r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?,
                r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?,
            ))
        ).map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
        x
    };

    if itens_consumo.is_empty() {
        return Err(format!("{origem_tipo} não possui itens de consumo ativos para fechar em venda."));
    }

    // 5. Calcular totais
    let total_minor: i64 = itens_consumo.iter().map(|(_, _, _, _, _, _, _, _, t)| t).sum();
    let total_itens = itens_consumo.len() as i64;

    let venda_id = Uuid::new_v4().to_string();
    let tipo_venda = origem_tipo.to_string(); // 'MESA' ou 'COMANDA'
    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // 6. Criar venda EM_ANDAMENTO com numero_venda = NULL
    tx.execute(
        "INSERT INTO vendas (id, numero_venda, sessao_caixa_id, usuario_id, status, tipo_venda,
                             subtotal_minor, desconto_total_minor, acrescimo_total_minor,
                             total_minor, origem_tipo, origem_id, criado_em, atualizado_em)
         VALUES (?1, NULL, ?2, ?3, 'EM_ANDAMENTO', ?4, ?5, 0, 0, ?5, ?6, ?7, ?8, ?8)",
        params![
            &venda_id, sessao_caixa_id, usuario_id, &tipo_venda,
            total_minor, origem_tipo, origem_id, &agora
        ],
    ).map_err(|e| e.to_string())?;

    // 7. Copiar itens de consumo para venda_itens
    for (item_gourmet_id, produto_id, descricao, codigo_produto, qtd, preco, desc, _acr, total_item) in &itens_consumo {
        let item_venda_id = Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO venda_itens (id, venda_id, produto_id, descricao_produto, codigo_produto,
                                     quantidade_escala3, preco_unitario_minor, desconto_item_minor,
                                     acrescimo_item_minor, total_item_minor, cancelado, criado_em)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, ?9, 0, ?10)",
            params![
                &item_venda_id, &venda_id, produto_id, descricao, codigo_produto,
                qtd, preco, desc, total_item, &agora
            ],
        ).map_err(|e| e.to_string())?;

        // Marcar item de consumo como FECHADO
        tx.execute(
            "UPDATE gourmet_itens SET status = 'FECHADO' WHERE id = ?1",
            params![item_gourmet_id],
        ).map_err(|e| e.to_string())?;
    }

    // 8. Outbox
    let evento = match origem_tipo {
        "MESA" => "MESA_CONVERTIDA_EM_VENDA",
        _ => "COMANDA_CONVERTIDA_EM_VENDA",
    };

    inserir_outbox(&tx, evento, json!({
        "venda_id": &venda_id,
        "origem_tipo": origem_tipo,
        "origem_id": origem_id,
        "usuario_id": usuario_id,
        "sessao_caixa_id": sessao_caixa_id,
        "total_minor": total_minor,
        "total_itens": total_itens,
        "criado_em": &agora
    }))?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(FechamentoEmVendaResp {
        venda_id,
        origem_tipo: origem_tipo.to_string(),
        origem_id: origem_id.to_string(),
        total_minor,
        total_itens,
        status_venda: "EM_ANDAMENTO".to_string(),
    })
}

#[tauri::command]
pub async fn fechar_mesa_em_venda(
    dto: FecharEmVendaReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<FechamentoEmVendaResp>, String> {
    info!(componente = "aureon-pdv::commands_gourmet", origem_id = %dto.origem_id, "Chamada: fechar_mesa_em_venda");
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let resp = fechar_em_venda_interno(&mut conn, "MESA", &dto.origem_id, &dto.usuario_id, &dto.sessao_caixa_id)?;
    Ok(RespostaBase::ok("Mesa convertida em venda com sucesso. Prossiga para o pagamento.", resp))
}

#[tauri::command]
pub async fn fechar_comanda_em_venda(
    dto: FecharEmVendaReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<FechamentoEmVendaResp>, String> {
    info!(componente = "aureon-pdv::commands_gourmet", origem_id = %dto.origem_id, "Chamada: fechar_comanda_em_venda");
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let resp = fechar_em_venda_interno(&mut conn, "COMANDA", &dto.origem_id, &dto.usuario_id, &dto.sessao_caixa_id)?;
    Ok(RespostaBase::ok("Comanda convertida em venda com sucesso. Prossiga para o pagamento.", resp))
}
