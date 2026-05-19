use tauri::State;
use tracing::{info, error};
use uuid::Uuid;

use aureon_core::{dtos::*, RespostaBase};
use crate::estado::EstadoApp;

/// Abre uma nova sessao de caixa para uma registradora.
/// Garante que nao haja outra sessao ABERTO para a mesma registradora.
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

    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    // Verificar se ja existe sessao ABERTO para esta registradora
    let sessao_aberta: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM sessoes_caixa WHERE registradora_id = ?1 AND status = 'ABERTO'",
        rusqlite::params![&dto.registradora_id],
        |row| row.get(0),
    ).unwrap_or(false);

    if sessao_aberta {
        return Err("Ja existe uma sessao de caixa aberta para esta registradora.".into());
    }

    let id = Uuid::new_v4().to_string();
    let agora = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    conn.execute(
        "INSERT INTO sessoes_caixa (id, registradora_id, usuario_id, status, valor_abertura, aberto_em)
         VALUES (?1, ?2, ?3, 'ABERTO', ?4, ?5)",
        rusqlite::params![
            &id,
            &dto.registradora_id,
            &dto.usuario_id,
            dto.valor_abertura,
            &agora
        ],
    ).map_err(|e| {
        error!(componente = "aureon-pdv::commands_caixa", erro = %e, "Erro ao abrir caixa");
        e.to_string()
    })?;

    let resp = SessaoCaixaResp {
        id,
        registradora_id: dto.registradora_id,
        usuario_id: dto.usuario_id,
        status: "ABERTO".into(),
        valor_abertura: dto.valor_abertura,
        valor_fechamento: None,
        aberto_em: agora,
        fechado_em: None,
    };

    Ok(RespostaBase::ok("Caixa aberto com sucesso", resp))
}

/// Fecha uma sessao de caixa existente.
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

    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    // Verificar se sessao existe e esta ABERTO
    let sessao_existe: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM sessoes_caixa WHERE id = ?1 AND status = 'ABERTO'",
        rusqlite::params![&dto.sessao_id],
        |row| row.get(0),
    ).unwrap_or(false);

    if !sessao_existe {
        return Err("Sessao de caixa nao encontrada ou ja fechada.".into());
    }

    let agora = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    conn.execute(
        "UPDATE sessoes_caixa SET status = 'FECHADO', valor_fechamento = ?1, fechado_em = ?2
         WHERE id = ?3",
        rusqlite::params![dto.valor_fechamento, &agora, &dto.sessao_id],
    ).map_err(|e| {
        error!(componente = "aureon-pdv::commands_caixa", erro = %e, "Erro ao fechar caixa");
        e.to_string()
    })?;

    // Buscar sessao atualizada para retornar
    let resp = conn.query_row(
        "SELECT id, registradora_id, usuario_id, status, valor_abertura, valor_fechamento, aberto_em, fechado_em
         FROM sessoes_caixa WHERE id = ?1",
        rusqlite::params![&dto.sessao_id],
        |row| {
            Ok(SessaoCaixaResp {
                id:               row.get(0)?,
                registradora_id:  row.get(1)?,
                usuario_id:       row.get(2)?,
                status:           row.get(3)?,
                valor_abertura:   row.get(4)?,
                valor_fechamento: row.get(5)?,
                aberto_em:        row.get(6)?,
                fechado_em:       row.get(7)?,
            })
        },
    ).map_err(|e| e.to_string())?;

    Ok(RespostaBase::ok("Caixa fechado com sucesso", resp))
}

/// Retorna a sessao ABERTO ativa para uma registradora, se houver.
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
        "SELECT id, registradora_id, usuario_id, status, valor_abertura, valor_fechamento, aberto_em, fechado_em
         FROM sessoes_caixa
         WHERE registradora_id = ?1 AND status = 'ABERTO'
         LIMIT 1",
        rusqlite::params![&registradora_id],
        |row| {
            Ok(SessaoCaixaResp {
                id:               row.get(0)?,
                registradora_id:  row.get(1)?,
                usuario_id:       row.get(2)?,
                status:           row.get(3)?,
                valor_abertura:   row.get(4)?,
                valor_fechamento: row.get(5)?,
                aberto_em:        row.get(6)?,
                fechado_em:       row.get(7)?,
            })
        },
    ).ok(); // None se nao houver sessao aberta

    Ok(RespostaBase::ok("Sessao ativa", sessao))
}

/// Lista as ultimas N sessoes de caixa (todas as registradoras).
#[tauri::command]
pub async fn listar_sessoes(
    limite: u32,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<Vec<SessaoCaixaResp>>, String> {
    info!(componente = "aureon-pdv::commands_caixa", limite, "Chamada: listar_sessoes");

    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let limite = limite.min(200) as i64; // maximo 200 para protecao

    let mut stmt = conn.prepare(
        "SELECT id, registradora_id, usuario_id, status, valor_abertura, valor_fechamento, aberto_em, fechado_em
         FROM sessoes_caixa
         ORDER BY aberto_em DESC
         LIMIT ?1"
    ).map_err(|e| e.to_string())?;

    let iter = stmt.query_map(rusqlite::params![limite], |row| {
        Ok(SessaoCaixaResp {
            id:               row.get(0)?,
            registradora_id:  row.get(1)?,
            usuario_id:       row.get(2)?,
            status:           row.get(3)?,
            valor_abertura:   row.get(4)?,
            valor_fechamento: row.get(5)?,
            aberto_em:        row.get(6)?,
            fechado_em:       row.get(7)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut sessoes = Vec::new();
    for s in iter {
        if let Ok(val) = s { sessoes.push(val); }
    }

    Ok(RespostaBase::ok("Sessoes listadas", sessoes))
}
