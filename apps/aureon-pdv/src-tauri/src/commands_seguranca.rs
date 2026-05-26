use std::sync::Mutex;
use tauri::{command, State};
use uuid::Uuid;
use chrono::Utc;
use rusqlite::OptionalExtension;
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use tracing::{info, warn, error};

use aureon_core::{
    dtos::{
        LoginLocalReq, LoginLocalResp, SessaoUsuarioResp, 
        UsuarioLocalResp, PerfilLocalResp, PermissaoLocalResp, 
        UsuarioTemPermissaoReq, UsuarioTemPermissaoResp,
        VerificarPermissaoOperacaoReq, VerificarPermissaoOperacaoResp,
        AutorizarOperacaoSupervisorReq, AutorizarOperacaoSupervisorResp
    },
};
use crate::estado::EstadoApp;

#[command]
pub fn login_local(
    req: LoginLocalReq,
    estado: State<'_, EstadoApp>,
) -> Result<LoginLocalResp, String> {
    info!(componente = "commands_seguranca", login = %req.login, "Tentativa de login local");

    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    // 1. Busca usuÃƒÂ¡rio
    let user_row: Option<(String, String, String, String, bool, bool)> = conn.query_row(
        "SELECT id, nome, login, senha_hash, ativo, exige_troca_senha FROM usuarios_local WHERE login = ?1",
        rusqlite::params![req.login],
        |row| Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get::<_, i32>(4)? == 1,
            row.get::<_, i32>(5)? == 1,
        ))
    ).optional().map_err(|e| e.to_string())?;

    let (user_id, nome, login, hash_banco, ativo, exige_troca_senha): (String, String, String, String, bool, bool) = match user_row {
        Some(u) => u,
        None => {
            // Falha login (nao encontrado) - Registrar auditoria
            let _ = conn.execute(
                "INSERT INTO auditoria_operacional_local (id, login, tipo_evento, sucesso, mensagem, criado_em) VALUES (?1, ?2, 'LOGIN_FALHA', 0, 'UsuÃƒÂ¡rio nÃƒÂ£o encontrado', ?3)",
                rusqlite::params![Uuid::new_v4().to_string(), req.login, Utc::now().to_rfc3339()]
            );
            return Ok(LoginLocalResp {
                sucesso: false,
                usuario_id: None,
                login: None,
                nome: None,
                sessao_id: None,
                perfis: vec![],
                permissoes: vec![],
                exige_troca_senha: false,
                mensagem: "UsuÃƒÂ¡rio ou senha incorretos".to_string(),
                warnings: vec![],
            });
        }
    };

    if !ativo {
        let _ = conn.execute(
            "INSERT INTO auditoria_operacional_local (id, usuario_id, login, tipo_evento, sucesso, mensagem, criado_em) VALUES (?1, ?2, ?3, 'LOGIN_FALHA', 0, 'UsuÃƒÂ¡rio inativo', ?4)",
            rusqlite::params![Uuid::new_v4().to_string(), user_id, login, Utc::now().to_rfc3339()]
        );
        return Ok(LoginLocalResp {
            sucesso: false,
            usuario_id: None,
            login: None,
            nome: None,
            sessao_id: None,
            perfis: vec![],
            permissoes: vec![],
            exige_troca_senha: false,
            mensagem: "UsuÃƒÂ¡rio inativo".to_string(),
            warnings: vec![],
        });
    }

    // 2. Valida Senha
    let parsed_hash = match PasswordHash::new(&hash_banco) {
        Ok(h) => h,
        Err(_) => {
            error!("Hash armazenado invÃƒÂ¡lido para o usuÃƒÂ¡rio {}", login);
            return Err("Erro interno ao validar senha".to_string());
        }
    };

    let argon2 = Argon2::default();
    if argon2.verify_password(req.senha_pura.as_bytes(), &parsed_hash).is_err() {
        let _ = conn.execute(
            "INSERT INTO auditoria_operacional_local (id, usuario_id, login, tipo_evento, sucesso, mensagem, criado_em) VALUES (?1, ?2, ?3, 'LOGIN_FALHA', 0, 'Senha incorreta', ?4)",
            rusqlite::params![Uuid::new_v4().to_string(), user_id, login, Utc::now().to_rfc3339()]
        );
        return Ok(LoginLocalResp {
            sucesso: false,
            usuario_id: None,
            login: None,
            nome: None,
            sessao_id: None,
            perfis: vec![],
            permissoes: vec![],
            exige_troca_senha: false,
            mensagem: "UsuÃƒÂ¡rio ou senha incorretos".to_string(),
            warnings: vec![],
        });
    }

    // 3. Sucesso! Carregar Perfis e PermissÃƒÂµes
    let mut perfis = Vec::new();
    let mut stmt = conn.prepare("SELECT p.codigo FROM perfis_local p JOIN usuario_perfis_local up ON p.id = up.perfil_id WHERE up.usuario_id = ?1").unwrap();
    let rows = stmt.query_map(rusqlite::params![user_id], |row| row.get::<_, String>(0)).unwrap();
    for r in rows {
        if let Ok(c) = r { perfis.push(c); }
    }

    let mut permissoes = Vec::new();
    let mut stmt = conn.prepare("
        SELECT DISTINCT pm.codigo 
        FROM permissoes_local pm 
        JOIN perfil_permissoes_local pp ON pm.id = pp.permissao_id 
        JOIN usuario_perfis_local up ON pp.perfil_id = up.perfil_id 
        WHERE up.usuario_id = ?1 AND pp.permitido = 1
    ").unwrap();
    let rows = stmt.query_map(rusqlite::params![user_id], |row| row.get::<_, String>(0)).unwrap();
    for r in rows {
        if let Ok(c) = r { permissoes.push(c); }
    }

    let now = Utc::now().to_rfc3339();

    // 4. Encerrar sessoes anteriores do mesmo terminal
    // OBS: Como nÃƒÂ£o temos terminal estrito neste bloco, vamos encerrar todas as ativas do user.
    let _ = conn.execute("UPDATE sessoes_usuario_local SET ativa = 0, encerrada_em = ?1 WHERE usuario_id = ?2 AND ativa = 1", rusqlite::params![now, user_id]);

    // 5. Criar SessÃƒÂ£o
    let sessao_id = Uuid::new_v4().to_string();
    let _ = conn.execute("
        INSERT INTO sessoes_usuario_local (id, usuario_id, login, nome_usuario, aberta_em, ativa) 
        VALUES (?1, ?2, ?3, ?4, ?5, 1)
    ", rusqlite::params![sessao_id, user_id, login, nome, now]);

    // 6. Atualizar ultimo login e gerar auditoria
    let _ = conn.execute("UPDATE usuarios_local SET ultimo_login_em = ?1 WHERE id = ?2", rusqlite::params![now, user_id]);
    let _ = conn.execute(
        "INSERT INTO auditoria_operacional_local (id, usuario_id, login, tipo_evento, sucesso, mensagem, criado_em) VALUES (?1, ?2, ?3, 'LOGIN_SUCESSO', 1, 'Login efetuado com sucesso', ?4)",
        rusqlite::params![Uuid::new_v4().to_string(), user_id, login, now]
    );

    Ok(LoginLocalResp {
        sucesso: true,
        usuario_id: Some(user_id),
        login: Some(login),
        nome: Some(nome),
        sessao_id: Some(sessao_id),
        perfis,
        permissoes,
        exige_troca_senha,
        mensagem: "Login realizado com sucesso".to_string(),
        warnings: vec![],
    })
}

#[command]
pub fn logout_local(estado: State<'_, EstadoApp>) -> Result<bool, String> {
    info!(componente = "commands_seguranca", "Logout local");
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    
    // Pega a sessÃƒÂ£o ativa
    let sessao_row: Option<(String, String, String)> = conn.query_row(
        "SELECT id, usuario_id, login FROM sessoes_usuario_local WHERE ativa = 1 ORDER BY aberta_em DESC LIMIT 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    ).optional().map_err(|e| e.to_string())?;

    if let Some((sessao_id, user_id, login)) = sessao_row {
        let now = Utc::now().to_rfc3339();
        let _ = conn.execute("UPDATE sessoes_usuario_local SET ativa = 0, encerrada_em = ?1 WHERE id = ?2", rusqlite::params![now, sessao_id]);
        let _ = conn.execute(
            "INSERT INTO auditoria_operacional_local (id, usuario_id, login, tipo_evento, sucesso, mensagem, criado_em) VALUES (?1, ?2, ?3, 'LOGOUT', 1, 'SessÃƒÂ£o encerrada manualmente', ?4)",
            rusqlite::params![Uuid::new_v4().to_string(), user_id, login, now]
        );
        Ok(true)
    } else {
        Ok(false)
    }
}

#[command]
pub fn obter_sessao_usuario_atual(estado: State<'_, EstadoApp>) -> Result<SessaoUsuarioResp, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let sessao_row: Option<(String, String, String, String, String)> = conn.query_row(
        "SELECT id, usuario_id, login, nome_usuario, aberta_em FROM sessoes_usuario_local WHERE ativa = 1 ORDER BY aberta_em DESC LIMIT 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
    ).optional().map_err(|e| e.to_string())?;

    let (sessao_id, user_id, login, nome, aberta_em) = match sessao_row {
        Some(s) => s,
        None => {
            return Ok(SessaoUsuarioResp {
                autenticado: false,
                usuario_id: None,
                login: None,
                nome: None,
                sessao_id: None,
                perfis: vec![],
                permissoes: vec![],
                aberta_em: None,
                mensagem: "Nenhuma sessÃƒÂ£o ativa".to_string(),
            });
        }
    };

    let mut perfis = Vec::new();
    let mut stmt = conn.prepare("SELECT p.codigo FROM perfis_local p JOIN usuario_perfis_local up ON p.id = up.perfil_id WHERE up.usuario_id = ?1").unwrap();
    let rows = stmt.query_map(rusqlite::params![user_id], |row| row.get::<_, String>(0)).unwrap();
    for r in rows { if let Ok(c) = r { perfis.push(c); } }

    let mut permissoes = Vec::new();
    let mut stmt = conn.prepare("
        SELECT DISTINCT pm.codigo 
        FROM permissoes_local pm 
        JOIN perfil_permissoes_local pp ON pm.id = pp.permissao_id 
        JOIN usuario_perfis_local up ON pp.perfil_id = up.perfil_id 
        WHERE up.usuario_id = ?1 AND pp.permitido = 1
    ").unwrap();
    let rows = stmt.query_map(rusqlite::params![user_id], |row| row.get::<_, String>(0)).unwrap();
    for r in rows { if let Ok(c) = r { permissoes.push(c); } }

    // Auditoria opcional
    let _ = conn.execute(
        "INSERT INTO auditoria_operacional_local (id, usuario_id, login, tipo_evento, sucesso, mensagem, criado_em) VALUES (?1, ?2, ?3, 'SESSAO_CONSULTADA', 1, 'SessÃƒÂ£o atual consultada', ?4)",
        rusqlite::params![Uuid::new_v4().to_string(), user_id, login, Utc::now().to_rfc3339()]
    );

    Ok(SessaoUsuarioResp {
        autenticado: true,
        usuario_id: Some(user_id),
        login: Some(login),
        nome: Some(nome),
        sessao_id: Some(sessao_id),
        perfis,
        permissoes,
        aberta_em: Some(aberta_em),
        mensagem: "SessÃƒÂ£o recuperada".to_string(),
    })
}

#[command]
pub fn listar_usuarios_local(estado: State<'_, EstadoApp>) -> Result<Vec<UsuarioLocalResp>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("SELECT id, nome, login, ativo, ultimo_login_em FROM usuarios_local ORDER BY nome").map_err(|e| e.to_string())?;
    let mut users = Vec::new();
    
    let rows = stmt.query_map([], |row| {
        let ativo_i32: i32 = row.get(3)?;
        Ok(UsuarioLocalResp {
            id: row.get(0)?,
            nome: row.get(1)?,
            login: row.get(2)?,
            ativo: ativo_i32 == 1,
            perfis: vec![],
            ultimo_login_em: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;

    for mut r in rows.flatten() {
        // Buscar os perfis daquele usuÃƒÂ¡rio
        let mut p_stmt = conn.prepare("SELECT p.codigo FROM perfis_local p JOIN usuario_perfis_local up ON p.id = up.perfil_id WHERE up.usuario_id = ?1").unwrap();
        let p_rows = p_stmt.query_map(rusqlite::params![r.id], |row| row.get::<_, String>(0)).unwrap();
        for p in p_rows.flatten() {
            r.perfis.push(p);
        }
        users.push(r);
    }

    Ok(users)
}

#[command]
pub fn listar_perfis_local(estado: State<'_, EstadoApp>) -> Result<Vec<PerfilLocalResp>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, codigo, nome, descricao, ativo FROM perfis_local ORDER BY nome").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| {
        let ativo: i32 = row.get(4)?;
        Ok(PerfilLocalResp {
            id: row.get(0)?,
            codigo: row.get(1)?,
            nome: row.get(2)?,
            descricao: row.get(3)?,
            ativo: ativo == 1,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut res = Vec::new();
    for r in rows {
        if let Ok(p) = r { res.push(p); }
    }
    Ok(res)
}

#[command]
pub fn listar_permissoes_local(estado: State<'_, EstadoApp>) -> Result<Vec<PermissaoLocalResp>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, codigo, modulo, acao, descricao, risco FROM permissoes_local ORDER BY modulo, acao").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| {
        Ok(PermissaoLocalResp {
            id: row.get(0)?,
            codigo: row.get(1)?,
            modulo: row.get(2)?,
            acao: row.get(3)?,
            descricao: row.get(4)?,
            risco: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut res = Vec::new();
    for r in rows {
        if let Ok(p) = r { res.push(p); }
    }
    Ok(res)
}

#[command]
pub fn usuario_tem_permissao(
    req: UsuarioTemPermissaoReq,
    estado: State<'_, EstadoApp>
) -> Result<UsuarioTemPermissaoResp, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    
    let uid = if let Some(u) = req.usuario_id.clone() {
        u
    } else {
        // Tenta pegar a sessao ativa
        let s_row: Option<String> = conn.query_row(
            "SELECT usuario_id FROM sessoes_usuario_local WHERE ativa = 1 LIMIT 1",
            [],
            |r| r.get(0)
        ).optional().map_err(|e| e.to_string())?;

        match s_row {
            Some(u) => u,
            None => return Ok(UsuarioTemPermissaoResp {
                permitido: false,
                usuario_id: None,
                permissao_codigo: req.permissao_codigo,
                mensagem: "Nenhum usuÃƒÂ¡rio logado".to_string()
            })
        }
    };

    let p_row: Option<i32> = conn.query_row("
        SELECT 1
        FROM permissoes_local pm 
        JOIN perfil_permissoes_local pp ON pm.id = pp.permissao_id 
        JOIN usuario_perfis_local up ON pp.perfil_id = up.perfil_id 
        WHERE up.usuario_id = ?1 AND pm.codigo = ?2 AND pp.permitido = 1
        LIMIT 1
    ", rusqlite::params![uid, req.permissao_codigo], |r| r.get(0)).optional().map_err(|e| e.to_string())?;

    let permitido = p_row.is_some();

    // Opcional: registrar em auditoria_operacional se a consulta for muito importante
    // NÃƒÂ£o faria isso para todo check para nÃƒÂ£o floodar.

    Ok(UsuarioTemPermissaoResp {
        permitido,
        usuario_id: Some(uid),
        permissao_codigo: req.permissao_codigo,
        mensagem: if permitido { "PermissÃƒÂ£o concedida".into() } else { "Acesso negado".into() }
    })
}

pub fn avaliar_permissao_usuario(
    conn: &rusqlite::Connection,
    permissao_codigo: &str,
    contexto: Option<&str>,
    origem: Option<&str>,
) -> Result<VerificarPermissaoOperacaoResp, String> {
    let s_row: Option<(String, String, String)> = conn.query_row(
        "SELECT id, usuario_id, login FROM sessoes_usuario_local WHERE ativa = 1 LIMIT 1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))
    ).optional().map_err(|e| e.to_string())?;

    let (sessao_id, usuario_id, login) = match s_row {
        Some(s) => s,
        None => {
            let msg = "OperaÃ§Ã£o exige usuÃ¡rio logado.".to_string();
            let _ = conn.execute(
                "INSERT INTO auditoria_operacional_local (id, usuario_id, login, tipo_evento, sucesso, mensagem, criado_em, entidade_id, modulo) VALUES (?1, NULL, NULL, 'SESSAO_AUSENTE_OPERACAO_NEGADA', 0, ?2, ?3, ?4, ?5)",
                rusqlite::params![Uuid::new_v4().to_string(), msg, Utc::now().to_rfc3339(), contexto, origem]
            );
            return Ok(VerificarPermissaoOperacaoResp {
                permitido: false,
                usuario_id: None,
                login: None,
                permissao_codigo: permissao_codigo.to_string(),
                mensagem: msg,
                motivo_negacao: Some("SESSAO_AUSENTE".to_string()),
                warnings: vec![],
            });
        }
    };

    let p_row: Option<i32> = conn.query_row("
        SELECT 1
        FROM permissoes_local pm 
        JOIN perfil_permissoes_local pp ON pm.id = pp.permissao_id 
        JOIN usuario_perfis_local up ON pp.perfil_id = up.perfil_id 
        WHERE up.usuario_id = ?1 AND pm.codigo = ?2 AND pp.permitido = 1
        LIMIT 1
    ", rusqlite::params![usuario_id, permissao_codigo], |r| r.get(0)).optional().map_err(|e| e.to_string())?;

    let permitido = p_row.is_some();
    
    let (evento, sucesso, msg, motivo) = if permitido {
        ("PERMISSAO_OPERACAO_PERMITIDA", 1, format!("PermissÃ£o {} concedida", permissao_codigo), None)
    } else {
        ("PERMISSAO_OPERACAO_NEGADA", 0, format!("OperaÃ§Ã£o negada por falta de permissÃ£o: {}", permissao_codigo), Some("PERMISSAO_NEGADA".to_string()))
    };

    let _ = conn.execute(
        "INSERT INTO auditoria_operacional_local (id, usuario_id, login, tipo_evento, sucesso, mensagem, criado_em, entidade_id, modulo) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![Uuid::new_v4().to_string(), usuario_id, login, evento, sucesso, msg, Utc::now().to_rfc3339(), contexto, origem]
    );

    Ok(VerificarPermissaoOperacaoResp {
        permitido,
        usuario_id: Some(usuario_id),
        login: Some(login),
        permissao_codigo: permissao_codigo.to_string(),
        mensagem: msg,
        motivo_negacao: motivo,
        warnings: vec![],
    })
}

pub fn garantir_permissao_usuario(
    conn: &rusqlite::Connection,
    permissao_codigo: &str,
    contexto: Option<&str>,
    origem: Option<&str>,
) -> Result<(), String> {
    let resp = avaliar_permissao_usuario(conn, permissao_codigo, contexto, origem)?;
    if resp.permitido {
        Ok(())
    } else {
        let _ = conn.execute(
            "INSERT INTO auditoria_operacional_local (id, usuario_id, login, tipo_evento, sucesso, mensagem, criado_em, entidade_id, modulo) VALUES (?1, ?2, ?3, 'OPERACAO_BLOQUEADA_PERMISSAO', 0, ?4, ?5, ?6, ?7)",
            rusqlite::params![Uuid::new_v4().to_string(), resp.usuario_id, resp.login, resp.mensagem.clone(), Utc::now().to_rfc3339(), contexto, origem]
        );
        Err(resp.mensagem)
    }
}

#[command]
pub fn verificar_permissao_operacao(
    req: aureon_core::dtos::VerificarPermissaoOperacaoReq,
    estado: State<'_, EstadoApp>
) -> Result<VerificarPermissaoOperacaoResp, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    avaliar_permissao_usuario(&conn, &req.permissao_codigo, req.contexto_id.as_deref(), req.origem.as_deref())
}

// --- SUPERVISOR / AUTORIZACAO SENSIVEL ---

pub fn validar_credencial_supervisor(
    conn: &rusqlite::Connection,
    permissao_codigo: &str,
    login_sup: &str,
    senha_sup: &str,
    contexto: Option<&str>,
    origem: Option<&str>,
) -> Result<AutorizarOperacaoSupervisorResp, String> {
    // 1. Verificar se usuario existe, esta ativo e pegar o hash e perfis
    let (u_id, phash, ativo) = match conn.query_row(
        "SELECT id, password_hash, ativo FROM usuarios_local WHERE login = ?1",
        rusqlite::params![login_sup],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, bool>(2)?))
    ) {
        Ok(r) => r,
        Err(_) => return Ok(AutorizarOperacaoSupervisorResp {
            autorizado: false,
            supervisor_usuario_id: None,
            supervisor_login: Some(login_sup.to_string()),
            permissao_codigo: permissao_codigo.to_string(),
            contexto_id: contexto.map(|s| s.to_string()),
            mensagem: "Supervisor não encontrado.".to_string(),
            autorizacao_id: None,
            warnings: vec![],
        }),
    };

    if !ativo {
        return Ok(AutorizarOperacaoSupervisorResp {
            autorizado: false,
            supervisor_usuario_id: Some(u_id),
            supervisor_login: Some(login_sup.to_string()),
            permissao_codigo: permissao_codigo.to_string(),
            contexto_id: contexto.map(|s| s.to_string()),
            mensagem: "Usuário supervisor está inativo.".to_string(),
            autorizacao_id: None,
            warnings: vec![],
        });
    }

    // 2. Verificar a senha com Argon2
    let ph_parsed = match PasswordHash::new(&phash) {
        Ok(ph) => ph,
        Err(_) => return Err("Falha interna de leitura de hash do supervisor.".to_string())
    };

    let verify_ok = Argon2::default()
        .verify_password(senha_sup.as_bytes(), &ph_parsed)
        .is_ok();

    if !verify_ok {
        let _ = conn.execute(
            "INSERT INTO auditoria_operacional_local (id, usuario_id, login, tipo_evento, sucesso, mensagem, criado_em, entidade_id, modulo) VALUES (?1, ?2, ?3, 'SUPERVISOR_CREDENCIAL_INVALIDA', 0, 'Senha incorreta', ?4, ?5, ?6)",
            rusqlite::params![Uuid::new_v4().to_string(), u_id, login_sup, Utc::now().to_rfc3339(), contexto, origem]
        );

        return Ok(AutorizarOperacaoSupervisorResp {
            autorizado: false,
            supervisor_usuario_id: Some(u_id),
            supervisor_login: Some(login_sup.to_string()),
            permissao_codigo: permissao_codigo.to_string(),
            contexto_id: contexto.map(|s| s.to_string()),
            mensagem: "Senha inválida para o supervisor.".to_string(),
            autorizacao_id: None,
            warnings: vec![],
        });
    }

    // 3. Verificar a permissao do supervisor
    let stmt = "
        SELECT COUNT(*) > 0 
        FROM usuarios_perfis_local up
        INNER JOIN perfil_permissoes_local pp ON pp.perfil_id = up.perfil_id
        INNER JOIN permissoes_local p ON p.id = pp.permissao_id
        WHERE up.usuario_id = ?1 AND p.codigo = ?2
    ";
    let tem_permissao: bool = conn.query_row(stmt, rusqlite::params![u_id, permissao_codigo], |row| row.get(0)).unwrap_or(false);

    if !tem_permissao {
        let _ = conn.execute(
            "INSERT INTO auditoria_operacional_local (id, usuario_id, login, tipo_evento, sucesso, mensagem, criado_em, entidade_id, modulo) VALUES (?1, ?2, ?3, 'SUPERVISOR_AUTORIZACAO_NEGADA', 0, ?4, ?5, ?6, ?7)",
            rusqlite::params![Uuid::new_v4().to_string(), u_id, login_sup, format!("Sem permissão ({})", permissao_codigo), Utc::now().to_rfc3339(), contexto, origem]
        );
        return Ok(AutorizarOperacaoSupervisorResp {
            autorizado: false,
            supervisor_usuario_id: Some(u_id),
            supervisor_login: Some(login_sup.to_string()),
            permissao_codigo: permissao_codigo.to_string(),
            contexto_id: contexto.map(|s| s.to_string()),
            mensagem: format!("Supervisor não possui a permissão requerida: {}", permissao_codigo),
            autorizacao_id: None,
            warnings: vec![],
        });
    }

    // Autorizado!
    let auth_id = Uuid::new_v4().to_string();
    let _ = conn.execute(
        "INSERT INTO auditoria_operacional_local (id, usuario_id, login, tipo_evento, sucesso, mensagem, criado_em, entidade_id, modulo) VALUES (?1, ?2, ?3, 'SUPERVISOR_AUTORIZACAO_CONCEDIDA', 1, ?4, ?5, ?6, ?7)",
        rusqlite::params![auth_id.clone(), u_id, login_sup, format!("Liberou {}", permissao_codigo), Utc::now().to_rfc3339(), contexto, origem]
    );

    Ok(AutorizarOperacaoSupervisorResp {
        autorizado: true,
        supervisor_usuario_id: Some(u_id),
        supervisor_login: Some(login_sup.to_string()),
        permissao_codigo: permissao_codigo.to_string(),
        contexto_id: contexto.map(|s| s.to_string()),
        mensagem: "Autorizado com sucesso.".to_string(),
        autorizacao_id: Some(auth_id),
        warnings: vec![],
    })
}

pub fn garantir_permissao_ou_supervisor(
    conn: &rusqlite::Connection,
    permissao_codigo: &str,
    contexto: Option<&str>,
    origem: Option<&str>,
    motivo: Option<&str>,
    req_sup: Option<&aureon_core::dtos::AutorizarOperacaoSupervisorReq>
) -> Result<(), String> {
    // 1. Tenta com o operador atual (sem emitir erro fatal caso falhe, porque podemos cobrir com o supervisor)
    let operador_resp = avaliar_permissao_usuario(conn, permissao_codigo, contexto, origem)?;
    if operador_resp.permitido {
        // Operador tem permissão. Validar motivo se obrigatório pela regra geral do bloco.
        // O Bloco 3 pede que toda operação sensível tenha motivo, mas vamos confiar no chamador que passa o motivo ou validá-lo aqui.
        if (permissao_codigo == "ITEM_CANCELAR" || permissao_codigo == "VENDA_CANCELAR" || permissao_codigo == "DESCONTO_APLICAR") && motivo.unwrap_or("").trim().is_empty() {
            return Err("Motivo é obrigatório para esta operação.".to_string());
        }

        let _ = conn.execute(
            "INSERT INTO auditoria_operacional_local (id, usuario_id, login, tipo_evento, sucesso, mensagem, criado_em, entidade_id, modulo) VALUES (?1, ?2, ?3, 'OPERACAO_SENSIVEL_AUTORIZADA', 1, ?4, ?5, ?6, ?7)",
            rusqlite::params![Uuid::new_v4().to_string(), operador_resp.usuario_id, operador_resp.login, format!("Motivo: {}", motivo.unwrap_or("N/A")), Utc::now().to_rfc3339(), contexto, origem]
        );
        return Ok(());
    }

    // Se chegou aqui, operador não tem permissão.
    // 2. Precisamos do supervisor
    let req = req_sup.ok_or_else(|| "Operação exige permissão especial ou liberação de supervisor.".to_string())?;

    if req.motivo.as_deref().unwrap_or("").trim().is_empty() && req.motivo_obrigatorio.unwrap_or(true) {
        return Err("Motivo é obrigatório para solicitar autorização de supervisor.".to_string());
    }

    let senha = req.supervisor_senha.as_deref().unwrap_or("");
    if senha.trim().is_empty() {
        return Err("Senha do supervisor não pode ser vazia.".to_string());
    }

    let sup_resp = validar_credencial_supervisor(conn, permissao_codigo, &req.supervisor_login, senha, contexto, origem)?;
    if !sup_resp.autorizado {
        let _ = conn.execute(
            "INSERT INTO auditoria_operacional_local (id, usuario_id, login, tipo_evento, sucesso, mensagem, criado_em, entidade_id, modulo) VALUES (?1, ?2, ?3, 'OPERACAO_SENSIVEL_BLOQUEADA', 0, ?4, ?5, ?6, ?7)",
            rusqlite::params![Uuid::new_v4().to_string(), operador_resp.usuario_id, operador_resp.login, format!("Supervisor negado: {}", sup_resp.mensagem), Utc::now().to_rfc3339(), contexto, origem]
        );
        return Err(sup_resp.mensagem);
    }

    // Supervisor aprovou
    let _ = conn.execute(
        "INSERT INTO auditoria_operacional_local (id, usuario_id, login, tipo_evento, sucesso, mensagem, criado_em, entidade_id, modulo) VALUES (?1, ?2, ?3, 'OPERACAO_SENSIVEL_AUTORIZADA', 1, ?4, ?5, ?6, ?7)",
        rusqlite::params![Uuid::new_v4().to_string(), operador_resp.usuario_id, operador_resp.login, format!("Auth via SUP {}, Motivo: {}", req.supervisor_login, req.motivo.as_deref().unwrap_or("N/A")), Utc::now().to_rfc3339(), contexto, origem]
    );

    Ok(())
}

#[command]
pub fn autorizar_operacao_supervisor(
    req: aureon_core::dtos::AutorizarOperacaoSupervisorReq,
    estado: State<'_, EstadoApp>
) -> Result<AutorizarOperacaoSupervisorResp, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    
    let senha = req.supervisor_senha.as_deref().unwrap_or("");
    if senha.trim().is_empty() {
        return Ok(AutorizarOperacaoSupervisorResp {
            autorizado: false,
            supervisor_usuario_id: None,
            supervisor_login: Some(req.supervisor_login.clone()),
            permissao_codigo: req.permissao_codigo.clone(),
            contexto_id: req.contexto_id.clone(),
            mensagem: "Senha é obrigatória.".to_string(),
            autorizacao_id: None,
            warnings: vec![],
        });
    }

    validar_credencial_supervisor(
        &conn, 
        &req.permissao_codigo, 
        &req.supervisor_login, 
        senha, 
        req.contexto_id.as_deref(), 
        req.origem.as_deref()
    )
}

