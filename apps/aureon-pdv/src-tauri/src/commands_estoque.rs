use aureon_core::{dtos::*, AureonError, RespostaBase};
use serde_json::json;
use tauri::{command, State};
use chrono::Utc;
use uuid::Uuid;
use rusqlite::Transaction;

use crate::commands_caixa::inserir_outbox;
use crate::estado::EstadoApp;

// ========================================================================
// FUNÇÕES INTERNAS DE ESTOQUE
// ========================================================================

/// Obtém o saldo atual da tabela de cache. Retorna 0 se não existir registro ainda.
pub fn obter_saldo_produto(tx: &Transaction, produto_id: &str) -> Result<i64, AureonError> {
    let mut stmt = tx.prepare("SELECT quantidade_escala3 FROM produtos_estoque_cache WHERE produto_id = ?").map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;
    let mut rows = stmt.query([produto_id]).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;
    if let Some(row) = rows.next().map_err(|e| AureonError::ConexaoSqlite(e.to_string()))? {
        Ok(row.get(0).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?)
    } else {
        Ok(0)
    }
}

/// Garante que exista uma linha na tabela de cache para o produto.
pub fn garantir_saldo_produto(tx: &Transaction, produto_id: &str, quantidade_escala3: i64) -> Result<(), AureonError> {
    let agora = Utc::now().to_rfc3339();
    tx.execute(
        "INSERT INTO produtos_estoque_cache (produto_id, quantidade_escala3, atualizado_em)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(produto_id) DO UPDATE SET
            quantidade_escala3 = excluded.quantidade_escala3,
            atualizado_em = excluded.atualizado_em",
        (produto_id, quantidade_escala3, agora),
    ).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;
    Ok(())
}

/// Registra a movimentação no Kardex. Nunca atualiza nem deleta.
pub fn registrar_movimentacao_estoque(
    tx: &Transaction,
    produto_id: &str,
    quantidade_escala3: i64,
    saldo_apos_escala3: i64,
    tipo_movimentacao: &str,
    origem_tipo: &str,
    origem_id: &str,
    motivo: Option<&str>,
    usuario_id: &str,
) -> Result<String, AureonError> {
    let mov_id = Uuid::new_v4().to_string();
    let criado_em = Utc::now().to_rfc3339();

    tx.execute(
        "INSERT INTO estoque_movimentacoes (
            id, produto_id, quantidade_escala3, saldo_apos_escala3,
            tipo_movimentacao, origem_tipo, origem_id, motivo, usuario_id, criado_em
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        (
            &mov_id,
            produto_id,
            quantidade_escala3,
            saldo_apos_escala3,
            tipo_movimentacao,
            origem_tipo,
            origem_id,
            motivo,
            usuario_id,
            &criado_em,
        ),
    ).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

    // Gera o evento no outbox
    inserir_outbox(tx, "ESTOQUE_MOVIMENTACAO_GERADA", json!({
        "id": mov_id,
        "produto_id": produto_id,
        "quantidade_escala3": quantidade_escala3,
        "saldo_apos_escala3": saldo_apos_escala3,
        "tipo_movimentacao": tipo_movimentacao,
        "origem_tipo": origem_tipo,
        "origem_id": origem_id,
        "usuario_id": usuario_id,
        "criado_em": criado_em
    })).map_err(|e| AureonError::Interno(e.to_string()))?;

    Ok(mov_id)
}

