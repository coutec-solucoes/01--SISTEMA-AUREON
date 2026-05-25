use tauri::State;
use tracing::{info, error};
use uuid::Uuid;
use serde_json::json;

use aureon_core::{dtos::*, RespostaBase};
use crate::estado::EstadoApp;
use crate::commands_caixa::outbox_inserir;
use crate::commands_estoque::processar_baixa_venda;

// ================================================================
// Command: registrar_pagamento
// Valores em minor unit (i64). Taxa escalada em 1_000_000.
// Snapshot da data/cotacao travado no momento do registro.
// ================================================================

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
        valor_minor = dto.valor_informado_minor,
        "Chamada: registrar_pagamento"
    );

    if dto.valor_informado_minor <= 0 {
        return Err("Valor do pagamento deve ser maior que zero.".into());
    }

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    // GUARDA OPERACIONAL DE LICENCA - Bloqueio de registro de pagamento
    crate::commands_licenciamento::garantir_operacao_licenciada(&conn, "FECHAR_VENDA_PAGAMENTO", Some(&dto.venda_id), None)?;

    let venda_ok: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM vendas WHERE id = ?1 AND status = 'EM_ANDAMENTO'",
        rusqlite::params![&dto.venda_id],
        |row| row.get(0),
    ).unwrap_or(false);

    if !venda_ok {
        return Err("Venda nao encontrada ou nao esta em andamento.".into());
    }

    if dto.forma_pagamento == "CREDITO_CLIENTE" {
        let cliente_id: Option<String> = conn.query_row(
            "SELECT cliente_id FROM vendas WHERE id = ?1",
            rusqlite::params![&dto.venda_id],
            |row| row.get(0),
        ).unwrap_or(None);

        if cliente_id.is_none() {
            return Err("Para usar Crédito Cliente (crediário), é necessário associar um cliente à venda.".into());
        }
    }

    // Buscar moeda principal
    let (moeda_principal, _data_cot_principal): (String, String) = conn.query_row(
        "SELECT codigo, COALESCE(MAX(data_cotacao), datetime('now')) FROM moedas_cache
         WHERE principal = 1 LIMIT 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).unwrap_or_else(|_| ("BRL".to_string(), chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()));

    // Calcular taxa escalada (1_000_000 = taxa 1.0)
    let (taxa_cambio_escala6, data_cotacao_usada): (i64, String) =
        if dto.moeda_codigo.to_uppercase() == moeda_principal.to_uppercase() {
            (1_000_000_i64, chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string())
        } else {
            // Buscar cotacao mais recente: taxa em f64, converter para escala6
            conn.query_row(
                "SELECT taxa, data_cotacao FROM cotacoes_cache
                 WHERE UPPER(moeda_origem) = UPPER(?1) AND UPPER(moeda_destino) = UPPER(?2)
                 ORDER BY data_cotacao DESC LIMIT 1",
                rusqlite::params![&dto.moeda_codigo, &moeda_principal],
                |row| {
                    let taxa_f64: f64 = row.get(0)?;
                    let data: String = row.get(1)?;
                    // Escala 1_000_000 — sem f64 nas operacoes financeiras apos este ponto
                    let taxa_escala6 = (taxa_f64 * 1_000_000.0).round() as i64;
                    Ok((taxa_escala6, data))
                },
            ).map_err(|_| format!(
                "Cotacao nao encontrada para {} -> {}. Atualize as cotacoes.",
                dto.moeda_codigo, moeda_principal
            ))?
        };

    // Conversao inteira: valor_informado * taxa_escala6 / 1_000_000
    let valor_convertido_minor = dto.valor_informado_minor
        .checked_mul(taxa_cambio_escala6)
        .ok_or("Overflow no calculo de conversao")?
        / 1_000_000;

    let agora = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let pag_id = Uuid::new_v4().to_string();
    let moeda_troco = dto.moeda_troco_codigo.clone()
        .unwrap_or_else(|| moeda_principal.clone());

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO venda_pagamentos (id, venda_id, forma_pagamento, moeda_codigo,
                                       valor_informado_minor, moeda_principal_codigo,
                                       valor_convertido_minor, taxa_cambio_escala6,
                                       data_cotacao_usada, troco_minor, moeda_troco_codigo,
                                       criado_em)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, ?11)",
        rusqlite::params![
            &pag_id,
            &dto.venda_id,
            &dto.forma_pagamento,
            &dto.moeda_codigo,
            dto.valor_informado_minor,
            &moeda_principal,
            valor_convertido_minor,
            taxa_cambio_escala6,
            &data_cotacao_usada,  // snapshot travado
            &moeda_troco,
            &agora
        ],
    ).map_err(|e| {
        error!(componente = "aureon-pdv::commands_pagamento", erro = %e, "Erro ao inserir pagamento");
        e.to_string()
    })?;

    // Evento outbox: PAGAMENTO_REGISTRADO
    outbox_inserir(
        &tx,
        "PAGAMENTO_REGISTRADO",
        json!({
            "pagamento_id": &pag_id,
            "venda_id": &dto.venda_id,
            "forma_pagamento": &dto.forma_pagamento,
            "moeda_codigo": &dto.moeda_codigo,
            "valor_informado_minor": dto.valor_informado_minor,
            "valor_convertido_minor": valor_convertido_minor,
            "taxa_cambio_escala6": taxa_cambio_escala6,
            "data_cotacao_usada": &data_cotacao_usada,
            "criado_em": &agora
        }),
    )?;

    tx.commit().map_err(|e| e.to_string())?;

    let resp = PagamentoResp {
        id: pag_id,
        venda_id: dto.venda_id,
        forma_pagamento: dto.forma_pagamento,
        moeda_codigo: dto.moeda_codigo,
        valor_informado_minor: dto.valor_informado_minor,
        moeda_principal_codigo: moeda_principal,
        valor_convertido_minor,
        taxa_cambio_escala6,
        data_cotacao_usada,
        troco_minor: 0, // calculado separadamente por calcular_troco
        moeda_troco_codigo: Some(moeda_troco),
        criado_em: agora,
    };

    Ok(RespostaBase::ok("Pagamento registrado", resp))
}

