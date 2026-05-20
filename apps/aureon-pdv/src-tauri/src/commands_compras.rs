use aureon_core::{dtos::*, RespostaBase};
use serde_json::json;
use tauri::{command, State};
use chrono::Utc;
use uuid::Uuid;

use crate::commands_caixa::inserir_outbox;
use crate::estado::EstadoApp;

// Helper para carregar a CompraResp completa por ID.
fn obter_compra_interna(conn: &rusqlite::Connection, compra_id: &str) -> Result<CompraResp, String> {
    let mut stmt = conn.prepare("
        SELECT id, fornecedor_id, fornecedor_nome_snapshot, numero_nota, serie,
               chave_acesso_xml_fiscal, data_emissao, status, moeda_codigo, taxa_cambio_escala6,
               subtotal_itens_minor, desconto_total_minor, frete_total_minor, outras_despesas_minor,
               impostos_total_minor, total_compra_minor, observacao, criado_em, atualizado_em,
               finalizada_em, cancelada_em, motivo_cancelamento, usuario_id
        FROM compras
        WHERE id = ?1
    ").map_err(|e| e.to_string())?;
    
    let compra_cab = stmt.query_row([compra_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, Option<String>>(6)?,
            row.get::<_, String>(7)?,
            row.get::<_, String>(8)?,
            row.get::<_, i64>(9)?,
            row.get::<_, i64>(10)?,
            row.get::<_, i64>(11)?,
            row.get::<_, i64>(12)?,
            row.get::<_, i64>(13)?,
            row.get::<_, i64>(14)?,
            row.get::<_, i64>(15)?,
            row.get::<_, Option<String>>(16)?,
            row.get::<_, String>(17)?,
            row.get::<_, String>(18)?,
            row.get::<_, Option<String>>(19)?,
            row.get::<_, Option<String>>(20)?,
            row.get::<_, Option<String>>(21)?,
            row.get::<_, String>(22)?,
        ))
    }).map_err(|e| format!("Compra não encontrada: {e}"))?;

    let mut stmt_itens = conn.prepare("
        SELECT id, compra_id, produto_id, descricao_produto_snapshot, quantidade_escala3,
               custo_unitario_minor, total_item_minor, lote, validade, serial, imei,
               cancelado, criado_em
        FROM compra_itens
        WHERE compra_id = ?1
    ").map_err(|e| e.to_string())?;

    let rows_itens = stmt_itens.query_map([compra_id], |row| {
        Ok(CompraItemResp {
            id: row.get(0)?,
            compra_id: row.get(1)?,
            produto_id: row.get(2)?,
            descricao_produto_snapshot: row.get(3)?,
            quantidade_escala3: row.get(4)?,
            custo_unitario_minor: row.get(5)?,
            total_item_minor: row.get(6)?,
            lote: row.get(7)?,
            validade: row.get(8)?,
            serial: row.get(9)?,
            imei: row.get(10)?,
            cancelado: row.get::<_, i32>(11)? != 0,
            criado_em: row.get(12)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut itens = vec![];
    for r in rows_itens {
        itens.push(r.map_err(|e| e.to_string())?);
    }

    Ok(CompraResp {
        id: compra_cab.0,
        fornecedor_id: compra_cab.1,
        fornecedor_nome_snapshot: compra_cab.2,
        numero_nota: compra_cab.3,
        serie: compra_cab.4,
        chave_acesso_xml_fiscal: compra_cab.5,
        data_emissao: compra_cab.6,
        status: compra_cab.7,
        moeda_codigo: compra_cab.8,
        taxa_cambio_escala6: compra_cab.9,
        subtotal_itens_minor: compra_cab.10,
        desconto_total_minor: compra_cab.11,
        frete_total_minor: compra_cab.12,
        outras_despesas_minor: compra_cab.13,
        impostos_total_minor: compra_cab.14,
        total_compra_minor: compra_cab.15,
        observacao: compra_cab.16,
        criado_em: compra_cab.17,
        atualizado_em: compra_cab.18,
        finalizada_em: compra_cab.19,
        cancelada_em: compra_cab.20,
        motivo_cancelamento: compra_cab.21,
        usuario_id: compra_cab.22,
        itens,
    })
}

#[command]
pub fn buscar_fornecedores_compra(
    estado: State<'_, EstadoApp>,
    busca: Option<String>,
) -> Result<RespostaBase<Vec<FornecedorResp>>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    
    let mut query = "SELECT id, nome, documento, ativo, atualizado_em FROM fornecedores_cache WHERE ativo = 1".to_string();
    let mut params: Vec<String> = vec![];
    
    if let Some(termo) = busca {
        let t = format!("%{}%", termo);
        query.push_str(" AND (nome LIKE ?1 OR documento LIKE ?1)");
        params.push(t);
    }
    
    query.push_str(" ORDER BY nome ASC");
    
    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
    let mut result = vec![];
    
    if params.is_empty() {
        let rows = stmt.query_map([], |row| {
            Ok(FornecedorResp {
                id: row.get(0)?,
                nome: row.get(1)?,
                documento: row.get(2)?,
                ativo: row.get::<_, i32>(3)? != 0,
                atualizado_em: row.get(4)?,
            })
        }).map_err(|e| e.to_string())?;
        for r in rows {
            result.push(r.map_err(|e| e.to_string())?);
        }
    } else {
        let rows = stmt.query_map([&params[0]], |row| {
            Ok(FornecedorResp {
                id: row.get(0)?,
                nome: row.get(1)?,
                documento: row.get(2)?,
                ativo: row.get::<_, i32>(3)? != 0,
                atualizado_em: row.get(4)?,
            })
        }).map_err(|e| e.to_string())?;
        for r in rows {
            result.push(r.map_err(|e| e.to_string())?);
        }
    }
    
    Ok(RespostaBase::ok("Fornecedores encontrados", result))
}

#[command]
pub fn listar_compras(
    estado: State<'_, EstadoApp>,
    status: Option<String>,
) -> Result<RespostaBase<Vec<CompraResp>>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    
    let mut query = "
        SELECT id
        FROM compras
    ".to_string();
    
    let mut params = vec![];
    if let Some(st) = status {
        query.push_str(" WHERE status = ?1");
        params.push(st);
    }
    
    query.push_str(" ORDER BY criado_em DESC");
    
    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
    let mut ids = vec![];
    
    if params.is_empty() {
        let rows = stmt.query_map([], |row| {
            row.get::<_, String>(0)
        }).map_err(|e| e.to_string())?;
        for r in rows {
            ids.push(r.map_err(|e| e.to_string())?);
        }
    } else {
        let rows = stmt.query_map([&params[0]], |row| {
            row.get::<_, String>(0)
        }).map_err(|e| e.to_string())?;
        for r in rows {
            ids.push(r.map_err(|e| e.to_string())?);
        }
    }
    
    let mut compras = vec![];
    for id in ids {
        let compra = obter_compra_interna(&conn, &id)?;
        compras.push(compra);
    }
    
    Ok(RespostaBase::ok("Compras listadas", compras))
}

#[command]
pub fn obter_compra(
    estado: State<'_, EstadoApp>,
    compra_id: String,
) -> Result<RespostaBase<CompraResp>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let compra = obter_compra_interna(&conn, &compra_id)?;
    Ok(RespostaBase::ok("Compra obtida", compra))
}

#[command]
pub fn iniciar_compra(
    estado: State<'_, EstadoApp>,
    dto: IniciarCompraReq,
) -> Result<RespostaBase<CompraResp>, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // 1. Verificar se fornecedor existe e está ativo
    let fornecedor = tx.query_row(
        "SELECT nome FROM fornecedores_cache WHERE id = ?1 AND ativo = 1",
        [&dto.fornecedor_id],
        |row| row.get::<_, String>(0)
    ).map_err(|_| "Fornecedor não encontrado ou inativo.".to_string())?;

    if dto.taxa_cambio_escala6 <= 0 {
        return Err("A taxa de câmbio precisa ser maior que zero.".to_string());
    }

    let compra_id = Uuid::new_v4().to_string();
    let agora = Utc::now().to_rfc3339();

    // 2. Inserir a compra
    tx.execute(
        "INSERT INTO compras (
            id, fornecedor_id, fornecedor_nome_snapshot, numero_nota, serie,
            chave_acesso_xml_fiscal, data_emissao, status, moeda_codigo, taxa_cambio_escala6,
            subtotal_itens_minor, desconto_total_minor, frete_total_minor, outras_despesas_minor,
            impostos_total_minor, total_compra_minor, observacao, criado_em, atualizado_em, usuario_id
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'EM_ANDAMENTO', ?8, ?9, 0, 0, 0, 0, 0, 0, ?10, ?11, ?11, ?12)",
        (
            &compra_id,
            &dto.fornecedor_id,
            &fornecedor,
            &dto.numero_nota,
            &dto.serie,
            &dto.chave_acesso_xml_fiscal,
            &dto.data_emissao,
            &dto.moeda_codigo,
            dto.taxa_cambio_escala6,
            &dto.observacao,
            &agora,
            &dto.usuario_id,
        ),
    ).map_err(|e| format!("Erro ao criar cabeçalho da compra: {e}"))?;

    // 3. Inserir outbox
    inserir_outbox(&tx, "COMPRA_CRIADA", json!({
        "compra_id": &compra_id,
        "fornecedor_id": &dto.fornecedor_id,
        "fornecedor_nome": &fornecedor,
        "numero_nota": &dto.numero_nota,
        "moeda_codigo": &dto.moeda_codigo,
        "taxa_cambio_escala6": dto.taxa_cambio_escala6,
        "usuario_id": &dto.usuario_id,
        "criado_em": &agora,
    }))?;

    tx.commit().map_err(|e| e.to_string())?;

    let compra = obter_compra_interna(&conn, &compra_id)?;
    Ok(RespostaBase::ok("Compra iniciada com sucesso", compra))
}

