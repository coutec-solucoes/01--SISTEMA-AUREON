use aureon_core::{
    dtos::*,
    RespostaBase,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::json;
use tauri::State;
use uuid::Uuid;
use chrono::Utc;
use tracing::info;

use crate::estado::EstadoApp;
use crate::commands_caixa::inserir_outbox;

// --- Auxiliares ---

fn calcular_total_pedido(conn: &Connection, delivery_id: &str) -> Result<i64, String> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(SUM(total_item_minor), 0) FROM delivery_itens WHERE delivery_id = ? AND cancelado = 0"
    ).map_err(|e| e.to_string())?;
    
    let total: i64 = stmt.query_row(params![delivery_id], |row| row.get(0))
        .map_err(|e| e.to_string())?;
        
    Ok(total)
}

fn atualizar_total_pedido(conn: &Connection, delivery_id: &str) -> Result<(), String> {
    let total_itens = calcular_total_pedido(conn, delivery_id)?;
    conn.execute(
        "UPDATE delivery_operacional SET total_consumo_minor = ? WHERE id = ?",
        params![total_itens, delivery_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

fn checar_caixa_aberto(conn: &Connection, sessao_id: &str) -> Result<(), String> {
    let ativo: Option<i64> = conn.query_row(
        "SELECT ativo FROM caixa_sessoes WHERE id = ?",
        params![sessao_id],
        |row| row.get(0)
    ).optional().map_err(|e| e.to_string())?;

    if ativo != Some(1) {
        return Err("Caixa precisa estar aberto para esta operação".into());
    }
    Ok(())
}

fn checar_pedido_nao_fechado(conn: &Connection, delivery_id: &str) -> Result<String, String> {
    let status: Option<String> = conn.query_row(
        "SELECT status FROM delivery_operacional WHERE id = ?",
        params![delivery_id],
        |row| row.get(0)
    ).optional().map_err(|e| e.to_string())?;

    match status {
        Some(s) => {
            if s == "FECHADO" || s == "CANCELADO" {
                return Err(format!("Pedido em status {}, não pode ser alterado.", s));
            }
            Ok(s)
        },
        None => Err("Pedido Delivery não encontrado".into()),
    }
}

// --- Listagem ---

#[tauri::command]
pub async fn listar_pedidos_delivery(
    estado: State<'_, EstadoApp>
) -> Result<RespostaBase<Vec<DeliveryOperacionalResp>>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("
        SELECT id, numero_pedido, cliente_id, nome_cliente_informal, telefone, endereco_completo,
               tipo_pedido, status, origem, entregador_id, taxa_entrega_minor, total_consumo_minor,
               sessao_caixa_id, observacao, previsao_entrega, aberto_em, fechado_em
        FROM delivery_operacional
        ORDER BY aberto_em DESC
    ").map_err(|e| e.to_string())?;

    let iter = stmt.query_map([], |row| {
        Ok(DeliveryOperacionalResp {
            id: row.get(0)?,
            numero_pedido: row.get(1)?,
            cliente_id: row.get(2)?,
            nome_cliente_informal: row.get(3)?,
            telefone: row.get(4)?,
            endereco_completo: row.get(5)?,
            tipo_pedido: row.get(6)?,
            status: row.get(7)?,
            origem: row.get(8)?,
            entregador_id: row.get(9)?,
            taxa_entrega_minor: row.get(10)?,
            total_consumo_minor: row.get(11)?,
            sessao_caixa_id: row.get(12)?,
            observacao: row.get(13)?,
            previsao_entrega: row.get(14)?,
            aberto_em: row.get(15)?,
            fechado_em: row.get(16)?,
            itens: vec![],
        })
    }).map_err(|e| e.to_string())?;

    let mut lista = Vec::new();
    for i in iter {
        if let Ok(ped) = i {
            // Pode otimizar depois, não populando itens na listagem geral para economizar IO
            lista.push(ped);
        }
    }

    Ok(RespostaBase::ok("OK", lista))
}

#[tauri::command]
pub async fn obter_pedido_delivery(
    delivery_id: String,
    estado: State<'_, EstadoApp>
) -> Result<RespostaBase<DeliveryOperacionalResp>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let ped = internal_obter_pedido(&conn, &delivery_id)?;
    Ok(RespostaBase::ok("OK", ped))
}

fn internal_obter_pedido(conn: &Connection, delivery_id: &str) -> Result<DeliveryOperacionalResp, String> {
    let ped_opt = conn.query_row("
        SELECT id, numero_pedido, cliente_id, nome_cliente_informal, telefone, endereco_completo,
               tipo_pedido, status, origem, entregador_id, taxa_entrega_minor, total_consumo_minor,
               sessao_caixa_id, observacao, previsao_entrega, aberto_em, fechado_em
        FROM delivery_operacional
        WHERE id = ?
    ", params![delivery_id], |row| {
        Ok(DeliveryOperacionalResp {
            id: row.get(0)?,
            numero_pedido: row.get(1)?,
            cliente_id: row.get(2)?,
            nome_cliente_informal: row.get(3)?,
            telefone: row.get(4)?,
            endereco_completo: row.get(5)?,
            tipo_pedido: row.get(6)?,
            status: row.get(7)?,
            origem: row.get(8)?,
            entregador_id: row.get(9)?,
            taxa_entrega_minor: row.get(10)?,
            total_consumo_minor: row.get(11)?,
            sessao_caixa_id: row.get(12)?,
            observacao: row.get(13)?,
            previsao_entrega: row.get(14)?,
            aberto_em: row.get(15)?,
            fechado_em: row.get(16)?,
            itens: vec![],
        })
    }).optional().map_err(|e| e.to_string())?;

    let mut ped = match ped_opt {
        Some(p) => p,
        None => return Err("Pedido não encontrado".into()),
    };

    let mut stmt = conn.prepare("
        SELECT id, delivery_id, produto_id, descricao_produto, codigo_produto,
               quantidade_escala3, preco_unitario_minor, desconto_item_minor,
               acrescimo_item_minor, total_item_minor, observacao_producao,
               local_producao_id, status, enviado_producao, cancelado
        FROM delivery_itens
        WHERE delivery_id = ?
        ORDER BY criado_em ASC
    ").map_err(|e| e.to_string())?;

    let iter = stmt.query_map(params![delivery_id], |row| {
        Ok(DeliveryItemResp {
            id: row.get(0)?,
            delivery_id: row.get(1)?,
            produto_id: row.get(2)?,
            descricao_produto: row.get(3)?,
            codigo_produto: row.get(4)?,
            quantidade_escala3: row.get(5)?,
            preco_unitario_minor: row.get(6)?,
            desconto_item_minor: row.get(7)?,
            acrescimo_item_minor: row.get(8)?,
            total_item_minor: row.get(9)?,
            observacao_producao: row.get(10)?,
            local_producao_id: row.get(11)?,
            status: row.get(12)?,
            enviado_producao: row.get::<_, i64>(13)? == 1,
            cancelado: row.get::<_, i64>(14)? == 1,
        })
    }).map_err(|e| e.to_string())?;

    for i in iter {
        if let Ok(it) = i { ped.itens.push(it); }
    }

    Ok(ped)
}

#[tauri::command]
pub async fn listar_entregadores_delivery(
    estado: State<'_, EstadoApp>
) -> Result<RespostaBase<Vec<EntregadorResp>>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, nome, documento, ativo FROM entregadores_cache ORDER BY nome").map_err(|e| e.to_string())?;
    let iter = stmt.query_map([], |row| {
        Ok(EntregadorResp {
            id: row.get(0)?,
            nome: row.get(1)?,
            documento: row.get(2)?,
            ativo: row.get::<_, i64>(3)? == 1,
        })
    }).map_err(|e| e.to_string())?;

    let mut lista = Vec::new();
    for i in iter {
        if let Ok(ent) = i { lista.push(ent); }
    }
    Ok(RespostaBase::ok("OK", lista))
}

// --- Criação / Aceite / Recusa ---

#[tauri::command]
pub async fn criar_pedido_local(
    dto: CriarPedidoLocalReq,
    estado: State<'_, EstadoApp>
) -> Result<RespostaBase<DeliveryOperacionalResp>, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    
    if dto.taxa_entrega_minor < 0 {
        return Err("Taxa de entrega não pode ser negativa".into());
    }
    if dto.tipo_pedido != "ENTREGA" && dto.tipo_pedido != "RETIRADA" {
        return Err("Tipo de pedido inválido".into());
    }
    if dto.tipo_pedido == "ENTREGA" {
        if let Some(ref end) = dto.endereco_completo {
            if end.trim().is_empty() { return Err("Endereço obrigatório para ENTREGA".into()); }
        } else {
            return Err("Endereço obrigatório para ENTREGA".into());
        }
    }

    checar_caixa_aberto(&conn, &dto.sessao_caixa_id).map_err(|e| e.to_string())?;

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    let max_num: i64 = tx.query_row("SELECT COALESCE(MAX(numero_pedido), 0) FROM delivery_operacional", [], |row| row.get(0)).unwrap_or(0);
    let novo_num = max_num + 1;
    let id = Uuid::new_v4().to_string();
    let agora = Utc::now().to_rfc3339();

    tx.execute("
        INSERT INTO delivery_operacional (
            id, numero_pedido, cliente_id, nome_cliente_informal, telefone, endereco_completo,
            tipo_pedido, status, origem, entregador_id, taxa_entrega_minor, total_consumo_minor,
            sessao_caixa_id, observacao, previsao_entrega, aberto_em
        ) VALUES (?, ?, NULL, ?, ?, ?, ?, 'ACEITO', 'LOCAL', NULL, ?, 0, ?, ?, NULL, ?)
    ", params![
        id, novo_num, dto.nome_cliente_informal, dto.telefone, dto.endereco_completo,
        dto.tipo_pedido, dto.taxa_entrega_minor, dto.sessao_caixa_id, dto.observacao, agora
    ]).map_err(|e| e.to_string())?;

    inserir_outbox(&tx, "DELIVERY_CRIADO", json!({
        "numero_pedido": novo_num, "origem": "LOCAL", "status": "ACEITO"
    })).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    let novo_pedido = internal_obter_pedido(&conn, &id)?;
    Ok(RespostaBase::ok("OK", novo_pedido))
}