/// Processa a baixa de estoque ao finalizar uma venda.
pub fn processar_baixa_venda(
    tx: &Transaction,
    venda_id: &str,
    usuario_id: &str,
) -> Result<(), AureonError> {
    // 1. Idempotência: verificar se já existe baixa
    {
        let mut stmt_check = tx.prepare(
            "SELECT id FROM estoque_movimentacoes WHERE origem_id = ? AND tipo_movimentacao = 'VENDA'"
        ).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;
        
        let mut rows_check = stmt_check.query([venda_id]).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;
        if rows_check.next().map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?.is_some() {
            return Ok(()); // Já baixado, ignorar silenciosamente
        }
    }

    // 2. Buscar itens ativos e que controlam estoque
    let mut stmt_itens = tx.prepare("
        SELECT vi.produto_id, SUM(vi.quantidade_escala3) as total_escala3
        FROM venda_itens vi
        INNER JOIN produtos_cache p ON vi.produto_id = p.id
        WHERE vi.venda_id = ? AND vi.cancelado = 0 AND p.controla_estoque = 1
        GROUP BY vi.produto_id
    ").map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;
    
    let rows = stmt_itens.query_map([venda_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?
        ))
    }).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

    let mut itens_para_baixar = vec![];
    for r in rows {
        itens_para_baixar.push(r.map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?);
    }
    drop(stmt_itens);

    // 3. Processar cada produto
    for (produto_id, qtd_escala3) in itens_para_baixar {
        let saldo_atual = obter_saldo_produto(tx, &produto_id)?;
        let novo_saldo = saldo_atual - qtd_escala3;
        
        garantir_saldo_produto(tx, &produto_id, novo_saldo)?;
        
        registrar_movimentacao_estoque(
            tx,
            &produto_id,
            -qtd_escala3,
            novo_saldo,
            "VENDA",
            "VENDA",
            venda_id,
            Some("Baixa automatica na finalizacao da venda"),
            usuario_id,
        )?;
    }

    Ok(())
}

/// Processa o estorno de estoque ao cancelar uma venda finalizada.
pub fn processar_estorno_venda(
    tx: &Transaction,
    venda_id: &str,
    usuario_id: &str,
) -> Result<(), AureonError> {
    // 1. Validar se houve baixa (não dá pra estornar o que não baixou)
    {
        let mut stmt_check_baixa = tx.prepare(
            "SELECT id FROM estoque_movimentacoes WHERE origem_id = ? AND tipo_movimentacao = 'VENDA'"
        ).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;
        
        let mut rows_check_baixa = stmt_check_baixa.query([venda_id]).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;
        if rows_check_baixa.next().map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?.is_none() {
            return Ok(()); // Sem baixa, logo sem estorno
        }
    }

    // 2. Idempotência: verificar se já existe estorno
    {
        let mut stmt_check_estorno = tx.prepare(
            "SELECT id FROM estoque_movimentacoes WHERE origem_id = ? AND tipo_movimentacao = 'ESTORNO_VENDA'"
        ).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;
        
        let mut rows_check_estorno = stmt_check_estorno.query([venda_id]).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;
        if rows_check_estorno.next().map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?.is_some() {
            return Ok(()); // Já estornado, ignorar silenciosamente
        }
    }

    // 3. Buscar itens que controlam estoque (soma igual à da baixa)
    let mut stmt_itens = tx.prepare("
        SELECT vi.produto_id, SUM(vi.quantidade_escala3) as total_escala3
        FROM venda_itens vi
        INNER JOIN produtos_cache p ON vi.produto_id = p.id
        WHERE vi.venda_id = ? AND vi.cancelado = 0 AND p.controla_estoque = 1
        GROUP BY vi.produto_id
    ").map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;
    
    let rows = stmt_itens.query_map([venda_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?
        ))
    }).map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?;

    let mut itens_para_estornar = vec![];
    for r in rows {
        itens_para_estornar.push(r.map_err(|e| AureonError::ConexaoSqlite(e.to_string()))?);
    }
    drop(stmt_itens);

    // 4. Processar cada produto (soma positiva)
    for (produto_id, qtd_escala3) in itens_para_estornar {
        let saldo_atual = obter_saldo_produto(tx, &produto_id)?;
        let novo_saldo = saldo_atual + qtd_escala3;
        
        garantir_saldo_produto(tx, &produto_id, novo_saldo)?;
        
        registrar_movimentacao_estoque(
            tx,
            &produto_id,
            qtd_escala3,
            novo_saldo,
            "ESTORNO_VENDA",
            "VENDA",
            venda_id,
            Some("Estorno automatico por cancelamento de venda"),
            usuario_id,
        )?;
    }

    Ok(())
}

// ========================================================================
// COMMANDS TAURI - ESTOQUE
// ========================================================================