#[command]
pub fn adicionar_item_compra(
    estado: State<'_, EstadoApp>,
    dto: AdicionarItemCompraReq,
) -> Result<RespostaBase<CompraResp>, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // 1. Verificar se a compra existe e está EM_ANDAMENTO
    let status_compra = tx.query_row(
        "SELECT status FROM compras WHERE id = ?1",
        [&dto.compra_id],
        |row| row.get::<_, String>(0)
    ).map_err(|_| "Compra não encontrada.".to_string())?;

    if status_compra != "EM_ANDAMENTO" {
        return Err("Apenas compras em andamento podem receber novos itens.".to_string());
    }

    // 2. Verificar produto
    let produto_nome = tx.query_row(
        "SELECT nome FROM produtos_cache WHERE id = ?1",
        [&dto.produto_id],
        |row| row.get::<_, String>(0)
    ).map_err(|_| "Produto não encontrado no cache.".to_string())?;

    if dto.quantidade_escala3 <= 0 {
        return Err("A quantidade deve ser maior que zero.".to_string());
    }
    if dto.custo_unitario_minor < 0 {
        return Err("O custo unitário não pode ser negativo.".to_string());
    }

    // Calcular total do item
    let total_item_minor = (dto.custo_unitario_minor * dto.quantidade_escala3) / 1000;
    let item_id = Uuid::new_v4().to_string();
    let agora = Utc::now().to_rfc3339();

    // 3. Inserir item
    tx.execute(
        "INSERT INTO compra_itens (
            id, compra_id, produto_id, descricao_produto_snapshot, quantidade_escala3,
            custo_unitario_minor, total_item_minor, lote, validade, serial, imei, cancelado, criado_em
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0, ?12)",
        (
            &item_id,
            &dto.compra_id,
            &dto.produto_id,
            &produto_nome,
            dto.quantidade_escala3,
            dto.custo_unitario_minor,
            total_item_minor,
            &dto.lote,
            &dto.validade,
            &dto.serial,
            &dto.imei,
            &agora,
        ),
    ).map_err(|e| format!("Erro ao inserir item na compra: {e}"))?;

    // 4. Recalcular totais da compra
    let subtotal: i64 = tx.query_row(
        "SELECT COALESCE(SUM(total_item_minor), 0) FROM compra_itens WHERE compra_id = ?1 AND cancelado = 0",
        [&dto.compra_id],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;

    let (desconto, frete, outras_despesas, impostos) = tx.query_row(
        "SELECT desconto_total_minor, frete_total_minor, outras_despesas_minor, impostos_total_minor FROM compras WHERE id = ?1",
        [&dto.compra_id],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?, row.get::<_, i64>(3)?))
    ).map_err(|e| e.to_string())?;

    let total_compra = subtotal - desconto + frete + outras_despesas + impostos;

    tx.execute(
        "UPDATE compras SET subtotal_itens_minor = ?1, total_compra_minor = ?2, atualizado_em = ?3 WHERE id = ?4",
        (subtotal, total_compra, &agora, &dto.compra_id)
    ).map_err(|e| format!("Erro ao atualizar totais da compra: {e}"))?;

    // 5. Inserir outbox
    inserir_outbox(&tx, "COMPRA_ITEM_ADICIONADO", json!({
        "compra_id": &dto.compra_id,
        "item_id": &item_id,
        "produto_id": &dto.produto_id,
        "quantidade_escala3": dto.quantidade_escala3,
        "custo_unitario_minor": dto.custo_unitario_minor,
        "total_item_minor": total_item_minor
    }))?;

    tx.commit().map_err(|e| e.to_string())?;

    let compra = obter_compra_interna(&conn, &dto.compra_id)?;
    Ok(RespostaBase::ok("Item adicionado com sucesso", compra))
}