#[tauri::command]
pub async fn aceitar_pedido_online(
    delivery_id: String,
    sessao_caixa_id: String,
    estado: State<'_, EstadoApp>
) -> Result<RespostaBase<String>, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    checar_caixa_aberto(&conn, &sessao_caixa_id).map_err(|e| e.to_string())?;

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    let status: Option<String> = tx.query_row("SELECT status FROM delivery_operacional WHERE id = ? AND origem = 'ONLINE'", params![delivery_id], |row| row.get(0)).optional().map_err(|e| e.to_string())?;
    
    match status {
        Some(s) if s == "NOVO" => {
            tx.execute("UPDATE delivery_operacional SET status = 'ACEITO', sessao_caixa_id = ? WHERE id = ?", params![sessao_caixa_id, delivery_id]).map_err(|e| e.to_string())?;
            inserir_outbox(&tx, "DELIVERY_ACEITO", json!({"delivery_id": delivery_id, "status": "ACEITO"})).map_err(|e| e.to_string())?;
            tx.commit().map_err(|e| e.to_string())?;
            Ok(RespostaBase::ok("Pedido aceito com sucesso", "OK".to_string()))
        },
        Some(_) => Err("Pedido não está no status NOVO".into()),
        None => Err("Pedido ONLINE não encontrado".into()),
    }
}

