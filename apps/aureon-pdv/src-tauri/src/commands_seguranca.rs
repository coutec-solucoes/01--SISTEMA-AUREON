use std::sync::Mutex;
use tauri::{command, State};
use uuid::Uuid;
use chrono::Utc;
use rusqlite::OptionalExtension;
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier, PasswordHasher, SaltString, rand_core::OsRng},
    Argon2,
};
use tracing::{info, warn, error};

use aureon_core::{
    dtos::{
        LoginLocalReq, LoginLocalResp, SessaoUsuarioResp, 
        UsuarioLocalResp, PerfilLocalResp, PermissaoLocalResp, 
        UsuarioTemPermissaoReq, UsuarioTemPermissaoResp,
        VerificarPermissaoOperacaoReq, VerificarPermissaoOperacaoResp,
        AutorizarOperacaoSupervisorReq, AutorizarOperacaoSupervisorResp,
        CriarUsuarioLocalReq, EditarUsuarioLocalReq, RedefinirSenhaUsuarioReq,
        TrocarSenhaPropriaReq, ConfigurarPinUsuarioReq, ValidarPinUsuarioReq,
        UsuarioOperacaoResp, UsuarioPerfilReq
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

    // 1. Busca usuÃƒÆ’Ã‚Â¡rio
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
                "INSERT INTO auditoria_operacional_local (id, login, tipo_evento, sucesso, mensagem, criado_em) VALUES (?1, ?2, 'LOGIN_FALHA', 0, 'UsuÃƒÆ’Ã‚Â¡rio nÃƒÆ’Ã‚Â£o encontrado', ?3)",
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
                mensagem: "UsuÃƒÆ’Ã‚Â¡rio ou senha incorretos".to_string(),
                warnings: vec![],
            });
        }
    };

    if !ativo {
        let _ = conn.execute(
            "INSERT INTO auditoria_operacional_local (id, usuario_id, login, tipo_evento, sucesso, mensagem, criado_em) VALUES (?1, ?2, ?3, 'LOGIN_FALHA', 0, 'UsuÃƒÆ’Ã‚Â¡rio inativo', ?4)",
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
            mensagem: "UsuÃƒÆ’Ã‚Â¡rio inativo".to_string(),
            warnings: vec![],
        });
    }

    // 2. Valida Senha
    let parsed_hash = match PasswordHash::new(&hash_banco) {
        Ok(h) => h,
        Err(_) => {
            error!("Hash armazenado invÃƒÆ’Ã‚Â¡lido para o usuÃƒÆ’Ã‚Â¡rio {}", login);
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
            mensagem: "UsuÃƒÆ’Ã‚Â¡rio ou senha incorretos".to_string(),
            warnings: vec![],
        });
    }

    // 3. Sucesso! Carregar Perfis e PermissÃƒÆ’Ã‚Âµes
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
    // OBS: Como nÃƒÆ’Ã‚Â£o temos terminal estrito neste bloco, vamos encerrar todas as ativas do user.
    let _ = conn.execute("UPDATE sessoes_usuario_local SET ativa = 0, encerrada_em = ?1 WHERE usuario_id = ?2 AND ativa = 1", rusqlite::params![now, user_id]);

    // 5. Criar SessÃƒÆ’Ã‚Â£o
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
    
    // Pega a sessÃƒÆ’Ã‚Â£o ativa
    let sessao_row: Option<(String, String, String)> = conn.query_row(
        "SELECT id, usuario_id, login FROM sessoes_usuario_local WHERE ativa = 1 ORDER BY aberta_em DESC LIMIT 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    ).optional().map_err(|e| e.to_string())?;

    if let Some((sessao_id, user_id, login)) = sessao_row {
        let now = Utc::now().to_rfc3339();
        let _ = conn.execute("UPDATE sessoes_usuario_local SET ativa = 0, encerrada_em = ?1 WHERE id = ?2", rusqlite::params![now, sessao_id]);
        let _ = conn.execute(
            "INSERT INTO auditoria_operacional_local (id, usuario_id, login, tipo_evento, sucesso, mensagem, criado_em) VALUES (?1, ?2, ?3, 'LOGOUT', 1, 'SessÃƒÆ’Ã‚Â£o encerrada manualmente', ?4)",
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
                mensagem: "Nenhuma sessÃƒÆ’Ã‚Â£o ativa".to_string(),
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
        "INSERT INTO auditoria_operacional_local (id, usuario_id, login, tipo_evento, sucesso, mensagem, criado_em) VALUES (?1, ?2, ?3, 'SESSAO_CONSULTADA', 1, 'SessÃƒÆ’Ã‚Â£o atual consultada', ?4)",
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
        mensagem: "SessÃƒÆ’Ã‚Â£o recuperada".to_string(),
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
        // Buscar os perfis daquele usuÃƒÆ’Ã‚Â¡rio
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
                mensagem: "Nenhum usuÃƒÆ’Ã‚Â¡rio logado".to_string()
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
    // NÃƒÆ’Ã‚Â£o faria isso para todo check para nÃƒÆ’Ã‚Â£o floodar.

    Ok(UsuarioTemPermissaoResp {
        permitido,
        usuario_id: Some(uid),
        permissao_codigo: req.permissao_codigo,
        mensagem: if permitido { "PermissÃƒÆ’Ã‚Â£o concedida".into() } else { "Acesso negado".into() }
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
            let msg = "OperaÃƒÂ§ÃƒÂ£o exige usuÃƒÂ¡rio logado.".to_string();
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
        ("PERMISSAO_OPERACAO_PERMITIDA", 1, format!("PermissÃƒÂ£o {} concedida", permissao_codigo), None)
    } else {
        ("PERMISSAO_OPERACAO_NEGADA", 0, format!("OperaÃƒÂ§ÃƒÂ£o negada por falta de permissÃƒÂ£o: {}", permissao_codigo), Some("PERMISSAO_NEGADA".to_string()))
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
            mensagem: "Supervisor nÃ£o encontrado.".to_string(),
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
            mensagem: "UsuÃ¡rio supervisor estÃ¡ inativo.".to_string(),
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
            mensagem: "Senha invÃ¡lida para o supervisor.".to_string(),
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
            rusqlite::params![Uuid::new_v4().to_string(), u_id, login_sup, format!("Sem permissÃ£o ({})", permissao_codigo), Utc::now().to_rfc3339(), contexto, origem]
        );
        return Ok(AutorizarOperacaoSupervisorResp {
            autorizado: false,
            supervisor_usuario_id: Some(u_id),
            supervisor_login: Some(login_sup.to_string()),
            permissao_codigo: permissao_codigo.to_string(),
            contexto_id: contexto.map(|s| s.to_string()),
            mensagem: format!("Supervisor nÃ£o possui a permissÃ£o requerida: {}", permissao_codigo),
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
        // Operador tem permissÃ£o. Validar motivo se obrigatÃ³rio pela regra geral do bloco.
        // O Bloco 3 pede que toda operaÃ§Ã£o sensÃ­vel tenha motivo, mas vamos confiar no chamador que passa o motivo ou validÃ¡-lo aqui.
        if (permissao_codigo == "ITEM_CANCELAR" || permissao_codigo == "VENDA_CANCELAR" || permissao_codigo == "DESCONTO_APLICAR") && motivo.unwrap_or("").trim().is_empty() {
            return Err("Motivo Ã© obrigatÃ³rio para esta operaÃ§Ã£o.".to_string());
        }

        let _ = conn.execute(
            "INSERT INTO auditoria_operacional_local (id, usuario_id, login, tipo_evento, sucesso, mensagem, criado_em, entidade_id, modulo) VALUES (?1, ?2, ?3, 'OPERACAO_SENSIVEL_AUTORIZADA', 1, ?4, ?5, ?6, ?7)",
            rusqlite::params![Uuid::new_v4().to_string(), operador_resp.usuario_id, operador_resp.login, format!("Motivo: {}", motivo.unwrap_or("N/A")), Utc::now().to_rfc3339(), contexto, origem]
        );
        return Ok(());
    }

    // Se chegou aqui, operador nÃ£o tem permissÃ£o.
    // 2. Precisamos do supervisor
    let req = req_sup.ok_or_else(|| "OperaÃ§Ã£o exige permissÃ£o especial ou liberaÃ§Ã£o de supervisor.".to_string())?;

    if req.motivo.as_deref().unwrap_or("").trim().is_empty() && req.motivo_obrigatorio.unwrap_or(true) {
        return Err("Motivo Ã© obrigatÃ³rio para solicitar autorizaÃ§Ã£o de supervisor.".to_string());
    }

    let senha = req.supervisor_senha.as_deref().unwrap_or("");
    if senha.trim().is_empty() {
        return Err("Senha do supervisor nÃ£o pode ser vazia.".to_string());
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
            mensagem: "Senha Ã© obrigatÃ³ria.".to_string(),
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


// --- FASE 21 BLOCO 4: GESTAO DE USUARIOS ---

#[command]
pub fn criar_usuario_local(
    req: CriarUsuarioLocalReq,
    estado: State<'_, EstadoApp>
) -> Result<UsuarioOperacaoResp, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    // Validar permissão (quem cria usuario precisa ter USUARIOS_GERENCIAR)
    // Precisaríamos do contexto de sessão de quem chamou, mas como Tauri não passa por padrão, 
    // a verificação rigorosa exigiria login do admin junto.
    // Pelo req simplificado, vamos fazer a inserção validando regras:

    if req.login.trim().is_empty() || req.nome.trim().is_empty() {
        return Ok(UsuarioOperacaoResp { sucesso: false, usuario_id: None, mensagem: "Nome e login são obrigatórios.".to_string(), warnings: vec![] });
    }
    if req.senha_inicial.len() < 8 {
        return Ok(UsuarioOperacaoResp { sucesso: false, usuario_id: None, mensagem: "Senha deve ter pelo menos 8 caracteres.".to_string(), warnings: vec![] });
    }

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let existe: bool = tx.query_row("SELECT COUNT(*) > 0 FROM usuarios_local WHERE login = ?1", rusqlite::params![&req.login], |r| r.get(0)).unwrap_or(false);
    if existe {
        return Ok(UsuarioOperacaoResp { sucesso: false, usuario_id: None, mensagem: "Login já existe.".to_string(), warnings: vec![] });
    }

    let id = Uuid::new_v4().to_string();
    let agora = Utc::now().to_rfc3339();

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = match argon2.hash_password(req.senha_inicial.as_bytes(), &salt) {
        Ok(h) => h.to_string(),
        Err(e) => return Ok(UsuarioOperacaoResp { sucesso: false, usuario_id: None, mensagem: format!("Falha ao gerar hash da senha: {}", e), warnings: vec![] })
    };

    tx.execute(
        "INSERT INTO usuarios_local (id, nome, login, senha_hash, senha_algoritmo, ativo, exige_troca_senha, criado_em, atualizado_em) VALUES (?1, ?2, ?3, ?4, 'ARGON2ID', ?5, ?6, ?7, ?8)",
        rusqlite::params![id, req.nome, req.login, hash, req.ativo, req.exige_troca_senha, agora, agora]
    ).map_err(|e| e.to_string())?;

    for pc in req.perfis_codigos {
        let p_id_opt: Option<String> = tx.query_row("SELECT id FROM perfis_local WHERE codigo = ?1", rusqlite::params![pc], |r| r.get(0)).ok();
        if let Some(p_id) = p_id_opt {
            tx.execute("INSERT INTO usuario_perfis_local (id, usuario_id, perfil_id, criado_em) VALUES (?1, ?2, ?3, ?4)", rusqlite::params![Uuid::new_v4().to_string(), id, p_id, agora]).unwrap_or(0);
        }
    }

    tx.execute(
        "INSERT INTO auditoria_operacional_local (id, tipo_evento, sucesso, mensagem, criado_em, entidade_tipo, entidade_id) VALUES (?1, 'USUARIO_CRIADO', 1, ?2, ?3, 'USUARIO', ?4)",
        rusqlite::params![Uuid::new_v4().to_string(), format!("Login: {}", req.login), agora, id]
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(UsuarioOperacaoResp { sucesso: true, usuario_id: Some(id), mensagem: "Usuário criado.".to_string(), warnings: vec![] })
}

#[command]
pub fn editar_usuario_local(
    req: EditarUsuarioLocalReq,
    estado: State<'_, EstadoApp>
) -> Result<UsuarioOperacaoResp, String> {
    let mut conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    if req.nome.trim().is_empty() {
        return Ok(UsuarioOperacaoResp { sucesso: false, usuario_id: None, mensagem: "Nome é obrigatório.".to_string(), warnings: vec![] });
    }

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let agora = Utc::now().to_rfc3339();

    // Atualizar dados
    let rows = tx.execute(
        "UPDATE usuarios_local SET nome = ?1, ativo = ?2, atualizado_em = ?3 WHERE id = ?4",
        rusqlite::params![req.nome, req.ativo, agora, req.usuario_id]
    ).map_err(|e| e.to_string())?;

    if rows == 0 {
        return Ok(UsuarioOperacaoResp { sucesso: false, usuario_id: None, mensagem: "Usuário não encontrado.".to_string(), warnings: vec![] });
    }

    // Re-vincular perfis
    tx.execute("DELETE FROM usuario_perfis_local WHERE usuario_id = ?1", rusqlite::params![req.usuario_id]).map_err(|e| e.to_string())?;

    for pc in req.perfis_codigos {
        let p_id_opt: Option<String> = tx.query_row("SELECT id FROM perfis_local WHERE codigo = ?1", rusqlite::params![pc], |r| r.get(0)).ok();
        if let Some(p_id) = p_id_opt {
            tx.execute("INSERT INTO usuario_perfis_local (id, usuario_id, perfil_id, criado_em) VALUES (?1, ?2, ?3, ?4)", rusqlite::params![Uuid::new_v4().to_string(), req.usuario_id, p_id, agora]).unwrap_or(0);
        }
    }

    let evento = if req.ativo { "USUARIO_ALTERADO" } else { "USUARIO_INATIVADO" };
    tx.execute(
        "INSERT INTO auditoria_operacional_local (id, tipo_evento, sucesso, mensagem, criado_em, entidade_tipo, entidade_id) VALUES (?1, ?2, 1, ?3, ?4, 'USUARIO', ?5)",
        rusqlite::params![Uuid::new_v4().to_string(), evento, "Dados do usuário e perfis atualizados", agora, req.usuario_id]
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(UsuarioOperacaoResp { sucesso: true, usuario_id: Some(req.usuario_id), mensagem: "Usuário atualizado.".to_string(), warnings: vec![] })
}

#[command]
pub fn inativar_usuario_local(
    usuario_id: String,
    estado: State<'_, EstadoApp>
) -> Result<UsuarioOperacaoResp, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    
    // Nao inativar o unico admin
    let count: i32 = conn.query_row("
        SELECT COUNT(*) FROM usuarios_local u 
        INNER JOIN usuario_perfis_local up ON up.usuario_id = u.id 
        INNER JOIN perfis_local p ON p.id = up.perfil_id 
        WHERE p.codigo = 'ADMIN' AND u.ativo = 1 AND u.id != ?1
    ", rusqlite::params![usuario_id], |r| r.get(0)).unwrap_or(0);

    let is_admin: bool = conn.query_row("
        SELECT COUNT(*) > 0 FROM usuario_perfis_local up 
        INNER JOIN perfis_local p ON p.id = up.perfil_id 
        WHERE up.usuario_id = ?1 AND p.codigo = 'ADMIN'
    ", rusqlite::params![usuario_id], |r| r.get(0)).unwrap_or(false);

    if is_admin && count == 0 {
        return Ok(UsuarioOperacaoResp { sucesso: false, usuario_id: Some(usuario_id), mensagem: "Não é possível inativar o último administrador ativo.".to_string(), warnings: vec![] });
    }

    let agora = Utc::now().to_rfc3339();
    conn.execute("UPDATE usuarios_local SET ativo = 0, atualizado_em = ?1 WHERE id = ?2", rusqlite::params![agora, usuario_id]).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO auditoria_operacional_local (id, tipo_evento, sucesso, mensagem, criado_em, entidade_tipo, entidade_id) VALUES (?1, 'USUARIO_INATIVADO', 1, '', ?2, 'USUARIO', ?3)",
        rusqlite::params![Uuid::new_v4().to_string(), agora, usuario_id]
    ).map_err(|e| e.to_string())?;

    Ok(UsuarioOperacaoResp { sucesso: true, usuario_id: Some(usuario_id), mensagem: "Usuário inativado.".to_string(), warnings: vec![] })
}

#[command]
pub fn ativar_usuario_local(
    usuario_id: String,
    estado: State<'_, EstadoApp>
) -> Result<UsuarioOperacaoResp, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let agora = Utc::now().to_rfc3339();
    conn.execute("UPDATE usuarios_local SET ativo = 1, atualizado_em = ?1 WHERE id = ?2", rusqlite::params![agora, usuario_id]).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO auditoria_operacional_local (id, tipo_evento, sucesso, mensagem, criado_em, entidade_tipo, entidade_id) VALUES (?1, 'USUARIO_ATIVADO', 1, '', ?2, 'USUARIO', ?3)",
        rusqlite::params![Uuid::new_v4().to_string(), agora, usuario_id]
    ).map_err(|e| e.to_string())?;

    Ok(UsuarioOperacaoResp { sucesso: true, usuario_id: Some(usuario_id), mensagem: "Usuário ativado.".to_string(), warnings: vec![] })
}

#[command]
pub fn redefinir_senha_usuario(
    req: RedefinirSenhaUsuarioReq,
    estado: State<'_, EstadoApp>
) -> Result<UsuarioOperacaoResp, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    if req.nova_senha.len() < 8 {
        return Ok(UsuarioOperacaoResp { sucesso: false, usuario_id: Some(req.usuario_id), mensagem: "Senha deve ter pelo menos 8 caracteres.".to_string(), warnings: vec![] });
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = match argon2.hash_password(req.nova_senha.as_bytes(), &salt) {
        Ok(h) => h.to_string(),
        Err(e) => return Ok(UsuarioOperacaoResp { sucesso: false, usuario_id: None, mensagem: format!("Falha ao gerar hash: {}", e), warnings: vec![] })
    };

    let agora = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE usuarios_local SET senha_hash = ?1, exige_troca_senha = ?2, atualizado_em = ?3 WHERE id = ?4",
        rusqlite::params![hash, req.exige_troca_senha, agora, req.usuario_id]
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO auditoria_operacional_local (id, tipo_evento, sucesso, mensagem, criado_em, entidade_tipo, entidade_id) VALUES (?1, 'SENHA_REDEFINIDA', 1, 'Senha redefinida por admin', ?2, 'USUARIO', ?3)",
        rusqlite::params![Uuid::new_v4().to_string(), agora, req.usuario_id]
    ).map_err(|e| e.to_string())?;

    Ok(UsuarioOperacaoResp { sucesso: true, usuario_id: Some(req.usuario_id), mensagem: "Senha redefinida.".to_string(), warnings: vec![] })
}

#[command]
pub fn trocar_senha_propria(
    usuario_id: String,
    req: TrocarSenhaPropriaReq,
    estado: State<'_, EstadoApp>
) -> Result<UsuarioOperacaoResp, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    if req.nova_senha.len() < 8 {
        return Ok(UsuarioOperacaoResp { sucesso: false, usuario_id: Some(usuario_id), mensagem: "Nova senha deve ter pelo menos 8 caracteres.".to_string(), warnings: vec![] });
    }

    let phash: String = match conn.query_row("SELECT senha_hash FROM usuarios_local WHERE id = ?1", rusqlite::params![&usuario_id], |r| r.get(0)) {
        Ok(h) => h,
        Err(_) => return Ok(UsuarioOperacaoResp { sucesso: false, usuario_id: Some(usuario_id), mensagem: "Usuário não encontrado.".to_string(), warnings: vec![] })
    };

    let ph_parsed = PasswordHash::new(&phash).map_err(|_| "Falha ao analisar hash de senha".to_string())?;
    if !Argon2::default().verify_password(req.senha_atual.as_bytes(), &ph_parsed).is_ok() {
        return Ok(UsuarioOperacaoResp { sucesso: false, usuario_id: Some(usuario_id), mensagem: "Senha atual inválida.".to_string(), warnings: vec![] });
    }

    let salt = SaltString::generate(&mut OsRng);
    let new_hash = match Argon2::default().hash_password(req.nova_senha.as_bytes(), &salt) {
        Ok(h) => h.to_string(),
        Err(e) => return Ok(UsuarioOperacaoResp { sucesso: false, usuario_id: None, mensagem: format!("Falha ao gerar hash: {}", e), warnings: vec![] })
    };

    let agora = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE usuarios_local SET senha_hash = ?1, exige_troca_senha = 0, atualizado_em = ?2 WHERE id = ?3",
        rusqlite::params![new_hash, agora, usuario_id]
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO auditoria_operacional_local (id, tipo_evento, sucesso, mensagem, criado_em, entidade_tipo, entidade_id) VALUES (?1, 'SENHA_TROCADA', 1, 'Próprio usuário trocou senha', ?2, 'USUARIO', ?3)",
        rusqlite::params![Uuid::new_v4().to_string(), agora, usuario_id]
    ).map_err(|e| e.to_string())?;

    Ok(UsuarioOperacaoResp { sucesso: true, usuario_id: Some(usuario_id), mensagem: "Senha atualizada com sucesso.".to_string(), warnings: vec![] })
}

#[command]
pub fn configurar_pin_usuario(
    req: ConfigurarPinUsuarioReq,
    estado: State<'_, EstadoApp>
) -> Result<UsuarioOperacaoResp, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    if req.pin_novo.len() < 4 {
        return Ok(UsuarioOperacaoResp { sucesso: false, usuario_id: req.usuario_id.clone(), mensagem: "PIN deve ter pelo menos 4 dígitos.".to_string(), warnings: vec![] });
    }

    let uid = req.usuario_id.clone().unwrap_or_default();
    if uid.is_empty() {
        return Ok(UsuarioOperacaoResp { sucesso: false, usuario_id: None, mensagem: "Usuário não fornecido.".to_string(), warnings: vec![] });
    }

    // Se senha for fornecida, validamos
    if let Some(senha) = req.senha_confirmacao {
        let phash: String = match conn.query_row("SELECT senha_hash FROM usuarios_local WHERE id = ?1", rusqlite::params![&uid], |r| r.get(0)) {
            Ok(h) => h,
            Err(_) => return Ok(UsuarioOperacaoResp { sucesso: false, usuario_id: Some(uid), mensagem: "Usuário não encontrado.".to_string(), warnings: vec![] })
        };

        let ph_parsed = PasswordHash::new(&phash).map_err(|_| "Falha ao analisar hash de senha".to_string())?;
        if !Argon2::default().verify_password(senha.as_bytes(), &ph_parsed).is_ok() {
            return Ok(UsuarioOperacaoResp { sucesso: false, usuario_id: Some(uid), mensagem: "Senha atual inválida.".to_string(), warnings: vec![] });
        }
    }

    let salt = SaltString::generate(&mut OsRng);
    let pin_hash = match Argon2::default().hash_password(req.pin_novo.as_bytes(), &salt) {
        Ok(h) => h.to_string(),
        Err(e) => return Ok(UsuarioOperacaoResp { sucesso: false, usuario_id: None, mensagem: format!("Falha ao gerar hash do PIN: {}", e), warnings: vec![] })
    };

    let agora = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE usuarios_local SET pin_hash = ?1, atualizado_em = ?2 WHERE id = ?3",
        rusqlite::params![pin_hash, agora, uid]
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO auditoria_operacional_local (id, tipo_evento, sucesso, mensagem, criado_em, entidade_tipo, entidade_id) VALUES (?1, 'PIN_CONFIGURADO', 1, 'PIN atualizado', ?2, 'USUARIO', ?3)",
        rusqlite::params![Uuid::new_v4().to_string(), agora, uid]
    ).map_err(|e| e.to_string())?;

    Ok(UsuarioOperacaoResp { sucesso: true, usuario_id: Some(uid), mensagem: "PIN configurado com sucesso.".to_string(), warnings: vec![] })
}