#[command]
pub fn consultar_saldos_estoque(
    estado: State<'_, EstadoApp>,
    busca: Option<String>,
) -> Result<RespostaBase<Vec<EstoqueSaldoResp>>, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let mut query = "
        SELECT p.id, p.codigo, p.descricao, p.controla_estoque,
               COALESCE(e.quantidade_escala3, 0) as quantidade_escala3,
               e.atualizado_em
        FROM produtos_cache p
        LEFT JOIN produtos_estoque_cache e ON p.id = e.produto_id
        WHERE p.controla_estoque = 1
    ".to_string();

    let mut params: Vec<String> = vec![];

    if let Some(termo) = busca {
        let t = format!("%{}%", termo);
        query.push_str(" AND (p.descricao LIKE ?1 OR p.codigo LIKE ?1)");
        params.push(t);
    }

    query.push_str(" ORDER BY p.descricao ASC");

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
    
    let mut saldos = vec![];

    if params.is_empty() {
        let rows = stmt.query_map([], |row| {
            Ok(EstoqueSaldoResp {
                produto_id: row.get(0)?,
                codigo: row.get(1)?,
                descricao: row.get(2)?,
                controla_estoque: row.get(3).unwrap_or(1) != 0,
                quantidade_escala3: row.get(4)?,
                atualizado_em: row.get(5)?,
            })
        }).map_err(|e| e.to_string())?;
        for r in rows { saldos.push(r.map_err(|e| e.to_string())?); }
    } else {
        let p = params.iter().map(|s| s as &dyn rusqlite::ToSql).collect::<Vec<_>>();
        let rows = stmt.query_map(p.as_slice(), |row| {
            Ok(EstoqueSaldoResp {
                produto_id: row.get(0)?,
                codigo: row.get(1)?,
                descricao: row.get(2)?,
                controla_estoque: row.get(3).unwrap_or(1) != 0,
                quantidade_escala3: row.get(4)?,
                atualizado_em: row.get(5)?,
            })
        }).map_err(|e| e.to_string())?;
        for r in rows { saldos.push(r.map_err(|e| e.to_string())?); }
    }

    Ok(RespostaBase {
        sucesso: true,
        mensagem: "Saldos obtidos com sucesso".into(),
        dados: Some(saldos),
        erro: None,
    })
}

