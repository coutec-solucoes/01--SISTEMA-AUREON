use aureon_core::{dtos::*, RespostaBase};
use serde_json::json;
use tauri::{command, State};
use chrono::Utc;
use uuid::Uuid;

use crate::commands_caixa::inserir_outbox;
use crate::commands_estoque::{obter_saldo_produto, garantir_saldo_produto, registrar_movimentacao_estoque};
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

#[command]
pub fn finalizar_compra(
    estado: State<'_, EstadoApp>,
    compra_id: String,
    usuario_id: String,
) -> Result<RespostaBase<CompraResp>, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // 1. Verificar se a compra existe e se está EM_ANDAMENTO
    let (status, moeda_codigo, taxa_cambio_escala6) = tx.query_row(
        "SELECT status, moeda_codigo, taxa_cambio_escala6 FROM compras WHERE id = ?1",
        [&compra_id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?))
    ).map_err(|_| "Compra não encontrada.".to_string())?;

    if status != "EM_ANDAMENTO" {
        return Err("Apenas compras em andamento podem ser finalizadas.".to_string());
    }

    // 2. Verificar se a compra possui itens ativos
    let total_itens_ativos: i64 = tx.query_row(
        "SELECT COUNT(*) FROM compra_itens WHERE compra_id = ?1 AND cancelado = 0",
        [&compra_id],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;

    if total_itens_ativos == 0 {
        return Err("Não é possível finalizar uma compra sem itens ativos.".to_string());
    }

    // 3. Verificar se já existe a entrada de estoque para evitar duplicações (Idempotência)
    let entrada_existe: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM estoque_movimentacoes WHERE tipo_movimentacao = 'ENTRADA_COMPRA' AND origem_tipo = 'COMPRA' AND origem_id = ?1)",
        [&compra_id],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;

    if !entrada_existe {
        // Buscar itens ativos
        let mut stmt_itens = tx.prepare("
            SELECT produto_id, quantidade_escala3, custo_unitario_minor
            FROM compra_itens
            WHERE compra_id = ?1 AND cancelado = 0
        ").map_err(|e| e.to_string())?;
        
        let rows = stmt_itens.query_map([&compra_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        }).map_err(|e| e.to_string())?;
        
        let mut itens = vec![];
        for r in rows {
            itens.push(r.map_err(|e| e.to_string())?);
        }
        drop(stmt_itens);

        let agora = Utc::now().to_rfc3339();

        for (produto_id, quantidade_escala3, custo_unitario_minor) in itens {
            // Obter se controla estoque
            let controla: i32 = tx.query_row(
                "SELECT controla_estoque FROM produtos_cache WHERE id = ?1",
                [&produto_id],
                |row| row.get(0)
            ).unwrap_or(1);

            if controla == 1 {
                let saldo_atual = obter_saldo_produto(&tx, &produto_id).map_err(|e| e.to_string())?;
                let novo_saldo = saldo_atual + quantidade_escala3;
                garantir_saldo_produto(&tx, &produto_id, novo_saldo).map_err(|e| e.to_string())?;
                registrar_movimentacao_estoque(
                    &tx,
                    &produto_id,
                    quantidade_escala3,
                    novo_saldo,
                    "ENTRADA_COMPRA",
                    "COMPRA",
                    &compra_id,
                    Some("Entrada de compra/manual"),
                    &usuario_id,
                ).map_err(|e| e.to_string())?;
            }

            // Atualização de último custo
            let custo_convertido_minor = if moeda_codigo == "BRL" || taxa_cambio_escala6 == 1_000_000 {
                custo_unitario_minor
            } else {
                (custo_unitario_minor * taxa_cambio_escala6) / 1_000_000
            };

            tx.execute(
                "UPDATE produtos_cache
                 SET ultimo_custo_minor = ?1,
                     ultimo_custo_moeda_codigo = ?2,
                     ultimo_custo_taxa_cambio_escala6 = ?3,
                     ultimo_custo_atualizado_em = ?4
                 WHERE id = ?5",
                (custo_convertido_minor, &moeda_codigo, taxa_cambio_escala6, &agora, &produto_id),
            ).map_err(|e| format!("Erro ao atualizar último custo do produto: {e}"))?;
        }
    }

    let agora = Utc::now().to_rfc3339();

    // FASE 13 - FINANCEIRO: Gerar contas_pagar automaticamente
    gerar_conta_pagar_compra(&tx, &compra_id, &usuario_id)?;

    // 4. Mudar status da compra
    tx.execute(
        "UPDATE compras
         SET status = 'FINALIZADA',
             finalizada_em = ?1,
             atualizado_em = ?2
         WHERE id = ?3",
        (&agora, &agora, &compra_id),
    ).map_err(|e| format!("Erro ao atualizar status da compra: {e}"))?;

    // 5. Inserir outbox COMPRA_FINALIZADA
    inserir_outbox(&tx, "COMPRA_FINALIZADA", json!({
        "compra_id": &compra_id,
        "finalizada_em": &agora,
        "usuario_id": &usuario_id
    }))?;

    tx.commit().map_err(|e| e.to_string())?;

    let compra = obter_compra_interna(&conn, &compra_id)?;
    Ok(RespostaBase::ok("Compra finalizada com sucesso", compra))
}