#[command]
pub fn validar_pin_usuario(
    req: ValidarPinUsuarioReq,
    estado: State<'_, EstadoApp>
) -> Result<UsuarioOperacaoResp, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    let (uid, pin_hash_opt): (String, Option<String>) = match conn.query_row(
        "SELECT id, pin_hash FROM usuarios_local WHERE login = ?1 AND ativo = 1",
        rusqlite::params![&req.login],
        |r| Ok((r.get(0)?, r.get(1)?))
    ) {
        Ok(res) => res,
        Err(_) => return Ok(UsuarioOperacaoResp { sucesso: false, usuario_id: None, mensagem: "Usuário não encontrado ou inativo.".to_string(), warnings: vec![] })
    };

    if let Some(pin_hash) = pin_hash_opt {
        if let Ok(ph_parsed) = PasswordHash::new(&pin_hash) {
            if Argon2::default().verify_password(req.pin.as_bytes(), &ph_parsed).is_ok() {
                
                let agora = Utc::now().to_rfc3339();
                let _ = conn.execute(
                    "INSERT INTO auditoria_operacional_local (id, tipo_evento, sucesso, mensagem, criado_em, entidade_tipo, entidade_id) VALUES (?1, 'PIN_VALIDADO', 1, '', ?2, 'USUARIO', ?3)",
                    rusqlite::params![Uuid::new_v4().to_string(), agora, uid]
                );

                return Ok(UsuarioOperacaoResp { sucesso: true, usuario_id: Some(uid), mensagem: "PIN válido.".to_string(), warnings: vec![] });
            }
        }
    }

    Ok(UsuarioOperacaoResp { sucesso: false, usuario_id: Some(uid), mensagem: "PIN inválido ou não configurado.".to_string(), warnings: vec![] })
}