// ================================================================
// Command: calcular_troco
// Aritmética inteira pura — sem f64
// ================================================================

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

    let total_venda_minor: i64 = conn.query_row(
        "SELECT total_minor FROM vendas WHERE id = ?1",
        rusqlite::params![&venda_id],
        |row| row.get(0),
    ).map_err(|_| "Venda nao encontrada.".to_string())?;

    let total_pago_minor: i64 = conn.query_row(
        "SELECT COALESCE(SUM(valor_convertido_minor), 0) FROM venda_pagamentos WHERE venda_id = ?1",
        rusqlite::params![&venda_id],
        |row| row.get(0),
    ).unwrap_or(0);

    let troco_minor = (total_pago_minor - total_venda_minor).max(0);

    Ok(RespostaBase::ok("Troco calculado", TrocoResp {
        total_venda_minor,
        total_pago_minor,
        troco_minor,
        quitado: total_pago_minor >= total_venda_minor,
    }))
}

// ================================================================
// Command: finalizar_venda
// numero_venda atribuido aqui — transacao atomica com incremento
// ================================================================

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

    // GUARDA OPERACIONAL DE LICENCA - Bloqueio de finalizacao de venda
    crate::commands_licenciamento::garantir_operacao_licenciada(&conn, "FINALIZAR_VENDA", Some(&venda_id), None)?;

    // Verificar venda EM_ANDAMENTO e buscar total e contagem de itens
    let (total_venda_minor, total_itens, usuario_id): (i64, i64, String) = conn.query_row(
        "SELECT v.total_minor, COUNT(CASE WHEN vi.cancelado = 0 THEN 1 END), v.usuario_id
         FROM vendas v
         LEFT JOIN venda_itens vi ON vi.venda_id = v.id
         WHERE v.id = ?1 AND v.status = 'EM_ANDAMENTO'
         GROUP BY v.id",
        rusqlite::params![&venda_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).map_err(|_| "Venda nao encontrada ou nao esta em andamento.".to_string())?;

    if total_itens == 0 {
        return Err("Nao e possivel finalizar uma venda sem itens ativos.".into());
    }

    // Verificar cobertura do pagamento — aritmética inteira
    let total_pago_minor: i64 = conn.query_row(
        "SELECT COALESCE(SUM(valor_convertido_minor), 0) FROM venda_pagamentos WHERE venda_id = ?1",
        rusqlite::params![&venda_id],
        |row| row.get(0),
    ).unwrap_or(0);

    if total_pago_minor < total_venda_minor {
        return Err(format!(
            "Pagamento insuficiente. Total: {} | Pago: {} | Faltam: {} (em minor unit)",
            total_venda_minor,
            total_pago_minor,
            total_venda_minor - total_pago_minor
        ));
    }

    let agora = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Transacao atomica: buscar+incrementar numero_venda + marcar FINALIZADA
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let numero_venda: i64 = tx.query_row(
        "SELECT proximo_numero FROM controle_numeracao WHERE id = 1",
        [],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE controle_numeracao SET proximo_numero = proximo_numero + 1, atualizado_em = ?1
         WHERE id = 1",
        rusqlite::params![&agora],
    ).map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE vendas
         SET status = 'FINALIZADA',
             numero_venda = ?1,
             finalizado_em = ?2,
             atualizado_em = ?2
         WHERE id = ?3",
        rusqlite::params![numero_venda, &agora, &venda_id],
    ).map_err(|e| e.to_string())?;

    // Calcular e gravar troco no pagamento mais recente (simplificado)
    let troco_minor = total_pago_minor - total_venda_minor;
    if troco_minor > 0 {
        tx.execute(
            "UPDATE venda_pagamentos SET troco_minor = ?1
             WHERE id = (SELECT id FROM venda_pagamentos WHERE venda_id = ?2
                         ORDER BY criado_em DESC LIMIT 1)",
            rusqlite::params![troco_minor, &venda_id],
        ).map_err(|e| e.to_string())?;
    }

    // Evento outbox: VENDA_FINALIZADA
    outbox_inserir(
        &tx,
        "VENDA_FINALIZADA",
        json!({
            "venda_id": &venda_id,
            "numero_venda": numero_venda,
            "total_minor": total_venda_minor,
            "total_pago_minor": total_pago_minor,
            "troco_minor": troco_minor,
            "finalizado_em": &agora
        }),
    )?;

    // Ressalva 1: se venda originou de MESA ou COMANDA, fechar a origem
    let origem_gourmet: Option<(String, String)> = tx.query_row(
        "SELECT origem_tipo, origem_id FROM vendas WHERE id = ?1 AND origem_tipo IN ('MESA', 'COMANDA')",
        rusqlite::params![&venda_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).ok();

    if let Some((ref orig_tipo, ref orig_id)) = origem_gourmet {
        match orig_tipo.as_str() {
            "MESA" => {
                tx.execute(
                    "UPDATE mesas_operacionais SET status = 'FECHADA', fechada_em = ?1 WHERE id = ?2",
                    rusqlite::params![&agora, orig_id],
                ).map_err(|e| e.to_string())?;
            }
            "COMANDA" => {
                tx.execute(
                    "UPDATE comandas_operacionais SET status = 'FECHADA', fechada_em = ?1 WHERE id = ?2",
                    rusqlite::params![&agora, orig_id],
                ).map_err(|e| e.to_string())?;
            }
            _ => {}
        }

        outbox_inserir(
            &tx,
            "ORIGEM_GOURMET_FECHADA_POR_PAGAMENTO",
            json!({
                "venda_id": &venda_id,
                "numero_venda": numero_venda,
                "origem_tipo": orig_tipo,
                "origem_id": orig_id,
                "fechado_em": &agora
            }),
        )?;
    }

    // FASE 13 - FINANCEIRO: Gerar contas_receber se houver pagamento com CREDITO_CLIENTE
    let total_credito_cliente_minor: i64 = tx.query_row(
        "SELECT COALESCE(SUM(valor_convertido_minor), 0) 
         FROM venda_pagamentos 
         WHERE venda_id = ?1 AND forma_pagamento = 'CREDITO_CLIENTE'",
        rusqlite::params![&venda_id],
        |row| row.get(0)
    ).unwrap_or(0);

    if total_credito_cliente_minor > 0 {
        let (cliente_id, cliente_nome_snapshot): (Option<String>, Option<String>) = tx.query_row(
            "SELECT v.cliente_id, c.nome 
             FROM vendas v
             LEFT JOIN clientes_cache c ON c.id = v.cliente_id
             WHERE v.id = ?1",
            rusqlite::params![&venda_id],
            |row| Ok((row.get(0)?, row.get(1)?))
        ).map_err(|e| format!("Erro ao obter cliente para crediário: {e}"))?;

        if cliente_id.is_none() {
            return Err("Para finalizar uma venda com Crédito Cliente, é obrigatório associar um cliente.".into());
        }

        // Idempotência: verificar se já existe conta a receber para esta venda
        let existe_conta: bool = tx.query_row(
            "SELECT COUNT(*) > 0 FROM contas_receber WHERE venda_id = ?1",
            rusqlite::params![&venda_id],
            |row| row.get(0)
        ).unwrap_or(false);

        if !existe_conta {
            let (moeda_principal, _): (String, String) = tx.query_row(
                "SELECT codigo, COALESCE(MAX(data_cotacao), datetime('now')) FROM moedas_cache
                 WHERE principal = 1 LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            ).unwrap_or_else(|_| ("BRL".to_string(), "".to_string()));

            let rec_id = uuid::Uuid::new_v4().to_string();
            let data_emissao = agora.split(' ').next().unwrap_or("").to_string(); // YYYY-MM-DD
            let vencimento_dt = chrono::Utc::now() + chrono::Duration::days(30);
            let data_vencimento = vencimento_dt.format("%Y-%m-%d").to_string();
            let nome_cliente = cliente_nome_snapshot.unwrap_or_else(|| "Cliente Não Identificado".to_string());

            tx.execute(
                "INSERT INTO contas_receber (
                    id, cliente_id, cliente_nome_snapshot, venda_id, descricao, moeda_codigo,
                    valor_original_minor, taxa_cambio_escala6, valor_original_principal_minor,
                    data_emissao, data_vencimento, status, saldo_pendente_minor, criado_em,
                    atualizado_em, usuario_id, observacao
                ) VALUES (?1, ?2, ?3, ?4, 'Venda em crediário / fiado', ?5, ?6, 1000000, ?6, ?7, ?8, 'PENDENTE', ?6, ?9, ?9, ?10, NULL)",
                rusqlite::params![
                    rec_id,
                    cliente_id,
                    nome_cliente,
                    venda_id,
                    &moeda_principal,
                    total_credito_cliente_minor,
                    data_emissao,
                    data_vencimento,
                    &agora,
                    &usuario_id
                ]
            ).map_err(|e| format!("Erro ao gerar conta a receber: {e}"))?;

            outbox_inserir(
                &tx,
                "CONTA_RECEBER_CRIADA",
                serde_json::json!({
                    "id": rec_id,
                    "cliente_id": cliente_id,
                    "cliente_nome_snapshot": nome_cliente,
                    "venda_id": venda_id,
                    "descricao": "Venda em crediário / fiado",
                    "moeda_codigo": &moeda_principal,
                    "valor_original_minor": total_credito_cliente_minor,
                    "taxa_cambio_escala6": 1000000,
                    "valor_original_principal_minor": total_credito_cliente_minor,
                    "data_emissao": data_emissao,
                    "data_vencimento": data_vencimento,
                    "status": "PENDENTE",
                    "saldo_pendente_minor": total_credito_cliente_minor,
                    "criado_em": &agora,
                    "usuario_id": &usuario_id
                })
            ).map_err(|e| format!("Erro ao registrar outbox de conta a receber: {e}"))?;
        }
    }

    // FASE 11 - ESTOQUE: Baixar itens da venda (só ocorre na finalização)
    processar_baixa_venda(&tx, &venda_id, &usuario_id).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    // Retornar resumo atualizado com numero_venda preenchido
    let resp = conn.query_row(
        "SELECT v.id, v.numero_venda, v.status, v.tipo_venda,
                v.subtotal_minor, v.desconto_total_minor, v.acrescimo_total_minor, v.total_minor,
                COUNT(CASE WHEN vi.cancelado = 0 THEN 1 END) as total_itens
         FROM vendas v
         LEFT JOIN venda_itens vi ON vi.venda_id = v.id
         WHERE v.id = ?1 GROUP BY v.id",
        rusqlite::params![&venda_id],
        |row| Ok(VendaResumoResp {
            id:                    row.get(0)?,
            numero_venda:          row.get(1)?,
            status:                row.get(2)?,
            tipo_venda:            row.get(3)?,
            subtotal_minor:        row.get(4)?,
            desconto_total_minor:  row.get(5)?,
            acrescimo_total_minor: row.get(6)?,
            total_minor:           row.get(7)?,
            total_itens:           row.get(8)?,
        }),
    ).map_err(|e| e.to_string())?;

    Ok(RespostaBase::ok("Venda finalizada com sucesso", resp))
}