#[command]
pub fn cancelar_compra_finalizada(
    estado: State<'_, EstadoApp>,
    dto: CancelarCompraFinalizadaReq,
) -> Result<RespostaBase<CompraResp>, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // 1. Verificar se a compra existe e se está FINALIZADA
    let status = tx.query_row(
        "SELECT status FROM compras WHERE id = ?1",
        [&dto.compra_id],
        |row| row.get::<_, String>(0)
    ).map_err(|_| "Compra não encontrada.".to_string())?;

    if status != "FINALIZADA" {
        return Err("Apenas compras finalizadas podem ser canceladas.".to_string());
    }

    if dto.motivo.trim().is_empty() {
        return Err("O motivo do cancelamento é obrigatório.".to_string());
    }

    // FASE 13 - FINANCEIRO: Cancelar conta a pagar correspondente
    cancelar_conta_pagar_compra(&tx, &dto.compra_id, &dto.usuario_id)?;

    // 2. Verificar se houve ENTRADA_COMPRA e se já houve ESTORNO_ENTRADA_COMPRA (Idempotência)
    let entrada_existe: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM estoque_movimentacoes WHERE tipo_movimentacao = 'ENTRADA_COMPRA' AND origem_tipo = 'COMPRA' AND origem_id = ?1)",
        [&dto.compra_id],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;

    let estorno_existe: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM estoque_movimentacoes WHERE tipo_movimentacao = 'ESTORNO_ENTRADA_COMPRA' AND origem_tipo = 'COMPRA' AND origem_id = ?1)",
        [&dto.compra_id],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;

    if entrada_existe && !estorno_existe {
        // Buscar itens ativos
        let mut stmt_itens = tx.prepare("
            SELECT produto_id, quantidade_escala3
            FROM compra_itens
            WHERE compra_id = ?1 AND cancelado = 0
        ").map_err(|e| e.to_string())?;
        
        let rows = stmt_itens.query_map([&dto.compra_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
            ))
        }).map_err(|e| e.to_string())?;
        
        let mut itens = vec![];
        for r in rows {
            itens.push(r.map_err(|e| e.to_string())?);
        }
        drop(stmt_itens);

        let motivo_kardex = format!("Estorno entrada compra: {}", dto.motivo);

        for (produto_id, quantidade_escala3) in itens {
            // Obter se controla estoque
            let controla: i32 = tx.query_row(
                "SELECT controla_estoque FROM produtos_cache WHERE id = ?1",
                [&produto_id],
                |row| row.get(0)
            ).unwrap_or(1);

            if controla == 1 {
                let saldo_atual = obter_saldo_produto(&tx, &produto_id).map_err(|e| e.to_string())?;
                let novo_saldo = saldo_atual - quantidade_escala3;
                garantir_saldo_produto(&tx, &produto_id, novo_saldo).map_err(|e| e.to_string())?;
                registrar_movimentacao_estoque(
                    &tx,
                    &produto_id,
                    -quantidade_escala3,
                    novo_saldo,
                    "ESTORNO_ENTRADA_COMPRA",
                    "COMPRA",
                    &dto.compra_id,
                    Some(&motivo_kardex),
                    &dto.usuario_id,
                ).map_err(|e| e.to_string())?;
            }
        }
    }

    let agora = Utc::now().to_rfc3339();

    // 3. Mudar status da compra
    tx.execute(
        "UPDATE compras
         SET status = 'CANCELADA',
             cancelada_em = ?1,
             motivo_cancelamento = ?2,
             atualizado_em = ?3
         WHERE id = ?4",
        (&agora, &dto.motivo, &agora, &dto.compra_id),
    ).map_err(|e| format!("Erro ao atualizar status da compra: {e}"))?;

    // 4. Inserir outbox COMPRA_CANCELADA
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

