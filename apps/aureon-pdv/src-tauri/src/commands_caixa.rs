use tauri::State;
use tracing::{info, error};
use uuid::Uuid;
use serde_json::json;

use aureon_core::{dtos::*, RespostaBase};
use crate::estado::EstadoApp;

// ================================================================
// Funcoes auxiliares de escala monetaria
// ================================================================

/// Escala da minor unit de uma moeda (centavos BRL/USD = 100, PYG = 1, etc.)
fn escala_moeda(codigo: &str) -> i64 {
    match codigo.to_uppercase().as_str() {
        "PYG" | "JPY" | "KRW" => 1,  // sem centavos
        _ => 100,                      // BRL, USD, EUR e maioria
    }
}

/// Insere evento no sync_outbox local para envio futuro ao servidor.
pub fn inserir_outbox(
    conn: &rusqlite::Connection,
    event_type: &str,
    payload: serde_json::Value,
) -> Result<(), String> {
    let event_id = Uuid::new_v4().to_string();
    let idempotency_key = Uuid::new_v4().to_string();
    let payload_str = payload.to_string();
    let agora = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    conn.execute(
        "INSERT INTO sync_outbox (event_id, idempotency_key, event_type, schema_version,
                                  payload, status, tentativas, criado_em, atualizado_em)
         VALUES (?1, ?2, ?3, 1, ?4, 'PENDENTE', 0, ?5, ?5)",
        rusqlite::params![
            &event_id,
            &idempotency_key,
            event_type,
            &payload_str,
            &agora
        ],
    ).map_err(|e| format!("Falha ao inserir outbox [{event_type}]: {e}"))?;

    Ok(())
}

/// Carrega os saldos de uma sessao para o DTO de resposta.
fn carregar_saldos(
    conn: &rusqlite::Connection,
    sessao_id: &str,
) -> Vec<SaldoMoedaResp> {
    let mut stmt = match conn.prepare(
        "SELECT moeda_codigo, valor_abertura_minor, valor_fechamento_informado_minor,
                valor_esperado_minor, diferenca_minor
         FROM sessoes_caixa_moedas WHERE sessao_id = ?1 ORDER BY moeda_codigo"
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let iter = match stmt.query_map(rusqlite::params![sessao_id], |row| {
        Ok(SaldoMoedaResp {
            moeda_codigo:                     row.get(0)?,
            valor_abertura_minor:             row.get(1)?,
            valor_fechamento_informado_minor: row.get(2)?,
            valor_esperado_minor:             row.get(3)?,
            diferenca_minor:                  row.get(4)?,
        })
    }) {
        Ok(i) => i,
        Err(_) => return vec![],
    };

    let mut saldos = Vec::new();
    for s in iter {
        if let Ok(val) = s { saldos.push(val); }
    }
    saldos
}

// ================================================================
// Command: abrir_caixa
// ================================================================

#[tauri::command]
pub async fn abrir_caixa(
    dto: AbrirCaixaReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<SessaoCaixaResp>, String> {
    info!(
        componente = "aureon-pdv::commands_caixa",
        registradora_id = %dto.registradora_id,
        usuario_id = %dto.usuario_id,
        "Chamada: abrir_caixa"
    );

    if dto.saldos_abertura.is_empty() {
        return Err("E obrigatorio informar ao menos um saldo de abertura com moeda.".into());
    }

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    // GUARDA OPERACIONAL DE LICENCA - Bloqueio de abertura de caixa
    crate::commands_licenciamento::garantir_operacao_licenciada(&conn, "ABRIR_CAIXA", Some(&dto.registradora_id), None)?;

    // GUARDA DE PERMISSÃO OPERACIONAL
    crate::commands_seguranca::garantir_permissao_usuario(&conn, "CAIXA_ABRIR", Some(&dto.registradora_id), None)?;

    // Bloquear se ja houver caixa ABERTO nesta registradora
    let sessao_aberta: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM sessoes_caixa WHERE registradora_id = ?1 AND status = 'ABERTO'",
        rusqlite::params![&dto.registradora_id],
        |row| row.get(0),
    ).unwrap_or(false);

    if sessao_aberta {
        return Err("Ja existe uma sessao de caixa aberta para esta registradora.".into());
    }

    let sessao_id = Uuid::new_v4().to_string();
    let agora = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO sessoes_caixa (id, registradora_id, usuario_id, status, aberto_em)
         VALUES (?1, ?2, ?3, 'ABERTO', ?4)",
        rusqlite::params![&sessao_id, &dto.registradora_id, &dto.usuario_id, &agora],
    ).map_err(|e| {
        error!(componente = "aureon-pdv::commands_caixa", erro = %e, "Erro ao abrir caixa");
        e.to_string()
    })?;

    // Inserir saldos por moeda
    for saldo in &dto.saldos_abertura {
        if saldo.valor_minor < 0 {
            return Err(format!("Saldo de abertura nao pode ser negativo: {}", saldo.moeda_codigo));
        }
        let saldo_id = Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO sessoes_caixa_moedas (id, sessao_id, moeda_codigo, valor_abertura_minor)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                &saldo_id,
                &sessao_id,
                &saldo.moeda_codigo,
                saldo.valor_minor
            ],
        ).map_err(|e| e.to_string())?;
    }

    // Evento outbox: CAIXA_ABERTO
    inserir_outbox(
        &tx,
        "CAIXA_ABERTO",
        json!({
            "sessao_id": &sessao_id,
            "registradora_id": &dto.registradora_id,
            "usuario_id": &dto.usuario_id,
            "aberto_em": &agora,
            "saldos": dto.saldos_abertura.iter().map(|s| json!({
                "moeda_codigo": &s.moeda_codigo,
                "valor_minor": s.valor_minor
            })).collect::<Vec<_>>()
        }),
    )?;

    tx.commit().map_err(|e| e.to_string())?;

    let saldos = carregar_saldos(&conn, &sessao_id);

    let resp = SessaoCaixaResp {
        id: sessao_id,
        registradora_id: dto.registradora_id,
        usuario_id: dto.usuario_id,
        status: "ABERTO".into(),
        aberto_em: agora,
        fechado_em: None,
        observacao: None,
        saldos,
    };

    Ok(RespostaBase::ok("Caixa aberto com sucesso", resp))
}

