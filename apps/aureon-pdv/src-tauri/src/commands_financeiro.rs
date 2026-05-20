use aureon_core::{dtos::*, RespostaBase};
use serde_json::json;
use tauri::{command, State};
use chrono::Utc;
use uuid::Uuid;

use crate::commands_caixa::inserir_outbox;
use crate::estado::EstadoApp;

fn obter_nome_fornecedor(conn: &rusqlite::Connection, id: &str) -> Result<String, String> {
    conn.query_row(
        "SELECT nome FROM fornecedores_cache WHERE id = ?1 AND ativo = 1",
        [id],
        |row| row.get(0)
    ).map_err(|_| "Fornecedor não encontrado ou inativo".to_string())
}

#[command]
pub async fn listar_contas_pagar(
    estado: State<'_, EstadoApp>,
    status: Option<String>,
    fornecedor_id: Option<String>,
) -> Result<RespostaBase<Vec<ContaPagarResp>>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let mut sql = "SELECT id, fornecedor_id, fornecedor_nome_snapshot, compra_id, descricao, moeda_codigo,
                          valor_original_minor, taxa_cambio_escala6, valor_original_principal_minor,
                          data_emissao, data_vencimento, status, saldo_pendente_minor, criado_em,
                          atualizado_em, usuario_id, observacao 
                   FROM contas_pagar WHERE 1=1".to_string();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];
    
    if let Some(ref st) = status {
        sql.push_str(" AND status = ?");
        params.push(Box::new(st.clone()));
    }
    if let Some(ref f_id) = fornecedor_id {
        sql.push_str(" AND fornecedor_id = ?");
        params.push(Box::new(f_id.clone()));
    }
    sql.push_str(" ORDER BY data_vencimento ASC");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    
    let rows = stmt.query_map(&*params_refs, |row| {
        Ok(ContaPagarResp {
            id: row.get(0)?,
            fornecedor_id: row.get(1)?,
            fornecedor_nome_snapshot: row.get(2)?,
            compra_id: row.get(3)?,
            descricao: row.get(4)?,
            moeda_codigo: row.get(5)?,
            valor_original_minor: row.get(6)?,
            taxa_cambio_escala6: row.get(7)?,
            valor_original_principal_minor: row.get(8)?,
            data_emissao: row.get(9)?,
            data_vencimento: row.get(10)?,
            status: row.get(11)?,
            saldo_pendente_minor: row.get(12)?,
            criado_em: row.get(13)?,
            atualizado_em: row.get(14)?,
            usuario_id: row.get(15)?,
            observacao: row.get(16)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut list = vec![];
    for r in rows {
        list.push(r.map_err(|e| e.to_string())?);
    }
    Ok(RespostaBase::ok("", list))
}

#[command]
pub async fn obter_conta_pagar(
    estado: State<'_, EstadoApp>,
    id: String,
) -> Result<RespostaBase<ContaPagarResp>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let res = conn.query_row(
        "SELECT id, fornecedor_id, fornecedor_nome_snapshot, compra_id, descricao, moeda_codigo,
                valor_original_minor, taxa_cambio_escala6, valor_original_principal_minor,
                data_emissao, data_vencimento, status, saldo_pendente_minor, criado_em,
                atualizado_em, usuario_id, observacao 
         FROM contas_pagar WHERE id = ?1",
        [&id],
        |row| {
            Ok(ContaPagarResp {
                id: row.get(0)?,
                fornecedor_id: row.get(1)?,
                fornecedor_nome_snapshot: row.get(2)?,
                compra_id: row.get(3)?,
                descricao: row.get(4)?,
                moeda_codigo: row.get(5)?,
                valor_original_minor: row.get(6)?,
                taxa_cambio_escala6: row.get(7)?,
                valor_original_principal_minor: row.get(8)?,
                data_emissao: row.get(9)?,
                data_vencimento: row.get(10)?,
                status: row.get(11)?,
                saldo_pendente_minor: row.get(12)?,
                criado_em: row.get(13)?,
                atualizado_em: row.get(14)?,
                usuario_id: row.get(15)?,
                observacao: row.get(16)?,
            })
        }
    );
    match res {
        Ok(c) => Ok(RespostaBase::ok("", c)),
        Err(_) => Ok(RespostaBase::falha_manual("Conta a pagar não encontrada", "ERR_FIN", "")),
    }
}

#[command]
pub async fn registrar_despesa_manual(
    estado: State<'_, EstadoApp>,
    dto: RegistrarDespesaReq,
) -> Result<RespostaBase<ContaPagarResp>, String> {
    if dto.descricao.trim().is_empty() {
        return Ok(RespostaBase::falha_manual("Descrição é obrigatória", "ERR_FIN", ""));
    }
    if dto.valor_original_minor <= 0 {
        return Ok(RespostaBase::falha_manual("Valor deve ser maior que zero", "ERR_FIN", ""));
    }
    if dto.moeda_codigo.trim().is_empty() {
        return Ok(RespostaBase::falha_manual("Moeda é obrigatória", "ERR_FIN", ""));
    }
    if dto.taxa_cambio_escala6 <= 0 {
        return Ok(RespostaBase::falha_manual("Taxa de câmbio inválida", "ERR_FIN", ""));
    }
    if dto.data_emissao.trim().is_empty() {
        return Ok(RespostaBase::falha_manual("Data de emissão é obrigatória", "ERR_FIN", ""));
    }
    if dto.data_vencimento.trim().is_empty() {
        return Ok(RespostaBase::falha_manual("Data de vencimento é obrigatória", "ERR_FIN", ""));
    }

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    
    let fornecedor_nome_snapshot = if let Some(ref f_id) = dto.fornecedor_id {
        if f_id.trim().is_empty() {
            None
        } else {
            match obter_nome_fornecedor(&conn, f_id) {
                Ok(nome) => Some(nome),
                Err(e) => return Ok(RespostaBase::falha_manual(e, "ERR_FIN", "")),
            }
        }
    } else {
        None
    };

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    
    let valor_principal_minor = match dto.valor_original_minor.checked_mul(dto.taxa_cambio_escala6) {
        Some(val) => val / 1_000_000,
        None => return Ok(RespostaBase::falha_manual("Erro aritmético no cálculo do valor principal", "ERR_FIN", "")),
    };

    tx.execute(
        "INSERT INTO contas_pagar (
            id, fornecedor_id, fornecedor_nome_snapshot, compra_id, descricao, moeda_codigo,
            valor_original_minor, taxa_cambio_escala6, valor_original_principal_minor,
            data_emissao, data_vencimento, status, saldo_pendente_minor, criado_em,
            atualizado_em, usuario_id, observacao
        ) VALUES (?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'PENDENTE', ?6, ?11, ?11, ?12, ?13)",
        rusqlite::params![
            id, dto.fornecedor_id, fornecedor_nome_snapshot, dto.descricao, dto.moeda_codigo,
            dto.valor_original_minor, dto.taxa_cambio_escala6, valor_principal_minor,
            dto.data_emissao, dto.data_vencimento, agora, dto.usuario_id, dto.observacao
        ],
    ).map_err(|e| e.to_string())?;

    let resp = ContaPagarResp {
        id: id.clone(),
        fornecedor_id: dto.fornecedor_id,
        fornecedor_nome_snapshot,
        compra_id: None,
        descricao: dto.descricao,
        moeda_codigo: dto.moeda_codigo,
        valor_original_minor: dto.valor_original_minor,
        taxa_cambio_escala6: dto.taxa_cambio_escala6,
        valor_original_principal_minor: valor_principal_minor,
        data_emissao: dto.data_emissao,
        data_vencimento: dto.data_vencimento,
        status: "PENDENTE".to_string(),
        saldo_pendente_minor: dto.valor_original_minor,
        criado_em: agora.clone(),
        atualizado_em: agora.clone(),
        usuario_id: dto.usuario_id,
        observacao: dto.observacao,
    };

    inserir_outbox(
        &tx,
        "CONTA_PAGAR_CRIADA",
        json!(&resp),
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(RespostaBase::ok("Despesa manual registrada com sucesso", resp))
}

#[command]
pub async fn baixar_conta_pagar(
    estado: State<'_, EstadoApp>,
    dto: BaixarContaPagarReq,
) -> Result<RespostaBase<ContaPagarResp>, String> {
    if dto.valor_informado_minor <= 0 {
        return Ok(RespostaBase::falha_manual("O valor da baixa deve ser maior que zero", "ERR_FIN", ""));
    }
    if dto.taxa_cambio_escala6 <= 0 {
        return Ok(RespostaBase::falha_manual("A taxa de câmbio da baixa deve ser maior que zero", "ERR_FIN", ""));
    }

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // 1. Validar se a sessão está aberta
    let status_sessao: String = tx
        .query_row(
            "SELECT status FROM sessoes_caixa WHERE id = ?1",
            rusqlite::params![&dto.sessao_caixa_id],
            |r| r.get(0),
        )
        .map_err(|_| "Sessão de caixa não encontrada".to_string())?;

    if status_sessao != "ABERTO" {
        return Ok(RespostaBase::falha_manual("A sessão de caixa não está aberta", "ERR_FIN", ""));
    }

    // 2. Obter conta a pagar
    let (status_titulo, moeda_titulo, saldo_pendente_minor) = match tx.query_row(
        "SELECT status, moeda_codigo, saldo_pendente_minor FROM contas_pagar WHERE id = ?1",
        rusqlite::params![&dto.conta_pagar_id],
        |row| Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
        ))
    ) {
        Ok(res) => res,
        Err(_) => return Ok(RespostaBase::falha_manual("Conta a pagar não encontrada", "ERR_FIN", "")),
    };

    if status_titulo == "CANCELADO" {
        return Ok(RespostaBase::falha_manual("Não é possível baixar um título cancelado", "ERR_FIN", ""));
    }
    if status_titulo == "PAGO" {
        return Ok(RespostaBase::falha_manual("Este título já está totalmente pago", "ERR_FIN", ""));
    }

    // Validar se moeda de pagamento coincide com a moeda do título
    if dto.moeda_codigo != moeda_titulo {
        return Ok(RespostaBase::falha_manual(
            format!("Moeda de pagamento ({}) deve coincidir com a moeda do título ({})", dto.moeda_codigo, moeda_titulo),
            "ERR_FIN",
            ""
        ));
    }

    if dto.valor_informado_minor > saldo_pendente_minor {
        return Ok(RespostaBase::falha_manual(
            format!("Valor da baixa ({}) não pode exceder o saldo pendente ({})", dto.valor_informado_minor, saldo_pendente_minor),
            "ERR_FIN",
            ""
        ));
    }

    // 3. Atualizar saldo e status
    let novo_saldo = saldo_pendente_minor - dto.valor_informado_minor;
    let novo_status = if novo_saldo == 0 { "PAGO" } else { "PAGO_PARCIAL" };
    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    tx.execute(
        "UPDATE contas_pagar 
         SET saldo_pendente_minor = ?1, status = ?2, atualizado_em = ?3
         WHERE id = ?4",
        rusqlite::params![novo_saldo, novo_status, agora, &dto.conta_pagar_id],
    ).map_err(|e| e.to_string())?;

    // 4. Inserir lançamento financeiro (INSERT ONLY)
    let lancamento_id = Uuid::new_v4().to_string();
    let valor_principal_minor = match dto.valor_informado_minor.checked_mul(dto.taxa_cambio_escala6) {
        Some(val) => val / 1_000_000,
        None => return Ok(RespostaBase::falha_manual("Erro aritmético no cálculo de valor principal", "ERR_FIN", "")),
    };

    tx.execute(
        "INSERT INTO financeiro_lancamentos (
            id, conta_pagar_id, conta_receber_id, sessao_caixa_id, tipo_lancamento,
            forma_pagamento, moeda_codigo, valor_informado_minor, taxa_cambio_escala6,
            valor_principal_minor, data_pagamento, usuario_id, observacao, criado_em
        ) VALUES (?1, ?2, NULL, ?3, 'PAGAMENTO', ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?9)",
        rusqlite::params![
            lancamento_id, dto.conta_pagar_id, dto.sessao_caixa_id, dto.forma_pagamento,
            dto.moeda_codigo, dto.valor_informado_minor, dto.taxa_cambio_escala6,
            valor_principal_minor, agora, dto.usuario_id, dto.observacao
        ],
    ).map_err(|e| e.to_string())?;

    // Obter resposta final atualizada
    let conta_atualizada = tx.query_row(
        "SELECT id, fornecedor_id, fornecedor_nome_snapshot, compra_id, descricao, moeda_codigo,
                valor_original_minor, taxa_cambio_escala6, valor_original_principal_minor,
                data_emissao, data_vencimento, status, saldo_pendente_minor, criado_em,
                atualizado_em, usuario_id, observacao 
         FROM contas_pagar WHERE id = ?1",
        [&dto.conta_pagar_id],
        |row| {
            Ok(ContaPagarResp {
                id: row.get(0)?,
                fornecedor_id: row.get(1)?,
                fornecedor_nome_snapshot: row.get(2)?,
                compra_id: row.get(3)?,
                descricao: row.get(4)?,
                moeda_codigo: row.get(5)?,
                valor_original_minor: row.get(6)?,
                taxa_cambio_escala6: row.get(7)?,
                valor_original_principal_minor: row.get(8)?,
                data_emissao: row.get(9)?,
                data_vencimento: row.get(10)?,
                status: row.get(11)?,
                saldo_pendente_minor: row.get(12)?,
                criado_em: row.get(13)?,
                atualizado_em: row.get(14)?,
                usuario_id: row.get(15)?,
                observacao: row.get(16)?,
            })
        }
    ).map_err(|e| e.to_string())?;

    // 5. Inserir eventos sync_outbox
    inserir_outbox(
        &tx,
        "CONTA_PAGAR_BAIXADA",
        json!({
            "conta_pagar_id": dto.conta_pagar_id,
            "novo_saldo_pendente_minor": novo_saldo,
            "novo_status": novo_status,
            "valor_baixado_minor": dto.valor_informado_minor,
            "moeda_codigo": dto.moeda_codigo,
            "taxa_cambio_escala6": dto.taxa_cambio_escala6,
            "atualizado_em": agora
        }),
    ).map_err(|e| e.to_string())?;

    inserir_outbox(
        &tx,
        "FINANCEIRO_LANCAMENTO_GERADO",
        json!({
            "id": lancamento_id,
            "conta_pagar_id": dto.conta_pagar_id,
            "sessao_caixa_id": dto.sessao_caixa_id,
            "tipo_lancamento": "PAGAMENTO",
            "forma_pagamento": dto.forma_pagamento,
            "moeda_codigo": dto.moeda_codigo,
            "valor_informado_minor": dto.valor_informado_minor,
            "taxa_cambio_escala6": dto.taxa_cambio_escala6,
            "valor_principal_minor": valor_principal_minor,
            "data_pagamento": agora,
            "usuario_id": dto.usuario_id
        }),
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(RespostaBase::ok("Baixa de conta a pagar realizada com sucesso", conta_atualizada))
}