#[command]
pub fn remover_item_compra(
    estado: State<'_, EstadoApp>,
    item_id: String,
) -> Result<RespostaBase<CompraResp>, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // 1. Obter compra_id do item
    let compra_id = tx.query_row(
        "SELECT compra_id FROM compra_itens WHERE id = ?1",
        [&item_id],
        |row| row.get::<_, String>(0)
    ).map_err(|_| "Item de compra não encontrado.".to_string())?;

    // 2. Verificar se a compra está EM_ANDAMENTO
    let status_compra = tx.query_row(
        "SELECT status FROM compras WHERE id = ?1",
        [&compra_id],
        |row| row.get::<_, String>(0)
    ).map_err(|_| "Compra não encontrada.".to_string())?;

    if status_compra != "EM_ANDAMENTO" {
        return Err("Apenas compras em andamento podem ter itens removidos.".to_string());
    }

    let agora = Utc::now().to_rfc3339();

    // 3. Marcar item como cancelado
    tx.execute(
        "UPDATE compra_itens SET cancelado = 1 WHERE id = ?1",
        [&item_id]
    ).map_err(|e| format!("Erro ao cancelar item: {e}"))?;

    // 4. Recalcular totais da compra
    let subtotal: i64 = tx.query_row(
        "SELECT COALESCE(SUM(total_item_minor), 0) FROM compra_itens WHERE compra_id = ?1 AND cancelado = 0",
        [&compra_id],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;

    let (desconto, frete, outras_despesas, impostos) = tx.query_row(
        "SELECT desconto_total_minor, frete_total_minor, outras_despesas_minor, impostos_total_minor FROM compras WHERE id = ?1",
        [&compra_id],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?, row.get::<_, i64>(3)?))
    ).map_err(|e| e.to_string())?;

    let total_compra = subtotal - desconto + frete + outras_despesas + impostos;

    tx.execute(
        "UPDATE compras SET subtotal_itens_minor = ?1, total_compra_minor = ?2, atualizado_em = ?3 WHERE id = ?4",
        (subtotal, total_compra, &agora, &compra_id)
    ).map_err(|e| format!("Erro ao atualizar totais da compra: {e}"))?;

    // 5. Inserir outbox
    inserir_outbox(&tx, "COMPRA_ITEM_REMOVIDO", json!({
        "compra_id": &compra_id,
        "item_id": &item_id
    }))?;

    tx.commit().map_err(|e| e.to_string())?;

    let compra = obter_compra_interna(&conn, &compra_id)?;
    Ok(RespostaBase::ok("Item removido com sucesso", compra))
}

