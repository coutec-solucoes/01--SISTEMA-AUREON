use tauri::{command, State};
use chrono::Utc;
use aureon_core::{dtos::*, RespostaBase};
use crate::estado::EstadoApp;

// Auxiliar para obter e aplicar datas padrão (últimos 30 dias se vazio)
fn obter_datas_periodo(data_inicio: &Option<String>, data_fim: &Option<String>) -> (String, String) {
    let inicio = data_inicio.clone().filter(|s| !s.is_empty()).unwrap_or_else(|| {
        (Utc::now() - chrono::Duration::days(30)).format("%Y-%m-%d 00:00:00").to_string()
    });
    let fim = data_fim.clone().filter(|s| !s.is_empty()).unwrap_or_else(|| {
        (Utc::now() + chrono::Duration::days(1)).format("%Y-%m-%d 23:59:59").to_string()
    });
    (inicio, fim)
}

#[command]
pub async fn obter_indicadores_dashboard(
    estado: State<'_, EstadoApp>,
    data_inicio: Option<String>,
    data_fim: Option<String>,
) -> Result<RespostaBase<IndicadoresDashboardResp>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let (d_inicio, d_fim) = obter_datas_periodo(&data_inicio, &data_fim);
    let hoje = Utc::now().format("%Y-%m-%d").to_string();

    // 1. Faturamento por moeda (vendas finalizadas)
    let mut stmt_fat = conn.prepare(
        "SELECT vp.moeda_codigo, SUM(vp.valor_informado_minor - CASE WHEN vp.moeda_codigo = COALESCE(vp.moeda_troco_codigo, '') THEN vp.troco_minor ELSE 0 END)
         FROM venda_pagamentos vp
         INNER JOIN vendas v ON v.id = vp.venda_id
         WHERE v.status = 'FINALIZADA' AND v.criado_em >= ?1 AND v.criado_em <= ?2
         GROUP BY vp.moeda_codigo"
    ).map_err(|e| e.to_string())?;
    let faturamento_por_moeda = stmt_fat.query_map([&d_inicio, &d_fim], |row| {
        Ok(TotalPorMoeda {
            moeda_codigo: row.get(0)?,
            valor_minor: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    // 2. Despesas por moeda (pagamentos / lançamentos de saída)
    let mut stmt_desp = conn.prepare(
        "SELECT fl.moeda_codigo, SUM(fl.valor_informado_minor)
         FROM financeiro_lancamentos fl
         WHERE fl.tipo_lancamento = 'PAGAMENTO' AND fl.criado_em >= ?1 AND fl.criado_em <= ?2
         GROUP BY fl.moeda_codigo"
    ).map_err(|e| e.to_string())?;
    let despesas_por_moeda = stmt_desp.query_map([&d_inicio, &d_fim], |row| {
        Ok(TotalPorMoeda {
            moeda_codigo: row.get(0)?,
            valor_minor: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    // 3. Total vendas quantidade
    let total_vendas_quantidade: i64 = conn.query_row(
        "SELECT COUNT(*) FROM vendas WHERE status = 'FINALIZADA' AND criado_em >= ?1 AND criado_em <= ?2",
        [&d_inicio, &d_fim],
        |row| row.get(0)
    ).unwrap_or(0);

    // 4. Total vendas itens quantidade (escala 3)
    let total_vendas_itens_quantidade_escala3: i64 = conn.query_row(
        "SELECT COALESCE(SUM(vi.quantidade_escala3), 0)
         FROM venda_itens vi
         INNER JOIN vendas v ON v.id = vi.venda_id
         WHERE v.status = 'FINALIZADA' AND vi.cancelado = 0 AND v.criado_em >= ?1 AND v.criado_em <= ?2",
        [&d_inicio, &d_fim],
        |row| row.get(0)
    ).unwrap_or(0);

    // 5. Produtos estoque crítico (ativos, que controlam estoque e saldo <= 0)
    let produtos_estoque_critico: i64 = conn.query_row(
        "SELECT COUNT(*) FROM produtos_cache pc
         LEFT JOIN produtos_estoque_cache pec ON pc.produto_id = pec.produto_id
         WHERE pc.controla_estoque = 1 AND pc.ativo = 1 AND COALESCE(pec.quantidade_escala3, 0) <= 0",
        [],
        |row| row.get(0)
    ).unwrap_or(0);

    // 6. Contas a pagar vencidas
    let mut stmt_cp_venc = conn.prepare(
        "SELECT moeda_codigo, SUM(saldo_pendente_minor)
         FROM contas_pagar
         WHERE status IN ('PENDENTE', 'PAGO_PARCIAL') AND data_vencimento < ?1
         GROUP BY moeda_codigo"
    ).map_err(|e| e.to_string())?;
    let contas_pagar_vencidas_por_moeda = stmt_cp_venc.query_map([&hoje], |row| {
        Ok(TotalPorMoeda {
            moeda_codigo: row.get(0)?,
            valor_minor: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    // 7. Contas a pagar a vencer
    let mut stmt_cp_aven = conn.prepare(
        "SELECT moeda_codigo, SUM(saldo_pendente_minor)
         FROM contas_pagar
         WHERE status IN ('PENDENTE', 'PAGO_PARCIAL') AND data_vencimento >= ?1
         GROUP BY moeda_codigo"
    ).map_err(|e| e.to_string())?;
    let contas_pagar_a_vencer_por_moeda = stmt_cp_aven.query_map([&hoje], |row| {
        Ok(TotalPorMoeda {
            moeda_codigo: row.get(0)?,
            valor_minor: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    // 8. Contas a receber vencidas
    let mut stmt_cr_venc = conn.prepare(
        "SELECT moeda_codigo, SUM(saldo_pendente_minor)
         FROM contas_receber
         WHERE status IN ('PENDENTE', 'PAGO_PARCIAL') AND data_vencimento < ?1
         GROUP BY moeda_codigo"
    ).map_err(|e| e.to_string())?;
    let contas_receber_vencidas_por_moeda = stmt_cr_venc.query_map([&hoje], |row| {
        Ok(TotalPorMoeda {
            moeda_codigo: row.get(0)?,
            valor_minor: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    // 9. Contas a receber a vencer
    let mut stmt_cr_aven = conn.prepare(
        "SELECT moeda_codigo, SUM(saldo_pendente_minor)
         FROM contas_receber
         WHERE status IN ('PENDENTE', 'PAGO_PARCIAL') AND data_vencimento >= ?1
         GROUP BY moeda_codigo"
    ).map_err(|e| e.to_string())?;
    let contas_receber_a_vencer_por_moeda = stmt_cr_aven.query_map([&hoje], |row| {
        Ok(TotalPorMoeda {
            moeda_codigo: row.get(0)?,
            valor_minor: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(RespostaBase::ok("Indicadores do dashboard obtidos com sucesso", IndicadoresDashboardResp {
        faturamento_por_moeda,
        despesas_por_moeda,
        total_vendas_quantidade,
        total_vendas_itens_quantidade_escala3,
        produtos_estoque_critico,
        contas_pagar_vencidas_por_moeda,
        contas_pagar_a_vencer_por_moeda,
        contas_receber_vencidas_por_moeda,
        contas_receber_a_vencer_por_moeda,
    }))
}

#[command]
pub async fn gerar_relatorio_vendas(
    estado: State<'_, EstadoApp>,
    filtros: FiltrosRelatorio,
) -> Result<RespostaBase<RelatorioVendasResp>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let (d_inicio, d_fim) = obter_datas_periodo(&filtros.data_inicio, &filtros.data_fim);

    let mut conds = vec!["v.criado_em >= ?1".to_string(), "v.criado_em <= ?2".to_string()];
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![
        Box::new(d_inicio.clone()),
        Box::new(d_fim.clone()),
    ];

    if let Some(ref u_id) = filtros.usuario_id {
        if !u_id.is_empty() {
            let idx = params_vec.len() + 1;
            conds.push(format!("v.usuario_id = ?{}", idx));
            params_vec.push(Box::new(u_id.clone()));
        }
    }

    if let Some(ref s_id) = filtros.sessao_caixa_id {
        if !s_id.is_empty() {
            let idx = params_vec.len() + 1;
            conds.push(format!("v.sessao_caixa_id = ?{}", idx));
            params_vec.push(Box::new(s_id.clone()));
        }
    }

    if let Some(ref m_cod) = filtros.moeda_codigo {
        if !m_cod.is_empty() {
            let idx = params_vec.len() + 1;
            conds.push(format!(
                "EXISTS (SELECT 1 FROM venda_pagamentos vp WHERE vp.venda_id = v.id AND vp.moeda_codigo = ?{})",
                idx
            ));
            params_vec.push(Box::new(m_cod.clone()));
        }
    }

    if let Some(ref f_pag) = filtros.forma_pagamento {
        if !f_pag.is_empty() {
            let idx = params_vec.len() + 1;
            conds.push(format!(
                "EXISTS (SELECT 1 FROM venda_pagamentos vp WHERE vp.venda_id = v.id AND vp.forma_pagamento = ?{})",
                idx
            ));
            params_vec.push(Box::new(f_pag.clone()));
        }
    }

    let where_clause = conds.join(" AND ");

    // Query 1: Vendas detalhadas
    let sql_vendas = format!(
        "SELECT v.id, v.numero_venda, v.criado_em, v.subtotal_minor, v.desconto_total_minor, v.acrescimo_total_minor, v.total_minor, v.status, c.nome, v.usuario_id
         FROM vendas v
         LEFT JOIN clientes_cache c ON v.cliente_id = c.id
         WHERE {}
         ORDER BY v.criado_em DESC
         LIMIT 1000",
        where_clause
    );

    let mut stmt_vendas = conn.prepare(&sql_vendas).map_err(|e| e.to_string())?;
    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

    let vendas = stmt_vendas.query_map(&*params_refs, |row| {
        Ok(RelatorioVendasItem {
            id: row.get(0)?,
            numero_venda: row.get(1)?,
            data_venda: row.get(2)?,
            total_bruto_minor: row.get(3)?,
            desconto_total_minor: row.get(4)?,
            acrescimo_total_minor: row.get(5)?,
            total_liquido_minor: row.get(6)?,
            status: row.get(7)?,
            cliente_nome: row.get(8)?,
            usuario_id: row.get(9)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    // Query 2: Totais por moeda (somente vendas finalizadas)
    let sql_totais = format!(
        "SELECT vp.moeda_codigo, SUM(vp.valor_informado_minor - CASE WHEN vp.moeda_codigo = COALESCE(vp.moeda_troco_codigo, '') THEN vp.troco_minor ELSE 0 END)
         FROM venda_pagamentos vp
         INNER JOIN vendas v ON v.id = vp.venda_id
         WHERE v.status = 'FINALIZADA' AND {}
         GROUP BY vp.moeda_codigo",
        where_clause
    );

    let mut stmt_totais = conn.prepare(&sql_totais).map_err(|e| e.to_string())?;
    let totais_por_moeda = stmt_totais.query_map(&*params_refs, |row| {
        Ok(TotalPorMoeda {
            moeda_codigo: row.get(0)?,
            valor_minor: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    // Query 3: Vendas agrupadas por forma de pagamento
    let sql_formas = format!(
        "SELECT vp.forma_pagamento, vp.moeda_codigo,
                SUM(vp.valor_informado_minor - CASE WHEN vp.moeda_codigo = COALESCE(vp.moeda_troco_codigo, '') THEN vp.troco_minor ELSE 0 END),
                COUNT(DISTINCT v.id)
         FROM venda_pagamentos vp
         INNER JOIN vendas v ON v.id = vp.venda_id
         WHERE v.status = 'FINALIZADA' AND {}
         GROUP BY vp.forma_pagamento, vp.moeda_codigo",
        where_clause
    );

    let mut stmt_formas = conn.prepare(&sql_formas).map_err(|e| e.to_string())?;
    let vendas_por_forma = stmt_formas.query_map(&*params_refs, |row| {
        Ok(VendasPorFormaPagamento {
            forma_pagamento: row.get(0)?,
            moeda_codigo: row.get(1)?,
            total_minor: row.get(2)?,
            quantidade: row.get(3)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(RespostaBase::ok("Relatório de vendas gerado com sucesso", RelatorioVendasResp {
        vendas,
        totais_por_moeda,
        vendas_por_forma,
    }))
}

#[command]
pub async fn gerar_relatorio_caixa(
    estado: State<'_, EstadoApp>,
    filtros: FiltrosRelatorio,
) -> Result<RespostaBase<RelatorioCaixaResp>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let (d_inicio, d_fim) = obter_datas_periodo(&filtros.data_inicio, &filtros.data_fim);

    let mut conds = vec!["s.aberto_em >= ?1".to_string(), "s.aberto_em <= ?2".to_string()];
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![
        Box::new(d_inicio.clone()),
        Box::new(d_fim.clone()),
    ];

    if let Some(ref u_id) = filtros.usuario_id {
        if !u_id.is_empty() {
            let idx = params_vec.len() + 1;
            conds.push(format!("s.usuario_id = ?{}", idx));
            params_vec.push(Box::new(u_id.clone()));
        }
    }

    let where_clause = conds.join(" AND ");

    let sql = format!(
        "SELECT s.id, s.usuario_id, s.registradora_id, s.status, s.aberto_em, s.fechado_em,
                sm.moeda_codigo, sm.valor_abertura_minor, sm.valor_esperado_minor,
                sm.valor_fechamento_informado_minor, sm.diferenca_minor
         FROM sessoes_caixa s
         INNER JOIN sessoes_caixa_moedas sm ON s.id = sm.sessao_id
         WHERE {}
         ORDER BY s.aberto_em DESC
         LIMIT 500",
        where_clause
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

    let sessoes = stmt.query_map(&*params_refs, |row| {
        Ok(RelatorioCaixaItem {
            id: row.get(0)?,
            operador_id: row.get(1)?,
            terminal_id: row.get(2)?,
            status: row.get(3)?,
            aberto_em: row.get(4)?,
            fechado_em: row.get(5)?,
            moeda_codigo: row.get(6)?,
            valor_abertura_minor: row.get(7)?,
            valor_fechamento_esperado_minor: row.get(8)?,
            valor_fechamento_informado_minor: row.get(9)?,
            diferenca_minor: row.get(10)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(RespostaBase::ok("Relatório de caixas gerado com sucesso", RelatorioCaixaResp {
        sessoes,
    }))
}

#[command]
pub async fn gerar_relatorio_financeiro(
    estado: State<'_, EstadoApp>,
    filtros: FiltrosRelatorio,
) -> Result<RespostaBase<RelatorioFinanceiroResp>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let (d_inicio, d_fim) = obter_datas_periodo(&filtros.data_inicio, &filtros.data_fim);

    // Contas a Pagar
    let mut cp_conds = vec!["data_vencimento >= ?1".to_string(), "data_vencimento <= ?2".to_string()];
    let mut cp_params: Vec<Box<dyn rusqlite::ToSql>> = vec![
        Box::new(d_inicio.clone()),
        Box::new(d_fim.clone()),
    ];
    if let Some(ref u_id) = filtros.usuario_id {
        if !u_id.is_empty() {
            let idx = cp_params.len() + 1;
            cp_conds.push(format!("usuario_id = ?{}", idx));
            cp_params.push(Box::new(u_id.clone()));
        }
    }
    if let Some(ref m_cod) = filtros.moeda_codigo {
        if !m_cod.is_empty() {
            let idx = cp_params.len() + 1;
            cp_conds.push(format!("moeda_codigo = ?{}", idx));
            cp_params.push(Box::new(m_cod.clone()));
        }
    }

    let cp_where = cp_conds.join(" AND ");
    let sql_cp = format!(
        "SELECT id, fornecedor_id, fornecedor_nome_snapshot, compra_id, descricao, moeda_codigo,
                valor_original_minor, taxa_cambio_escala6, valor_original_principal_minor,
                data_emissao, data_vencimento, status, saldo_pendente_minor, criado_em,
                atualizado_em, usuario_id, observacao
         FROM contas_pagar
         WHERE {}
         ORDER BY data_vencimento ASC
         LIMIT 500",
        cp_where
    );
    let mut stmt_cp = conn.prepare(&sql_cp).map_err(|e| e.to_string())?;
    let cp_refs: Vec<&dyn rusqlite::ToSql> = cp_params.iter().map(|p| p.as_ref()).collect();
    let contas_pagar = stmt_cp.query_map(&*cp_refs, |row| {
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
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    // Contas a Receber
    let mut cr_conds = vec!["data_vencimento >= ?1".to_string(), "data_vencimento <= ?2".to_string()];
    let mut cr_params: Vec<Box<dyn rusqlite::ToSql>> = vec![
        Box::new(d_inicio.clone()),
        Box::new(d_fim.clone()),
    ];
    if let Some(ref u_id) = filtros.usuario_id {
        if !u_id.is_empty() {
            let idx = cr_params.len() + 1;
            cr_conds.push(format!("usuario_id = ?{}", idx));
            cr_params.push(Box::new(u_id.clone()));
        }
    }
    if let Some(ref m_cod) = filtros.moeda_codigo {
        if !m_cod.is_empty() {
            let idx = cr_params.len() + 1;
            cr_conds.push(format!("moeda_codigo = ?{}", idx));
            cr_params.push(Box::new(m_cod.clone()));
        }
    }

    let cr_where = cr_conds.join(" AND ");
    let sql_cr = format!(
        "SELECT id, cliente_id, cliente_nome_snapshot, venda_id, descricao, moeda_codigo,
                valor_original_minor, taxa_cambio_escala6, valor_original_principal_minor,
                data_emissao, data_vencimento, status, saldo_pendente_minor, criado_em,
                atualizado_em, usuario_id, observacao
         FROM contas_receber
         WHERE {}
         ORDER BY data_vencimento ASC
         LIMIT 500",
        cr_where
    );
    let mut stmt_cr = conn.prepare(&sql_cr).map_err(|e| e.to_string())?;
    let cr_refs: Vec<&dyn rusqlite::ToSql> = cr_params.iter().map(|p| p.as_ref()).collect();
    let contas_receber = stmt_cr.query_map(&*cr_refs, |row| {
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
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    // Lançamentos Financeiros
    let mut fl_conds = vec!["data_pagamento >= ?1".to_string(), "data_pagamento <= ?2".to_string()];
    let mut fl_params: Vec<Box<dyn rusqlite::ToSql>> = vec![
        Box::new(d_inicio.clone()),
        Box::new(d_fim.clone()),
    ];
    if let Some(ref u_id) = filtros.usuario_id {
        if !u_id.is_empty() {
            let idx = fl_params.len() + 1;
            fl_conds.push(format!("usuario_id = ?{}", idx));
            fl_params.push(Box::new(u_id.clone()));
        }
    }
    if let Some(ref m_cod) = filtros.moeda_codigo {
        if !m_cod.is_empty() {
            let idx = fl_params.len() + 1;
            fl_conds.push(format!("moeda_codigo = ?{}", idx));
            fl_params.push(Box::new(m_cod.clone()));
        }
    }
    if let Some(ref f_pag) = filtros.forma_pagamento {
        if !f_pag.is_empty() {
            let idx = fl_params.len() + 1;
            fl_conds.push(format!("forma_pagamento = ?{}", idx));
            fl_params.push(Box::new(f_pag.clone()));
        }
    }

    let fl_where = fl_conds.join(" AND ");
    let sql_fl = format!(
        "SELECT id, conta_pagar_id, conta_receber_id, sessao_caixa_id, tipo_lancamento,
                forma_pagamento, moeda_codigo, valor_informado_minor, taxa_cambio_escala6,
                valor_principal_minor, data_pagamento, usuario_id, observacao, criado_em
         FROM financeiro_lancamentos
         WHERE {}
         ORDER BY data_pagamento DESC
         LIMIT 500",
        fl_where
    );
    let mut stmt_fl = conn.prepare(&sql_fl).map_err(|e| e.to_string())?;
    let fl_refs: Vec<&dyn rusqlite::ToSql> = fl_params.iter().map(|p| p.as_ref()).collect();
    let lancamentos = stmt_fl.query_map(&*fl_refs, |row| {
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
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    // Totais pendentes gerais
    let mut stmt_cpp = conn.prepare(
        "SELECT moeda_codigo, SUM(saldo_pendente_minor)
         FROM contas_pagar
         WHERE status IN ('PENDENTE', 'PAGO_PARCIAL')
         GROUP BY moeda_codigo"
    ).map_err(|e| e.to_string())?;
    let total_pagar_pendente = stmt_cpp.query_map([], |row| {
        Ok(TotalPorMoeda {
            moeda_codigo: row.get(0)?,
            valor_minor: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    let mut stmt_crp = conn.prepare(
        "SELECT moeda_codigo, SUM(saldo_pendente_minor)
         FROM contas_receber
         WHERE status IN ('PENDENTE', 'PAGO_PARCIAL')
         GROUP BY moeda_codigo"
    ).map_err(|e| e.to_string())?;
    let total_receber_pendente = stmt_crp.query_map([], |row| {
        Ok(TotalPorMoeda {
            moeda_codigo: row.get(0)?,
            valor_minor: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(RespostaBase::ok("Relatório financeiro gerado com sucesso", RelatorioFinanceiroResp {
        contas_pagar,
        contas_receber,
        lancamentos,
        total_pagar_pendente,
        total_receber_pendente,
    }))
}

#[command]
pub async fn gerar_relatorio_estoque_kardex(
    estado: State<'_, EstadoApp>,
    filtros: FiltrosRelatorio,
) -> Result<RespostaBase<RelatorioEstoqueResp>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let (d_inicio, d_fim) = obter_datas_periodo(&filtros.data_inicio, &filtros.data_fim);

    // 1. Posição atual de estoque e valorização baseada no último custo
    let mut stmt_pos = conn.prepare(
        "SELECT pc.produto_id, pc.nome, pc.codigo, pc.controla_estoque,
                COALESCE(pec.quantidade_escala3, 0) as quantidade_escala3,
                pc.ultimo_custo_minor,
                COALESCE(pc.ultimo_custo_taxa_cambio_escala6, 1000000) as taxa_escala6
         FROM produtos_cache pc
         LEFT JOIN produtos_estoque_cache pec ON pc.produto_id = pec.produto_id
         WHERE pc.controla_estoque = 1 AND pc.ativo = 1
         ORDER BY pc.nome ASC"
    ).map_err(|e| e.to_string())?;

    let mut custo_total_estimado_brl: i64 = 0;
    let mut posicao_estoque = vec![];

    let mut rows_pos = stmt_pos.query([]).map_err(|e| e.to_string())?;
    while let Some(row) = rows_pos.next().map_err(|e| e.to_string())? {
        let produto_id: String = row.get(0).map_err(|e| e.to_string())?;
        let produto_nome: String = row.get(1).map_err(|e| e.to_string())?;
        let produto_sku: String = row.get::<_, Option<String>>(2).ok().flatten().unwrap_or_default();
        let controla_estoque: bool = row.get::<_, i32>(3).unwrap_or(1) != 0;
        let quantidade_escala3: i64 = row.get(4).map_err(|e| e.to_string())?;
        let ultimo_custo_minor: i64 = row.get(5).map_err(|e| e.to_string())?;
        let taxa_escala6: i64 = row.get(6).map_err(|e| e.to_string())?;

        // Converter o custo em moeda secundária para a moeda principal (BRL)
        let custo_brl_minor = (ultimo_custo_minor * taxa_escala6) / 1000000;
        if quantidade_escala3 > 0 {
            custo_total_estimado_brl += (quantidade_escala3 * custo_brl_minor) / 1000;
        }

        posicao_estoque.push(PosicaoEstoqueItem {
            produto_id,
            produto_nome,
            produto_sku,
            controla_estoque,
            quantidade_escala3,
            estoque_minimo_escala3: 0,
            ultimo_custo_minor,
        });
    }

    // 2. Movimentações do Kardex filtradas por período
    let mut stmt_kardex = conn.prepare(
        "SELECT em.id, em.produto_id, pc.nome, pc.codigo, em.tipo_movimentacao,
                em.quantidade_escala3, em.criado_em, em.origem_id, em.usuario_id, em.motivo
         FROM estoque_movimentacoes em
         INNER JOIN produtos_cache pc ON em.produto_id = pc.produto_id
         WHERE em.criado_em >= ?1 AND em.criado_em <= ?2
         ORDER BY em.criado_em DESC
         LIMIT 1000"
    ).map_err(|e| e.to_string())?;

    let itens_kardex = stmt_kardex.query_map([&d_inicio, &d_fim], |row| {
        Ok(EstoqueKardexItem {
            id: row.get(0)?,
            produto_id: row.get(1)?,
            produto_nome: row.get(2)?,
            produto_sku: row.get(3)?,
            tipo_movimentacao: row.get(4)?,
            quantidade_escala3: row.get(5)?,
            data_movimentacao: row.get(6)?,
            origem_id: row.get(7)?,
            usuario_id: row.get(8)?,
            observacao: row.get(9)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(RespostaBase::ok("Relatório de estoque gerado com sucesso", RelatorioEstoqueResp {
        posicao_estoque,
        itens_kardex,
        custo_total_estimado_brl,
    }))
}

#[command]
pub async fn gerar_relatorio_compras(
    estado: State<'_, EstadoApp>,
    filtros: FiltrosRelatorio,
) -> Result<RespostaBase<RelatorioComprasResp>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let (d_inicio, d_fim) = obter_datas_periodo(&filtros.data_inicio, &filtros.data_fim);

    let mut conds = vec!["c.criado_em >= ?1".to_string(), "c.criado_em <= ?2".to_string()];
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![
        Box::new(d_inicio.clone()),
        Box::new(d_fim.clone()),
    ];

    if let Some(ref u_id) = filtros.usuario_id {
        if !u_id.is_empty() {
            let idx = params_vec.len() + 1;
            conds.push(format!("c.usuario_id = ?{}", idx));
            params_vec.push(Box::new(u_id.clone()));
        }
    }

    if let Some(ref m_cod) = filtros.moeda_codigo {
        if !m_cod.is_empty() {
            let idx = params_vec.len() + 1;
            conds.push(format!("c.moeda_codigo = ?{}", idx));
            params_vec.push(Box::new(m_cod.clone()));
        }
    }

    let where_clause = conds.join(" AND ");

    // Query 1: Compras detalhadas
    let sql_compras = format!(
        "SELECT c.id, c.fornecedor_nome_snapshot, c.criado_em, c.status, c.moeda_codigo, c.total_compra_minor, c.taxa_cambio_escala6,
                (SELECT COALESCE(SUM(ci.quantidade_escala3), 0) FROM compra_itens ci WHERE ci.compra_id = c.id AND ci.cancelado = 0)
         FROM compras c
         WHERE {}
         ORDER BY c.criado_em DESC
         LIMIT 500",
        where_clause
    );

    let mut stmt_compras = conn.prepare(&sql_compras).map_err(|e| e.to_string())?;
    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

    let compras = stmt_compras.query_map(&*params_refs, |row| {
        let id: String = row.get(0)?;
        let fornecedor_nome: String = row.get(1)?;
        let data_compra: String = row.get(2)?;
        let status: String = row.get(3)?;
        let moeda_codigo: String = row.get(4)?;
        let total_original_minor: i64 = row.get(5)?;
        let taxa_cambio_escala6: i64 = row.get(6)?;
        let total_itens_escala3: i64 = row.get(7)?;

        let total_principal_brl_minor = (total_original_minor * taxa_cambio_escala6) / 1000000;

        Ok(CompraRelatorioItem {
            id,
            fornecedor_nome,
            data_compra,
            status,
            moeda_codigo,
            total_original_minor,
            total_principal_brl_minor,
            total_itens_escala3,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    // Query 2: Totais por Fornecedor (somente finalizadas)
    let sql_forn = format!(
        "SELECT c.fornecedor_nome_snapshot, SUM((c.total_compra_minor * c.taxa_cambio_escala6) / 1000000)
         FROM compras c
         WHERE c.status = 'FINALIZADA' AND {}
         GROUP BY c.fornecedor_nome_snapshot",
        where_clause
    );
    let mut stmt_forn = conn.prepare(&sql_forn).map_err(|e| e.to_string())?;
    let total_por_fornecedor = stmt_forn.query_map(&*params_refs, |row| {
        Ok(CompraFornecedorTotal {
            fornecedor_nome: row.get(0)?,
            total_principal_brl_minor: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    // Query 3: Totais por Moeda (somente finalizadas)
    let sql_moeda = format!(
        "SELECT c.moeda_codigo, SUM(c.total_compra_minor)
         FROM compras c
         WHERE c.status = 'FINALIZADA' AND {}
         GROUP BY c.moeda_codigo",
        where_clause
    );
    let mut stmt_moeda = conn.prepare(&sql_moeda).map_err(|e| e.to_string())?;
    let total_por_moeda = stmt_moeda.query_map(&*params_refs, |row| {
        Ok(TotalPorMoeda {
            moeda_codigo: row.get(0)?,
            valor_minor: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(RespostaBase::ok("Relatório de compras gerado com sucesso", RelatorioComprasResp {
        compras,
        total_por_fornecedor,
        total_por_moeda,
    }))
}

#[command]
pub async fn gerar_relatorio_produtos_mais_vendidos(
    estado: State<'_, EstadoApp>,
    filtros: FiltrosRelatorio,
) -> Result<RespostaBase<Vec<ProdutoMaisVendidoResp>>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let (d_inicio, d_fim) = obter_datas_periodo(&filtros.data_inicio, &filtros.data_fim);

    let mut conds = vec!["v.criado_em >= ?1".to_string(), "v.criado_em <= ?2".to_string()];
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![
        Box::new(d_inicio.clone()),
        Box::new(d_fim.clone()),
    ];

    if let Some(ref u_id) = filtros.usuario_id {
        if !u_id.is_empty() {
            let idx = params_vec.len() + 1;
            conds.push(format!("v.usuario_id = ?{}", idx));
            params_vec.push(Box::new(u_id.clone()));
        }
    }

    let where_clause = conds.join(" AND ");

    let sql = format!(
        "SELECT vi.produto_id, vi.descricao_produto, vi.codigo_produto,
                SUM(vi.quantidade_escala3) as total_qtd,
                SUM(vi.total_item_minor) as total_val
         FROM venda_itens vi
         INNER JOIN vendas v ON vi.venda_id = v.id
         WHERE v.status = 'FINALIZADA' AND vi.cancelado = 0 AND {}
         GROUP BY vi.produto_id, vi.descricao_produto, vi.codigo_produto
         ORDER BY total_qtd DESC
         LIMIT 100",
        where_clause
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

    let ranking = stmt.query_map(&*params_refs, |row| {
        Ok(ProdutoMaisVendidoResp {
            produto_id: row.get(0)?,
            produto_nome: row.get(1)?,
            produto_sku: row.get::<_, Option<String>>(2).ok().flatten().unwrap_or_default(),
            quantidade_vendida_escala3: row.get(3)?,
            faturamento_bruto_minor: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    Ok(RespostaBase::ok("Ranking de produtos mais vendidos gerado com sucesso", ranking))
}

#[command]
pub async fn gerar_relatorio_gourmet_delivery(
    estado: State<'_, EstadoApp>,
    filtros: FiltrosRelatorio,
) -> Result<RespostaBase<RelatorioGourmetDeliveryResp>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let (d_inicio, d_fim) = obter_datas_periodo(&filtros.data_inicio, &filtros.data_fim);

    // 1. Total pedidos delivery
    let total_pedidos_delivery: i64 = conn.query_row(
        "SELECT COUNT(*) FROM delivery_operacional WHERE aberto_em >= ?1 AND aberto_em <= ?2",
        [&d_inicio, &d_fim],
        |row| row.get(0)
    ).unwrap_or(0);

    // 2. Contagem delivery por status
    let mut stmt_del_stat = conn.prepare(
        "SELECT status, COUNT(*)
         FROM delivery_operacional
         WHERE aberto_em >= ?1 AND aberto_em <= ?2
         GROUP BY status"
    ).map_err(|e| e.to_string())?;
    let delivery_por_status = stmt_del_stat.query_map([&d_inicio, &d_fim], |row| {
        Ok(DeliveryStatusContagem {
            status: row.get(0)?,
            quantidade: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    // 3. Faturamento delivery por moeda
    let mut stmt_del_fat = conn.prepare(
        "SELECT vp.moeda_codigo, SUM(vp.valor_informado_minor - CASE WHEN vp.moeda_codigo = COALESCE(vp.moeda_troco_codigo, '') THEN vp.troco_minor ELSE 0 END)
         FROM venda_pagamentos vp
         INNER JOIN vendas v ON v.id = vp.venda_id
         WHERE v.origem_tipo = 'DELIVERY' AND v.status = 'FINALIZADA' AND v.criado_em >= ?1 AND v.criado_em <= ?2
         GROUP BY vp.moeda_codigo"
    ).map_err(|e| e.to_string())?;
    let faturamento_delivery_moeda = stmt_del_fat.query_map([&d_inicio, &d_fim], |row| {
        Ok(TotalPorMoeda {
            moeda_codigo: row.get(0)?,
            valor_minor: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    // 4. Taxa de entrega total
    let taxa_entrega_total_minor: i64 = conn.query_row(
        "SELECT COALESCE(SUM(v.taxa_entrega_minor), 0)
         FROM vendas v
         WHERE v.origem_tipo = 'DELIVERY' AND v.status = 'FINALIZADA' AND v.criado_em >= ?1 AND v.criado_em <= ?2",
        [&d_inicio, &d_fim],
        |row| row.get(0)
    ).unwrap_or(0);

    // 5. Total atendimentos gourmet (Mesa + Comanda abertos no período)
    let total_mesas: i64 = conn.query_row(
        "SELECT COUNT(*) FROM mesas_operacionais WHERE aberta_em >= ?1 AND aberta_em <= ?2",
        [&d_inicio, &d_fim],
        |row| row.get(0)
    ).unwrap_or(0);

    let total_comandas: i64 = conn.query_row(
        "SELECT COUNT(*) FROM comandas_operacionais WHERE aberta_em >= ?1 AND aberta_em <= ?2",
        [&d_inicio, &d_fim],
        |row| row.get(0)
    ).unwrap_or(0);

    let total_atendimentos_gourmet = total_mesas + total_comandas;

    // 6. Faturamento gourmet por moeda
    let mut stmt_gour_fat = conn.prepare(
        "SELECT vp.moeda_codigo, SUM(vp.valor_informado_minor - CASE WHEN vp.moeda_codigo = COALESCE(vp.moeda_troco_codigo, '') THEN vp.troco_minor ELSE 0 END)
         FROM venda_pagamentos vp
         INNER JOIN vendas v ON v.id = vp.venda_id
         WHERE v.origem_tipo IN ('MESA', 'COMANDA') AND v.status = 'FINALIZADA' AND v.criado_em >= ?1 AND v.criado_em <= ?2
         GROUP BY vp.moeda_codigo"
    ).map_err(|e| e.to_string())?;
    let faturamento_gourmet_moeda = stmt_gour_fat.query_map([&d_inicio, &d_fim], |row| {
        Ok(TotalPorMoeda {
            moeda_codigo: row.get(0)?,
            valor_minor: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    // 7. Ticket médio gourmet BRL (principal)
    let res_ticket: (i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(total_minor), 0), COUNT(*)
         FROM vendas
         WHERE origem_tipo IN ('MESA', 'COMANDA') AND status = 'FINALIZADA' AND criado_em >= ?1 AND criado_em <= ?2",
        [&d_inicio, &d_fim],
        |row| Ok((row.get(0)?, row.get(1)?))
    ).unwrap_or((0, 0));

    let ticket_medio_gourmet_brl_minor = if res_ticket.1 > 0 {
        res_ticket.0 / res_ticket.1
    } else {
        0
    };

    Ok(RespostaBase::ok("Relatório Gourmet e Delivery gerado com sucesso", RelatorioGourmetDeliveryResp {
        total_pedidos_delivery,
        delivery_por_status,
        faturamento_delivery_moeda,
        taxa_entrega_total_minor,
        total_atendimentos_gourmet,
        faturamento_gourmet_moeda,
        ticket_medio_gourmet_brl_minor,
    }))
}