fn gerar_conta_pagar_compra(
    tx: &rusqlite::Transaction<'_>,
    compra_id: &str,
    usuario_id: &str,
) -> Result<(), String> {
    // 1. Idempotência: verificar se já existe contas_pagar com compra_id
    let existe: bool = tx.query_row(
        "SELECT COUNT(*) > 0 FROM contas_pagar WHERE compra_id = ?1",
        rusqlite::params![compra_id],
        |row| row.get(0)
    ).unwrap_or(false);

    if existe {
        return Ok(());
    }

    // 2. Buscar dados da compra
    let (
        fornecedor_id,
        fornecedor_nome_snapshot,
        data_emissao_opt,
        moeda_codigo,
        taxa_cambio_escala6,
        total_compra_minor,
        criado_em
    ) = tx.query_row(
        "SELECT fornecedor_id, fornecedor_nome_snapshot, data_emissao, moeda_codigo,
                taxa_cambio_escala6, total_compra_minor, criado_em 
         FROM compras WHERE id = ?1",
        [compra_id],
        |row| Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, String>(6)?,
        ))
    ).map_err(|e| format!("Erro ao obter compra para gerar contas a pagar: {e}"))?;

    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let data_emissao = data_emissao_opt.unwrap_or_else(|| criado_em.clone());
    
    let vencimento_dt = Utc::now() + chrono::Duration::days(30);
    let data_vencimento = vencimento_dt.format("%Y-%m-%d %H:%M:%S").to_string();

    let valor_original_principal_minor = match total_compra_minor.checked_mul(taxa_cambio_escala6) {
        Some(val) => val / 1_000_000,
        None => return Err("Overflow no cálculo de conversão do valor principal da compra".to_string()),
    };

    let saldo_pendente_minor = total_compra_minor; 

    let cp_id = uuid::Uuid::new_v4().to_string();

    tx.execute(
        "INSERT INTO contas_pagar (
            id, fornecedor_id, fornecedor_nome_snapshot, compra_id, descricao, moeda_codigo,
            valor_original_minor, taxa_cambio_escala6, valor_original_principal_minor,
            data_emissao, data_vencimento, status, saldo_pendente_minor, criado_em,
            atualizado_em, usuario_id, observacao
        ) VALUES (?1, ?2, ?3, ?4, 'Compra manual / Nota de compra', ?5, ?6, ?7, ?8, ?9, ?10, 'PENDENTE', ?11, ?12, ?12, ?13, NULL)",
        rusqlite::params![
            cp_id,
            fornecedor_id,
            fornecedor_nome_snapshot,
            compra_id,
            moeda_codigo,
            total_compra_minor,
            taxa_cambio_escala6,
            valor_original_principal_minor,
            data_emissao,
            data_vencimento,
            saldo_pendente_minor,
            agora,
            usuario_id
        ]
    ).map_err(|e| format!("Erro ao inserir contas_pagar da compra: {e}"))?;

    // outbox CONTA_PAGAR_CRIADA
    inserir_outbox(
        tx,
        "CONTA_PAGAR_CRIADA",
        json!({
            "id": cp_id,
            "fornecedor_id": fornecedor_id,
            "fornecedor_nome_snapshot": fornecedor_nome_snapshot,
            "compra_id": compra_id,
            "descricao": "Compra manual / Nota de compra",
            "moeda_codigo": moeda_codigo,
            "valor_original_minor": total_compra_minor,
            "taxa_cambio_escala6": taxa_cambio_escala6,
            "valor_original_principal_minor": valor_original_principal_minor,
            "data_emissao": data_emissao,
            "data_vencimento": data_vencimento,
            "status": "PENDENTE",
            "saldo_pendente_minor": saldo_pendente_minor,
            "criado_em": agora,
            "usuario_id": usuario_id
        })
    )?;

    Ok(())
}

fn cancelar_conta_pagar_compra(
    tx: &rusqlite::Transaction<'_>,
    compra_id: &str,
    usuario_id: &str,
) -> Result<(), String> {
    // 1. Obter contas_pagar vinculada
    let conta_opt: Option<(String, String)> = tx.query_row(
        "SELECT id, status FROM contas_pagar WHERE compra_id = ?1",
        [compra_id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    ).ok();

    if let Some((cp_id, status)) = conta_opt {
        if status == "PAGO_PARCIAL" || status == "PAGO" {
            return Err("Não é possível estornar esta compra porque o título financeiro correspondente já possui pagamentos baixados.".to_string());
        }

        if status == "PENDENTE" {
            let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

            tx.execute(
                "UPDATE contas_pagar 
                 SET status = 'CANCELADO', saldo_pendente_minor = 0, atualizado_em = ?1, observacao = 'Cancelado por estorno de compra'
                 WHERE id = ?2",
                rusqlite::params![agora, cp_id],
            ).map_err(|e| format!("Erro ao cancelar conta a pagar vinculada: {e}"))?;

            // outbox CONTA_PAGAR_CANCELADA
            inserir_outbox(
                tx,
                "CONTA_PAGAR_CANCELADA",
                json!({
                    "conta_pagar_id": cp_id,
                    "cancelado_em": agora,
                    "motivo": "Cancelado por estorno de compra",
                    "usuario_id": usuario_id
                })
            )?;
        }
    }

    Ok(())
}