#[command]
pub fn listar_kardex_produto(
    estado: State<'_, EstadoApp>,
    produto_id: String,
) -> Result<RespostaBase<Vec<EstoqueMovimentacaoResp>>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare("
        SELECT id, produto_id, quantidade_escala3, saldo_apos_escala3,
               tipo_movimentacao, origem_tipo, origem_id, motivo, usuario_id, criado_em
        FROM estoque_movimentacoes
        WHERE produto_id = ?
        ORDER BY criado_em DESC
        LIMIT 100
    ").map_err(|e| e.to_string())?;

    let rows = stmt.query_map([produto_id], |row| {
        Ok(EstoqueMovimentacaoResp {
            id: row.get(0)?,
            produto_id: row.get(1)?,
            quantidade_escala3: row.get(2)?,
            saldo_apos_escala3: row.get(3)?,
            tipo_movimentacao: row.get(4)?,
            origem_tipo: row.get(5)?,
            origem_id: row.get(6)?,
            motivo: row.get(7)?,
            usuario_id: row.get(8)?,
            criado_em: row.get(9)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut movs = vec![];
    for r in rows {
        movs.push(r.map_err(|e| e.to_string())?);
    }

    Ok(RespostaBase {
        sucesso: true,
        mensagem: "Kardex obtido com sucesso".into(),
        dados: Some(movs),
        erro: None,
    })
}

#[command]
pub fn ajustar_estoque_manual(
    estado: State<'_, EstadoApp>,
    dto: AjusteEstoqueReq,
) -> Result<RespostaBase<String>, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    if dto.quantidade_escala3 <= 0 {
        return Ok(RespostaBase {
            sucesso: false,
            mensagem: "Quantidade de ajuste deve ser maior que 0.".into(),
            dados: None,
            erro: None,
        });
    }

    if dto.motivo.trim().is_empty() {
        return Ok(RespostaBase {
            sucesso: false,
            mensagem: "O motivo é obrigatório para ajustes manuais.".into(),
            dados: None,
            erro: None,
        });
    }

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    // Validar produto e controla_estoque usando escopo explícito
    {
        let mut stmt_prod = tx.prepare("SELECT controla_estoque FROM produtos_cache WHERE id = ?").map_err(|e| e.to_string())?;
        let mut rows_prod = stmt_prod.query([&dto.produto_id]).map_err(|e| e.to_string())?;
        
        if let Some(row) = rows_prod.next().map_err(|e| e.to_string())? {
            let controla: i32 = row.get(0).unwrap_or(1);
            if controla == 0 {
                return Ok(RespostaBase {
                    sucesso: false,
                    mensagem: "Produto não controla estoque.".into(),
                    dados: None,
                    erro: None,
                });
            }
        } else {
            return Ok(RespostaBase {
                sucesso: false,
                mensagem: "Produto não encontrado.".into(),
                dados: None,
                erro: None,
            });
        }
    } // rows_prod e stmt_prod dropados aqui

    // Idempotência
    let origem_id = dto.idempotency_key.unwrap_or_else(|| Uuid::new_v4().to_string());
    
    {
        let mut stmt_check = tx.prepare("SELECT id FROM estoque_movimentacoes WHERE origem_id = ? AND tipo_movimentacao = ?").map_err(|e| e.to_string())?;
        let mut rows_check = stmt_check.query((&origem_id, &dto.tipo_ajuste)).map_err(|e| e.to_string())?;
        if rows_check.next().map_err(|e| e.to_string())?.is_some() {
            return Ok(RespostaBase {
                sucesso: true,
                mensagem: "Ajuste já processado anteriormente (idempotência).".into(),
                dados: Some(origem_id.clone()),
                erro: None,
            });
        }
    } // rows_check e stmt_check dropados aqui

    let saldo_atual = obter_saldo_produto(&tx, &dto.produto_id).map_err(|e| e.to_string())?;
    
    let (delta, novo_saldo) = if dto.tipo_ajuste == "AJUSTE_ENTRADA" {
        (dto.quantidade_escala3, saldo_atual + dto.quantidade_escala3)
    } else if dto.tipo_ajuste == "AJUSTE_SAIDA" {
        (-dto.quantidade_escala3, saldo_atual - dto.quantidade_escala3)
    } else {
        return Ok(RespostaBase {
            sucesso: false,
            mensagem: "Tipo de ajuste inválido.".into(),
            dados: None,
            erro: None,
        });
    };

    garantir_saldo_produto(&tx, &dto.produto_id, novo_saldo).map_err(|e| e.to_string())?;

    registrar_movimentacao_estoque(
        &tx,
        &dto.produto_id,
        delta,
        novo_saldo,
        &dto.tipo_ajuste,
        "AJUSTE_MANUAL",
        &origem_id,
        Some(&dto.motivo),
        &dto.usuario_id,
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(RespostaBase {
        sucesso: true,
        mensagem: "Ajuste realizado com sucesso.".into(),
        dados: Some(origem_id),
        erro: None,
    })
}

#[command]
pub fn registrar_inventario(
    estado: State<'_, EstadoApp>,
    dto: InventarioEstoqueReq,
) -> Result<RespostaBase<String>, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    let origem_id = dto.idempotency_key.unwrap_or_else(|| Uuid::new_v4().to_string());

    // Idempotência
    {
        let mut stmt_check = tx.prepare("SELECT id FROM estoque_movimentacoes WHERE origem_id = ? AND tipo_movimentacao = 'INVENTARIO'").map_err(|e| e.to_string())?;
        let mut rows_check = stmt_check.query([&origem_id]).map_err(|e| e.to_string())?;
        if rows_check.next().map_err(|e| e.to_string())?.is_some() {
            return Ok(RespostaBase {
                sucesso: true,
                mensagem: "Inventário já processado anteriormente (idempotência).".into(),
                dados: Some(origem_id.clone()),
                erro: None,
            });
        }
    } // rows_check e stmt_check dropados aqui

    let mut count_alteracoes = 0;

    for contagem in dto.contagens {
        let saldo_atual = obter_saldo_produto(&tx, &contagem.produto_id).map_err(|e| e.to_string())?;
        
        let delta = contagem.saldo_real_escala3 - saldo_atual;
        
        if delta != 0 {
            garantir_saldo_produto(&tx, &contagem.produto_id, contagem.saldo_real_escala3).map_err(|e| e.to_string())?;
            
            registrar_movimentacao_estoque(
                &tx,
                &contagem.produto_id,
                delta,
                contagem.saldo_real_escala3,
                "INVENTARIO",
                "INVENTARIO",
                &origem_id,
                dto.motivo.as_deref(),
                &dto.usuario_id,
            ).map_err(|e| e.to_string())?;

            count_alteracoes += 1;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;

    Ok(RespostaBase {
        sucesso: true,
        mensagem: format!("Inventário registrado com sucesso. {} produtos alterados.", count_alteracoes),
        dados: Some(origem_id),
        erro: None,
    })
}