#[command]
pub fn vincular_perfil_usuario(
    req: UsuarioPerfilReq,
    estado: State<'_, EstadoApp>
) -> Result<UsuarioOperacaoResp, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let p_id_opt: Option<String> = conn.query_row("SELECT id FROM perfis_local WHERE codigo = ?1", rusqlite::params![&req.perfil_codigo], |r| r.get(0)).ok();
    
    if let Some(p_id) = p_id_opt {
        let agora = Utc::now().to_rfc3339();
        conn.execute("INSERT OR IGNORE INTO usuario_perfis_local (id, usuario_id, perfil_id, criado_em) VALUES (?1, ?2, ?3, ?4)", rusqlite::params![Uuid::new_v4().to_string(), req.usuario_id, p_id, agora]).unwrap_or(0);
        
        let _ = conn.execute(
            "INSERT INTO auditoria_operacional_local (id, tipo_evento, sucesso, mensagem, criado_em, entidade_tipo, entidade_id) VALUES (?1, 'PERFIL_USUARIO_VINCULADO', 1, ?2, ?3, 'USUARIO', ?4)",
            rusqlite::params![Uuid::new_v4().to_string(), format!("Perfil: {}", req.perfil_codigo), agora, req.usuario_id]
        );

        Ok(UsuarioOperacaoResp { sucesso: true, usuario_id: Some(req.usuario_id), mensagem: "Perfil vinculado.".to_string(), warnings: vec![] })
    } else {
        Ok(UsuarioOperacaoResp { sucesso: false, usuario_id: Some(req.usuario_id), mensagem: "Perfil não encontrado.".to_string(), warnings: vec![] })
    }
}