#[command]
pub async fn cancelar_conta_pagar(
    estado: State<'_, EstadoApp>,
    dto: CancelarContaPagarReq,
) -> Result<RespostaBase<ContaPagarResp>, String> {
    if dto.motivo.trim().is_empty() {
        return Ok(RespostaBase::falha_manual("Motivo do cancelamento é obrigatório", "ERR_FIN", ""));
    }

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // Obter status atual
    let status_atual: String = match tx.query_row(
        "SELECT status FROM contas_pagar WHERE id = ?1",
        rusqlite::params![&dto.conta_pagar_id],
        |row| row.get(0)
    ) {
        Ok(status) => status,
        Err(_) => return Ok(RespostaBase::falha_manual("Conta a pagar não encontrada", "ERR_FIN", "")),
    };

    if status_atual != "PENDENTE" {
        return Ok(RespostaBase::falha_manual("Apenas contas com status PENDENTE podem ser canceladas", "ERR_FIN", ""));
    }

    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Ao cancelar, o saldo_pendente_minor é zerado para fins de auditoria, restando apenas o valor_original_minor como histórico
    tx.execute(
        "UPDATE contas_pagar
         SET status = 'CANCELADO', saldo_pendente_minor = 0, atualizado_em = ?1, observacao = ?2
         WHERE id = ?3",
        rusqlite::params![agora, dto.motivo, &dto.conta_pagar_id],
    ).map_err(|e| e.to_string())?;

    let conta_atualizada = tx.query_row(
        "SELECT id, fornecedor_id, fornecedor_nome_snapshot, compra_id, descricao, moeda_codigo,
                valor_original_minor, taxa_cambio_escala6, valor_original_principal_minor,
                data_emissao, data_vencimento, status, saldo_pendente_minor, criado_em,
                atualizado_em, usuario_id, observacao 
         FROM contas_pagar WHERE id = ?1",
        [&dto.conta_pagar_id],
        |row| {
            Ok(ContaPagarResp {
                id: row.get(0)?,
                fornecedor_id: row.get(1)?,
                fornecedor_nome_snapshot: row.get(2)?,
                compra_id: row.get(3)?,
                descricao: row.get(4)?,
                moeda_codigo: row.get(5)?,
                valor_original_minor: row.get(6)?,
                taxa_cambio_escala6: row.get(7)?,
                valor_original_principal_minor: row.get(8)?,
                data_emissao: row.get(9)?,
                data_vencimento: row.get(10)?,
                status: row.get(11)?,
                saldo_pendente_minor: row.get(12)?,
                criado_em: row.get(13)?,
                atualizado_em: row.get(14)?,
                usuario_id: row.get(15)?,
                observacao: row.get(16)?,
            })
        }
    ).map_err(|e| e.to_string())?;

    inserir_outbox(
        &tx,
        "CONTA_PAGAR_CANCELADA",
        json!({
            "conta_pagar_id": dto.conta_pagar_id,
            "cancelado_em": agora,
            "motivo": dto.motivo,
            "usuario_id": dto.usuario_id
        }),
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(RespostaBase::ok("Conta a pagar cancelada com sucesso", conta_atualizada))
}

#[command]
pub async fn listar_lancamentos_financeiros(
    estado: State<'_, EstadoApp>,
    conta_pagar_id: Option<String>,
    conta_receber_id: Option<String>,
    sessao_caixa_id: Option<String>,
) -> Result<RespostaBase<Vec<FinanceiroLancamentoResp>>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let mut sql = "SELECT id, conta_pagar_id, conta_receber_id, sessao_caixa_id, tipo_lancamento,
                          forma_pagamento, moeda_codigo, valor_informado_minor, taxa_cambio_escala6,
                          valor_principal_minor, data_pagamento, usuario_id, observacao, criado_em 
                   FROM financeiro_lancamentos WHERE 1=1".to_string();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];
    
    if let Some(ref cp_id) = conta_pagar_id {
        sql.push_str(" AND conta_pagar_id = ?");
        params.push(Box::new(cp_id.clone()));
    }
    if let Some(ref cr_id) = conta_receber_id {
        sql.push_str(" AND conta_receber_id = ?");
        params.push(Box::new(cr_id.clone()));
    }
    if let Some(ref sc_id) = sessao_caixa_id {
        sql.push_str(" AND sessao_caixa_id = ?");
        params.push(Box::new(sc_id.clone()));
    }
    sql.push_str(" ORDER BY criado_em DESC");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    
    let rows = stmt.query_map(&*params_refs, |row| {
        Ok(FinanceiroLancamentoResp {
            id: row.get(0)?,
            conta_pagar_id: row.get(1)?,
            conta_receber_id: row.get(2)?,
            sessao_caixa_id: row.get(3)?,
            tipo_lancamento: row.get(4)?,
            forma_pagamento: row.get(5)?,
            moeda_codigo: row.get(6)?,
            valor_informado_minor: row.get(7)?,
            taxa_cambio_escala6: row.get(8)?,
            valor_principal_minor: row.get(9)?,
            data_pagamento: row.get(10)?,
            usuario_id: row.get(11)?,
            observacao: row.get(12)?,
            criado_em: row.get(13)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut list = vec![];
    for r in rows {
        list.push(r.map_err(|e| e.to_string())?);
    }
    Ok(RespostaBase::ok("", list))
}

#[command]
pub async fn listar_contas_receber(
    estado: State<'_, EstadoApp>,
    status: Option<String>,
    cliente_id: Option<String>,
) -> Result<RespostaBase<Vec<ContaReceberResp>>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let mut sql = "SELECT id, cliente_id, cliente_nome_snapshot, venda_id, descricao, moeda_codigo,
                          valor_original_minor, taxa_cambio_escala6, valor_original_principal_minor,
                          data_emissao, data_vencimento, status, saldo_pendente_minor, criado_em,
                          atualizado_em, usuario_id, observacao 
                   FROM contas_receber WHERE 1=1".to_string();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];
    
    if let Some(ref st) = status {
        sql.push_str(" AND status = ?");
        params.push(Box::new(st.clone()));
    }
    if let Some(ref c_id) = cliente_id {
        sql.push_str(" AND cliente_id = ?");
        params.push(Box::new(c_id.clone()));
    }
    sql.push_str(" ORDER BY data_vencimento ASC");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    
    let rows = stmt.query_map(&*params_refs, |row| {
        Ok(ContaReceberResp {
            id: row.get(0)?,
            cliente_id: row.get(1)?,
            cliente_nome_snapshot: row.get(2)?,
            venda_id: row.get(3)?,
            descricao: row.get(4)?,
            moeda_codigo: row.get(5)?,
            valor_original_minor: row.get(6)?,
            taxa_cambio_escala6: row.get(7)?,
            valor_original_principal_minor: row.get(8)?,
            data_emissao: row.get(9)?,
            data_vencimento: row.get(10)?,
            status: row.get(11)?,
            saldo_pendente_minor: row.get(12)?,
            criado_em: row.get(13)?,
            atualizado_em: row.get(14)?,
            usuario_id: row.get(15)?,
            observacao: row.get(16)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut list = vec![];
    for r in rows {
        list.push(r.map_err(|e| e.to_string())?);
    }
    Ok(RespostaBase::ok("", list))
}

#[command]
pub async fn obter_conta_receber(
    estado: State<'_, EstadoApp>,
    id: String,
) -> Result<RespostaBase<ContaReceberResp>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let res = conn.query_row(
        "SELECT id, cliente_id, cliente_nome_snapshot, venda_id, descricao, moeda_codigo,
                valor_original_minor, taxa_cambio_escala6, valor_original_principal_minor,
                data_emissao, data_vencimento, status, saldo_pendente_minor, criado_em,
                atualizado_em, usuario_id, observacao 
         FROM contas_receber WHERE id = ?1",
        [&id],
        |row| {
            Ok(ContaReceberResp {
                id: row.get(0)?,
                cliente_id: row.get(1)?,
                cliente_nome_snapshot: row.get(2)?,
                venda_id: row.get(3)?,
                descricao: row.get(4)?,
                moeda_codigo: row.get(5)?,
                valor_original_minor: row.get(6)?,
                taxa_cambio_escala6: row.get(7)?,
                valor_original_principal_minor: row.get(8)?,
                data_emissao: row.get(9)?,
                data_vencimento: row.get(10)?,
                status: row.get(11)?,
                saldo_pendente_minor: row.get(12)?,
                criado_em: row.get(13)?,
                atualizado_em: row.get(14)?,
                usuario_id: row.get(15)?,
                observacao: row.get(16)?,
            })
        }
    );
    match res {
        Ok(c) => Ok(RespostaBase::ok("", c)),
        Err(_) => Ok(RespostaBase::falha_manual("Conta a receber não encontrada", "ERR_FIN", "")),
    }
}

#[command]
pub async fn baixar_conta_receber(
    estado: State<'_, EstadoApp>,
    dto: BaixarContaReceberReq,
) -> Result<RespostaBase<ContaReceberResp>, String> {
    if dto.valor_informado_minor <= 0 {
        return Ok(RespostaBase::falha_manual("O valor da baixa deve ser maior que zero", "ERR_FIN", ""));
    }
    if dto.taxa_cambio_escala6 <= 0 {
        return Ok(RespostaBase::falha_manual("A taxa de câmbio da baixa deve ser maior que zero", "ERR_FIN", ""));
    }

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // 1. Validar se a sessão de caixa está aberta
    let status_sessao: String = tx
        .query_row(
            "SELECT status FROM sessoes_caixa WHERE id = ?1",
            rusqlite::params![&dto.sessao_caixa_id],
            |r| r.get(0),
        )
        .map_err(|_| "Sessão de caixa não encontrada".to_string())?;

    if status_sessao != "ABERTO" {
        return Ok(RespostaBase::falha_manual("A sessão de caixa não está aberta", "ERR_FIN", ""));
    }

    // 2. Obter conta a receber
    let (status_titulo, moeda_titulo, saldo_pendente_minor) = match tx.query_row(
        "SELECT status, moeda_codigo, saldo_pendente_minor FROM contas_receber WHERE id = ?1",
        rusqlite::params![&dto.conta_receber_id],
        |row| Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
        ))
    ) {
        Ok(res) => res,
        Err(_) => return Ok(RespostaBase::falha_manual("Conta a receber não encontrada", "ERR_FIN", "")),
    };

    if status_titulo == "CANCELADO" {
        return Ok(RespostaBase::falha_manual("Não é possível baixar um título cancelado", "ERR_FIN", ""));
    }
    if status_titulo == "PAGO" {
        return Ok(RespostaBase::falha_manual("Este título já está totalmente pago", "ERR_FIN", ""));
    }

    // Validar se moeda coincide
    if dto.moeda_codigo != moeda_titulo {
        return Ok(RespostaBase::falha_manual(
            format!("Moeda de recebimento ({}) deve coincidir com a moeda do título ({})", dto.moeda_codigo, moeda_titulo),
            "ERR_FIN",
            ""
        ));
    }

    if dto.valor_informado_minor > saldo_pendente_minor {
        return Ok(RespostaBase::falha_manual(
            format!("Valor do recebimento ({}) não pode exceder o saldo pendente ({})", dto.valor_informado_minor, saldo_pendente_minor),
            "ERR_FIN",
            ""
        ));
    }

    // 3. Atualizar saldo e status
    let novo_saldo = saldo_pendente_minor - dto.valor_informado_minor;
    let novo_status = if novo_saldo == 0 { "PAGO" } else { "PAGO_PARCIAL" };
    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    tx.execute(
        "UPDATE contas_receber 
         SET saldo_pendente_minor = ?1, status = ?2, atualizado_em = ?3
         WHERE id = ?4",
        rusqlite::params![novo_saldo, novo_status, agora, &dto.conta_receber_id],
    ).map_err(|e| e.to_string())?;

    // 4. Inserir lançamento financeiro (tipo_lancamento = RECEBIMENTO)
    let lancamento_id = Uuid::new_v4().to_string();
    let valor_principal_minor = match dto.valor_informado_minor.checked_mul(dto.taxa_cambio_escala6) {
        Some(val) => val / 1_000_000,
        None => return Ok(RespostaBase::falha_manual("Erro aritmético no cálculo de valor principal", "ERR_FIN", "")),
    };

    tx.execute(
        "INSERT INTO financeiro_lancamentos (
            id, conta_pagar_id, conta_receber_id, sessao_caixa_id, tipo_lancamento,
            forma_pagamento, moeda_codigo, valor_informado_minor, taxa_cambio_escala6,
            valor_principal_minor, data_pagamento, usuario_id, observacao, criado_em
        ) VALUES (?1, NULL, ?2, ?3, 'RECEBIMENTO', ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?9)",
        rusqlite::params![
            lancamento_id, dto.conta_receber_id, dto.sessao_caixa_id, dto.forma_pagamento,
            dto.moeda_codigo, dto.valor_informado_minor, dto.taxa_cambio_escala6,
            valor_principal_minor, agora, dto.usuario_id, dto.observacao
        ],
    ).map_err(|e| e.to_string())?;

    // Obter resposta final atualizada
    let conta_atualizada = tx.query_row(
        "SELECT id, cliente_id, cliente_nome_snapshot, venda_id, descricao, moeda_codigo,
                valor_original_minor, taxa_cambio_escala6, valor_original_principal_minor,
                data_emissao, data_vencimento, status, saldo_pendente_minor, criado_em,
                atualizado_em, usuario_id, observacao 
         FROM contas_receber WHERE id = ?1",
        [&dto.conta_receber_id],
        |row| {
            Ok(ContaReceberResp {
                id: row.get(0)?,
                cliente_id: row.get(1)?,
                cliente_nome_snapshot: row.get(2)?,
                venda_id: row.get(3)?,
                descricao: row.get(4)?,
                moeda_codigo: row.get(5)?,
                valor_original_minor: row.get(6)?,
                taxa_cambio_escala6: row.get(7)?,
                valor_original_principal_minor: row.get(8)?,
                data_emissao: row.get(9)?,
                data_vencimento: row.get(10)?,
                status: row.get(11)?,
                saldo_pendente_minor: row.get(12)?,
                criado_em: row.get(13)?,
                atualizado_em: row.get(14)?,
                usuario_id: row.get(15)?,
                observacao: row.get(16)?,
            })
        }
    ).map_err(|e| e.to_string())?;

    // 5. Inserir eventos sync_outbox
    inserir_outbox(
        &tx,
        "CONTA_RECEBER_BAIXADA",
        json!({
            "conta_receber_id": dto.conta_receber_id,
            "novo_saldo_pendente_minor": novo_saldo,
            "novo_status": novo_status,
            "valor_baixado_minor": dto.valor_informado_minor,
            "moeda_codigo": dto.moeda_codigo,
            "taxa_cambio_escala6": dto.taxa_cambio_escala6,
            "atualizado_em": agora
        }),
    ).map_err(|e| e.to_string())?;

    inserir_outbox(
        &tx,
        "FINANCEIRO_LANCAMENTO_GERADO",
        json!({
            "id": lancamento_id,
            "conta_receber_id": dto.conta_receber_id,
            "sessao_caixa_id": dto.sessao_caixa_id,
            "tipo_lancamento": "RECEBIMENTO",
            "forma_pagamento": dto.forma_pagamento,
            "moeda_codigo": dto.moeda_codigo,
            "valor_informado_minor": dto.valor_informado_minor,
            "taxa_cambio_escala6": dto.taxa_cambio_escala6,
            "valor_principal_minor": valor_principal_minor,
            "data_pagamento": agora,
            "usuario_id": dto.usuario_id
        }),
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(RespostaBase::ok("Recebimento de conta a receber realizado com sucesso", conta_atualizada))
}

#[command]
pub async fn cancelar_conta_receber(
    estado: State<'_, EstadoApp>,
    dto: CancelarContaReceberReq,
) -> Result<RespostaBase<ContaReceberResp>, String> {
    if dto.motivo.trim().is_empty() {
        return Ok(RespostaBase::falha_manual("Motivo do cancelamento é obrigatório", "ERR_FIN", ""));
    }

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // Obter status atual
    let status_atual: String = match tx.query_row(
        "SELECT status FROM contas_receber WHERE id = ?1",
        rusqlite::params![&dto.conta_receber_id],
        |row| row.get(0)
    ) {
        Ok(status) => status,
        Err(_) => return Ok(RespostaBase::falha_manual("Conta a receber não encontrada", "ERR_FIN", "")),
    };

    if status_atual != "PENDENTE" {
        return Ok(RespostaBase::falha_manual("Apenas contas com status PENDENTE podem ser canceladas", "ERR_FIN", ""));
    }

    let agora = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    tx.execute(
        "UPDATE contas_receber 
         SET status = 'CANCELADO', saldo_pendente_minor = 0, atualizado_em = ?1, observacao = ?2
         WHERE id = ?3",
        rusqlite::params![agora, dto.motivo, &dto.conta_receber_id],
    ).map_err(|e| e.to_string())?;

    let conta_atualizada = tx.query_row(
        "SELECT id, cliente_id, cliente_nome_snapshot, venda_id, descricao, moeda_codigo,
                valor_original_minor, taxa_cambio_escala6, valor_original_principal_minor,
                data_emissao, data_vencimento, status, saldo_pendente_minor, criado_em,
                atualizado_em, usuario_id, observacao 
         FROM contas_receber WHERE id = ?1",
        [&dto.conta_receber_id],
        |row| {
            Ok(ContaReceberResp {
                id: row.get(0)?,
                cliente_id: row.get(1)?,
                cliente_nome_snapshot: row.get(2)?,
                venda_id: row.get(3)?,
                descricao: row.get(4)?,
                moeda_codigo: row.get(5)?,
                valor_original_minor: row.get(6)?,
                taxa_cambio_escala6: row.get(7)?,
                valor_original_principal_minor: row.get(8)?,
                data_emissao: row.get(9)?,
                data_vencimento: row.get(10)?,
                status: row.get(11)?,
                saldo_pendente_minor: row.get(12)?,
                criado_em: row.get(13)?,
                atualizado_em: row.get(14)?,
                usuario_id: row.get(15)?,
                observacao: row.get(16)?,
            })
        }
    ).map_err(|e| e.to_string())?;

    inserir_outbox(
        &tx,
        "CONTA_RECEBER_CANCELADA",
        json!({
            "conta_receber_id": dto.conta_receber_id,
            "cancelado_em": agora,
            "motivo": dto.motivo,
            "usuario_id": dto.usuario_id
        }),
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(RespostaBase::ok("Conta a receber cancelada com sucesso", conta_atualizada))
}