// ================================================================
// Command: fechar_caixa
// ================================================================

#[tauri::command]
pub async fn fechar_caixa(
    dto: FecharCaixaReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<SessaoCaixaResp>, String> {
    info!(
        componente = "aureon-pdv::commands_caixa",
        sessao_id = %dto.sessao_id,
        "Chamada: fechar_caixa"
    );

    if dto.saldos_fechamento.is_empty() {
        return Err("E obrigatorio informar saldo de fechamento por moeda.".into());
    }

    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    // GUARDA DE PERMISSÃO OPERACIONAL
    crate::commands_seguranca::garantir_permissao_usuario(&conn, "CAIXA_FECHAR", Some(&dto.sessao_id), None)?;

    // Verificar sessao ativa
    let (registradora_id, usuario_id): (String, String) = conn.query_row(
        "SELECT registradora_id, usuario_id FROM sessoes_caixa WHERE id = ?1 AND status = 'ABERTO'",
        rusqlite::params![&dto.sessao_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).map_err(|_| "Sessao de caixa nao encontrada ou ja fechada.".to_string())?;

    let agora = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // Para cada moeda, apurar pagamentos recebidos e calcular diferenca
    for saldo_inf in &dto.saldos_fechamento {
        let valor_abertura: i64 = tx.query_row(
            "SELECT COALESCE(valor_abertura_minor, 0) FROM sessoes_caixa_moedas WHERE sessao_id = ?1 AND moeda_codigo = ?2",
            rusqlite::params![&dto.sessao_id, &saldo_inf.moeda_codigo],
            |row| row.get(0),
        ).unwrap_or(0);

        let valor_vendas: i64 = tx.query_row(
            "SELECT COALESCE(SUM(valor_informado_minor), 0) FROM venda_pagamentos vp
             INNER JOIN vendas v ON v.id = vp.venda_id
             INNER JOIN sessoes_caixa sc ON sc.id = v.sessao_caixa_id
             WHERE sc.id = ?1 AND vp.moeda_codigo = ?2 AND v.status = 'FINALIZADA'
               AND vp.forma_pagamento <> 'CREDITO_CLIENTE'",
            rusqlite::params![&dto.sessao_id, &saldo_inf.moeda_codigo],
            |row| row.get(0),
        ).unwrap_or(0);

        let valor_suprimentos: i64 = tx.query_row(
            "SELECT COALESCE(SUM(valor_minor), 0) FROM caixa_movimentacoes
             WHERE sessao_caixa_id = ?1 AND moeda_codigo = ?2 AND cancelado = 0 AND tipo_movimentacao = 'SUPRIMENTO'",
            rusqlite::params![&dto.sessao_id, &saldo_inf.moeda_codigo],
            |row| row.get(0),
        ).unwrap_or(0);

        let valor_sangrias: i64 = tx.query_row(
            "SELECT COALESCE(SUM(valor_minor), 0) FROM caixa_movimentacoes
             WHERE sessao_caixa_id = ?1 AND moeda_codigo = ?2 AND cancelado = 0 AND tipo_movimentacao IN ('SANGRIA', 'VALE_FUNCIONARIO')",
            rusqlite::params![&dto.sessao_id, &saldo_inf.moeda_codigo],
            |row| row.get(0),
        ).unwrap_or(0);

        // FASE 13 - FINANCEIRO: Integrar recebimentos/pagamentos financeiros no fechamento
        let valor_recebimentos_fin: i64 = tx.query_row(
            "SELECT COALESCE(SUM(valor_informado_minor), 0) FROM financeiro_lancamentos
             WHERE sessao_caixa_id = ?1 AND moeda_codigo = ?2 AND tipo_lancamento = 'RECEBIMENTO'",
            rusqlite::params![&dto.sessao_id, &saldo_inf.moeda_codigo],
            |row| row.get(0),
        ).unwrap_or(0);

        let valor_pagamentos_fin: i64 = tx.query_row(
            "SELECT COALESCE(SUM(valor_informado_minor), 0) FROM financeiro_lancamentos
             WHERE sessao_caixa_id = ?1 AND moeda_codigo = ?2 AND tipo_lancamento = 'PAGAMENTO'",
            rusqlite::params![&dto.sessao_id, &saldo_inf.moeda_codigo],
            |row| row.get(0),
        ).unwrap_or(0);

        let valor_esperado = valor_abertura + valor_vendas + valor_suprimentos - valor_sangrias + valor_recebimentos_fin - valor_pagamentos_fin;
        let diferenca = saldo_inf.valor_minor - valor_esperado;

        tx.execute(
            "UPDATE sessoes_caixa_moedas
             SET valor_fechamento_informado_minor = ?1,
                 valor_esperado_minor = ?2,
                 diferenca_minor = ?3
             WHERE sessao_id = ?4 AND moeda_codigo = ?5",
            rusqlite::params![
                saldo_inf.valor_minor,
                valor_esperado,
                diferenca,
                &dto.sessao_id,
                &saldo_inf.moeda_codigo
            ],
        ).map_err(|e| e.to_string())?;
    }

    tx.execute(
        "UPDATE sessoes_caixa SET status = 'FECHADO', fechado_em = ?1, observacao = ?2
         WHERE id = ?3",
        rusqlite::params![&agora, &dto.observacao, &dto.sessao_id],
    ).map_err(|e| e.to_string())?;

    // Evento outbox: CAIXA_FECHADO
    inserir_outbox(
        &tx,
        "CAIXA_FECHADO",
        json!({
            "sessao_id": &dto.sessao_id,
            "fechado_em": &agora,
            "saldos": dto.saldos_fechamento.iter().map(|s| json!({
                "moeda_codigo": &s.moeda_codigo,
                "valor_minor": s.valor_minor
            })).collect::<Vec<_>>()
        }),
    )?;

    tx.commit().map_err(|e| e.to_string())?;

    let saldos = carregar_saldos(&conn, &dto.sessao_id);

    let resp = SessaoCaixaResp {
        id: dto.sessao_id,
        registradora_id,
        usuario_id,
        status: "FECHADO".into(),
        aberto_em: String::new(), // carregado do banco se necessario
        fechado_em: Some(agora),
        observacao: dto.observacao,
        saldos,
    };

    Ok(RespostaBase::ok("Caixa fechado com sucesso", resp))
}

// ================================================================
// Command: obter_sessao_ativa
// ================================================================

#[tauri::command]
pub async fn obter_sessao_ativa(
    registradora_id: String,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<Option<SessaoCaixaResp>>, String> {
    info!(
        componente = "aureon-pdv::commands_caixa",
        registradora_id = %registradora_id,
        "Chamada: obter_sessao_ativa"
    );

    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let sessao = conn.query_row(
        "SELECT id, registradora_id, usuario_id, status, aberto_em, fechado_em, observacao
         FROM sessoes_caixa WHERE registradora_id = ?1 AND status = 'ABERTO' LIMIT 1",
        rusqlite::params![&registradora_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
            ))
        },
    ).ok();

    let resp = sessao.map(|(id, reg_id, usr_id, status, aberto_em, fechado_em, obs)| {
        let saldos = carregar_saldos(&conn, &id);
        SessaoCaixaResp {
            id,
            registradora_id: reg_id,
            usuario_id: usr_id,
            status,
            aberto_em,
            fechado_em,
            observacao: obs,
            saldos,
        }
    });

    Ok(RespostaBase::ok("Sessao ativa", resp))
}

// ================================================================
// Command: listar_sessoes
// ================================================================

#[tauri::command]
pub async fn listar_sessoes(
    limite: u32,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<Vec<SessaoCaixaResp>>, String> {
    info!(componente = "aureon-pdv::commands_caixa", limite, "Chamada: listar_sessoes");

    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let limite = (limite.min(200)) as i64;

    let mut stmt = conn.prepare(
        "SELECT id, registradora_id, usuario_id, status, aberto_em, fechado_em, observacao
         FROM sessoes_caixa ORDER BY aberto_em DESC LIMIT ?1"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(rusqlite::params![limite], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, Option<String>>(6)?,
        ))
    }).map_err(|e| e.to_string())?;

    let mut sessoes = Vec::new();
    for row in rows {
        if let Ok((id, reg_id, usr_id, status, aberto_em, fechado_em, obs)) = row {
            let saldos = carregar_saldos(&conn, &id);
            sessoes.push(SessaoCaixaResp {
                id,
                registradora_id: reg_id,
                usuario_id: usr_id,
                status,
                aberto_em,
                fechado_em,
                observacao: obs,
                saldos,
            });
        }
    }

    Ok(RespostaBase::ok("Sessoes listadas", sessoes))
}

// Exporta a funcao auxiliar para outros modulos usarem
pub use self::inserir_outbox as outbox_inserir;