#[tauri::command]
pub async fn recusar_pedido_online(
    dto: RecusarPedidoOnlineReq,
    estado: State<'_, EstadoApp>
) -> Result<RespostaBase<String>, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    checar_caixa_aberto(&conn, &dto.sessao_caixa_id).map_err(|e| e.to_string())?;

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    let status: Option<String> = tx.query_row("SELECT status FROM delivery_operacional WHERE id = ? AND origem = 'ONLINE'", params![dto.delivery_id], |row| row.get(0)).optional().map_err(|e| e.to_string())?;
    
    match status {
        Some(s) if s == "NOVO" => {
            tx.execute("UPDATE delivery_operacional SET status = 'CANCELADO', observacao = ? WHERE id = ?", params![format!("RECUSADO: {}", dto.motivo), dto.delivery_id]).map_err(|e| e.to_string())?;
            inserir_outbox(&tx, "DELIVERY_RECUSADO", json!({"delivery_id": dto.delivery_id, "motivo": dto.motivo})).map_err(|e| e.to_string())?;
            tx.commit().map_err(|e| e.to_string())?;
            Ok(RespostaBase::ok("Pedido recusado com sucesso", "OK".to_string()))
        },
        Some(_) => Err("Pedido não está no status NOVO".into()),
        None => Err("Pedido ONLINE não encontrado".into()),
    }
}