// ================================================================
// Command: listar_pagamentos_venda
// ================================================================

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
                valor_informado_minor, moeda_principal_codigo, valor_convertido_minor,
                taxa_cambio_escala6, data_cotacao_usada, troco_minor,
                moeda_troco_codigo, criado_em
         FROM venda_pagamentos WHERE venda_id = ?1 ORDER BY criado_em ASC"
    ).map_err(|e| e.to_string())?;

    let iter = stmt.query_map(rusqlite::params![&venda_id], |row| {
        Ok(PagamentoResp {
            id:                     row.get(0)?,
            venda_id:               row.get(1)?,
            forma_pagamento:        row.get(2)?,
            moeda_codigo:           row.get(3)?,
            valor_informado_minor:  row.get(4)?,
            moeda_principal_codigo: row.get(5)?,
            valor_convertido_minor: row.get(6)?,
            taxa_cambio_escala6:    row.get(7)?,
            data_cotacao_usada:     row.get(8)?,
            troco_minor:            row.get(9)?,
            moeda_troco_codigo:     row.get(10)?,
            criado_em:              row.get(11)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut pagamentos = Vec::new();
    for p in iter { if let Ok(val) = p { pagamentos.push(val); } }

    Ok(RespostaBase::ok("Pagamentos da venda", pagamentos))
}