#[command]
pub fn cancelar_compra_em_andamento(
    estado: State<'_, EstadoApp>,
    dto: CancelarCompraEmAndamentoReq,
) -> Result<RespostaBase<CompraResp>, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // 1. Verificar se a compra está EM_ANDAMENTO
    let status_compra = tx.query_row(
        "SELECT status FROM compras WHERE id = ?1",
        [&dto.compra_id],
        |row| row.get::<_, String>(0)
    ).map_err(|_| "Compra não encontrada.".to_string())?;

    if status_compra != "EM_ANDAMENTO" {
        return Err("Apenas compras em andamento podem ser canceladas.".to_string());
    }

    if dto.motivo.trim().is_empty() {
        return Err("O motivo do cancelamento é obrigatório.".to_string());
    }

    let agora = Utc::now().to_rfc3339();

    // 2. Atualizar compra
    tx.execute(
        "UPDATE compras
         SET status = 'CANCELADA',
             cancelada_em = ?1,
             motivo_cancelamento = ?2,
             atualizado_em = ?3
         WHERE id = ?4",
        (&agora, &dto.motivo, &agora, &dto.compra_id),
    ).map_err(|e| format!("Erro ao cancelar compra: {e}"))?;

    // 3. Inserir outbox
    inserir_outbox(&tx, "COMPRA_CANCELADA", json!({
        "compra_id": &dto.compra_id,
        "motivo": &dto.motivo,
        "cancelada_em": &agora,
        "usuario_id": &dto.usuario_id
    }))?;

    tx.commit().map_err(|e| e.to_string())?;

    let compra = obter_compra_interna(&conn, &dto.compra_id)?;
    Ok(RespostaBase::ok("Compra cancelada com sucesso", compra))
}