// --- Alteração de Status ---

#[tauri::command]
pub async fn atualizar_status_delivery(
    dto: AtualizarStatusDeliveryReq,
    estado: State<'_, EstadoApp>
) -> Result<RespostaBase<String>, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    checar_caixa_aberto(&conn, &dto.sessao_caixa_id).map_err(|e| e.to_string())?;

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    let ped: Option<(String, String, Option<String>)> = tx.query_row("SELECT status, tipo_pedido, entregador_id FROM delivery_operacional WHERE id = ?", params![dto.delivery_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))).optional().map_err(|e| e.to_string())?;
    
    if let Some((status_atual, tipo_pedido, entregador_id)) = ped {
        if status_atual == "FECHADO" || status_atual == "CANCELADO" {
            return Err("Não pode alterar status de pedido finalizado".into());
        }

        // Validação de regras de transição simples
        let transicao_valida = match (status_atual.as_str(), dto.novo_status.as_str()) {
            ("NOVO", "ACEITO") | ("NOVO", "CANCELADO") => true,
            ("ACEITO", "PREPARANDO") | ("ACEITO", "CANCELADO") => true,
            ("PREPARANDO", "PRONTO") | ("PREPARANDO", "CANCELADO") => true,
            ("PRONTO", "DESPACHADO") | ("PRONTO", "CANCELADO") => true,
            // FECHADO será manipulado apenas via fechar_delivery_em_venda, não por atualizar_status_delivery diretamente
            ("DESPACHADO", "CANCELADO") => true,
            _ => false
        };

        if !transicao_valida {
            return Err(format!("Transição de {} para {} inválida", status_atual, dto.novo_status));
        }

        if dto.novo_status == "DESPACHADO" && tipo_pedido == "ENTREGA" && entregador_id.is_none() {
            return Err("Exige entregador antes de despachar a ENTREGA".into());
        }

        tx.execute("UPDATE delivery_operacional SET status = ? WHERE id = ?", params![dto.novo_status, dto.delivery_id]).map_err(|e| e.to_string())?;
        inserir_outbox(&tx, "DELIVERY_STATUS_ALTERADO", json!({"delivery_id": dto.delivery_id, "status": dto.novo_status})).map_err(|e| e.to_string())?;
        
        tx.commit().map_err(|e| e.to_string())?;
        Ok(RespostaBase::ok("OK", format!("Status alterado para {}", dto.novo_status)))
    } else {
        Err("Pedido não encontrado".into())
    }
}

#[tauri::command]
pub async fn definir_entregador(
    dto: DefinirEntregadorReq,
    estado: State<'_, EstadoApp>
) -> Result<RespostaBase<String>, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    checar_caixa_aberto(&conn, &dto.sessao_caixa_id).map_err(|e| e.to_string())?;

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let status_ped = checar_pedido_nao_fechado(&tx, &dto.delivery_id).map_err(|e| e.to_string())?;

    if status_ped == "DESPACHADO" {
        return Err("Pedido já foi despachado".into()); // Dependendo da regra, pode trocar motoboy no caminho, mas por hora travado
    }

    let ativo: Option<i64> = tx.query_row("SELECT ativo FROM entregadores_cache WHERE id = ?", params![dto.entregador_id], |row| row.get(0)).optional().map_err(|e| e.to_string())?;
    if ativo != Some(1) {
        return Err("Entregador inválido ou inativo".into());
    }

    tx.execute("UPDATE delivery_operacional SET entregador_id = ? WHERE id = ?", params![dto.entregador_id, dto.delivery_id]).map_err(|e| e.to_string())?;
    inserir_outbox(&tx, "DELIVERY_ENTREGADOR_DEFINIDO", json!({"delivery_id": dto.delivery_id, "entregador_id": dto.entregador_id})).map_err(|e| e.to_string())?;
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(RespostaBase::ok("OK", "Entregador definido".to_string()))
}