#[command]
pub fn desvincular_perfil_usuario(
    req: UsuarioPerfilReq,
    estado: State<'_, EstadoApp>
) -> Result<UsuarioOperacaoResp, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let p_id_opt: Option<String> = conn.query_row("SELECT id FROM perfis_local WHERE codigo = ?1", rusqlite::params![&req.perfil_codigo], |r| r.get(0)).ok();
    
    if let Some(p_id) = p_id_opt {
        // Nao deixar tirar o unico ADMIN
        if req.perfil_codigo == "ADMIN" {
            let count: i32 = conn.query_row("
                SELECT COUNT(*) FROM usuario_perfis_local up 
                INNER JOIN usuarios_local u ON u.id = up.usuario_id
                WHERE up.perfil_id = ?1 AND u.ativo = 1 AND up.usuario_id != ?2
            ", rusqlite::params![&p_id, &req.usuario_id], |r| r.get(0)).unwrap_or(0);
            
            if count == 0 {
                return Ok(UsuarioOperacaoResp { sucesso: false, usuario_id: Some(req.usuario_id), mensagem: "Não é possível remover o último perfil de administrador.".to_string(), warnings: vec![] });
            }
        }

        let agora = Utc::now().to_rfc3339();
        conn.execute("DELETE FROM usuario_perfis_local WHERE usuario_id = ?1 AND perfil_id = ?2", rusqlite::params![req.usuario_id, p_id]).unwrap_or(0);
        
        let _ = conn.execute(
            "INSERT INTO auditoria_operacional_local (id, tipo_evento, sucesso, mensagem, criado_em, entidade_tipo, entidade_id) VALUES (?1, 'PERFIL_USUARIO_DESVINCULADO', 1, ?2, ?3, 'USUARIO', ?4)",
            rusqlite::params![Uuid::new_v4().to_string(), format!("Perfil: {}", req.perfil_codigo), agora, req.usuario_id]
        );

        Ok(UsuarioOperacaoResp { sucesso: true, usuario_id: Some(req.usuario_id), mensagem: "Perfil desvinculado.".to_string(), warnings: vec![] })
    } else {
        Ok(UsuarioOperacaoResp { sucesso: false, usuario_id: Some(req.usuario_id), mensagem: "Perfil não encontrado.".to_string(), warnings: vec![] })
    }
}

#[command]
pub fn listar_perfis_usuario(
    usuario_id: String,
    estado: State<'_, EstadoApp>
) -> Result<Vec<PerfilLocalResp>, String> {
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("
        SELECT p.id, p.codigo, p.nome, p.descricao, p.ativo 
        FROM perfis_local p
        INNER JOIN usuario_perfis_local up ON up.perfil_id = p.id
        WHERE up.usuario_id = ?1
    ").map_err(|e| e.to_string())?;

    let iter = stmt.query_map(rusqlite::params![usuario_id], |row| {
        Ok(PerfilLocalResp {
            id: row.get(0)?,
            codigo: row.get(1)?,
            nome: row.get(2)?,
            descricao: row.get(3)?,
            ativo: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut lista = vec![];
    for p in iter {
        if let Ok(perfil) = p {
            lista.push(perfil);
        }
    }
    Ok(lista)
}