// --- Manipulação de Itens ---

#[tauri::command]
pub async fn adicionar_item_delivery(
    dto: AdicionarItemDeliveryReq,
    estado: State<'_, EstadoApp>
) -> Result<RespostaBase<String>, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    checar_caixa_aberto(&conn, &dto.sessao_caixa_id).map_err(|e| e.to_string())?;

    if dto.quantidade_escala3 <= 0 || dto.acrescimo_item_minor < 0 || dto.desconto_item_minor < 0 {
        return Err("Valores do item inválidos".into());
    }

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    checar_pedido_nao_fechado(&tx, &dto.delivery_id).map_err(|e| e.to_string())?;

    let (desc, cod, preco, local_prod): (String, Option<String>, i64, Option<String>) = tx.query_row(
        "SELECT nome, codigo, preco_base_minor, local_producao_id FROM produtos_cache WHERE id = ?",
        params![dto.produto_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    ).map_err(|_| "Produto não encontrado no cache".to_string())?;

    let subtotal = (preco * dto.quantidade_escala3) / 1000;
    let total_item = subtotal - dto.desconto_item_minor + dto.acrescimo_item_minor;

    let item_id = Uuid::new_v4().to_string();
    let agora = Utc::now().to_rfc3339();

    tx.execute("
        INSERT INTO delivery_itens (
            id, delivery_id, produto_id, descricao_produto, codigo_produto,
            quantidade_escala3, preco_unitario_minor, desconto_item_minor, acrescimo_item_minor,
            total_item_minor, observacao_producao, local_producao_id,
            status, enviado_producao, cancelado, criado_em
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'PENDENTE', 0, 0, ?)
    ", params![
        item_id, dto.delivery_id, dto.produto_id, desc, cod,
        dto.quantidade_escala3, preco, dto.desconto_item_minor, dto.acrescimo_item_minor,
        total_item, dto.observacao_producao, local_prod, agora
    ]).map_err(|e| e.to_string())?;

    atualizar_total_pedido(&tx, &dto.delivery_id).map_err(|e| e.to_string())?;

    inserir_outbox(&tx, "DELIVERY_ITEM_ADICIONADO", json!({
        "item_id": item_id, "delivery_id": dto.delivery_id, "total_item_minor": total_item
    })).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(RespostaBase::ok("OK", item_id))
}

#[tauri::command]
pub async fn cancelar_item_delivery(
    dto: CancelarItemDeliveryReq,
    estado: State<'_, EstadoApp>
) -> Result<RespostaBase<String>, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    checar_caixa_aberto(&conn, &dto.sessao_caixa_id).map_err(|e| e.to_string())?;

    if dto.motivo_cancelamento.trim().is_empty() {
        return Err("Motivo de cancelamento obrigatório".into());
    }

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let (delivery_id, cancelado): (String, i64) = tx.query_row(
        "SELECT delivery_id, cancelado FROM delivery_itens WHERE id = ?",
        params![dto.item_id],
        |row| Ok((row.get(0)?, row.get(1)?))
    ).map_err(|_| "Item não encontrado".to_string())?;

    if cancelado == 1 {
        return Err("Item já está cancelado".into());
    }

    checar_pedido_nao_fechado(&tx, &delivery_id).map_err(|e| e.to_string())?;

    let agora = Utc::now().to_rfc3339();
    tx.execute("
        UPDATE delivery_itens 
        SET cancelado = 1, cancelado_em = ?, motivo_cancelamento = ?, 
            usuario_cancelamento_id = ?, supervisor_id = ?, status = 'CANCELADO'
        WHERE id = ?
    ", params![
        agora, dto.motivo_cancelamento, dto.usuario_cancelamento_id, 
        dto.supervisor_id, dto.item_id
    ]).map_err(|e| e.to_string())?;

    atualizar_total_pedido(&tx, &delivery_id).map_err(|e| e.to_string())?;

    inserir_outbox(&tx, "DELIVERY_ITEM_CANCELADO", json!({
        "item_id": dto.item_id, "delivery_id": delivery_id, "motivo": dto.motivo_cancelamento
    })).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(RespostaBase::ok("OK", "Item cancelado com sucesso".to_string()))
}
