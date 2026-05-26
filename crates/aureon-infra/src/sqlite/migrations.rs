use rusqlite::Connection;
use aureon_core::AureonError;
use tracing::{info, warn};

/// Migrations SQLite versionadas.
/// Cada migration é identificada por versão e nome.
struct Migration {
    versao: i32,
    nome:   &'static str,
    sql:    &'static str,
}

/// Lista de migrations em ordem de execução.
/// NUNCA alterar migrations existentes — apenas adicionar novas.
fn migrations() -> Vec<Migration> {
    vec![
        Migration {
            versao: 1,
            nome:   "schema_inicial",
            sql:    include_str!("../../../../database/migrations/sqlite/001_schema_inicial.sql"),
        },
        Migration {
            versao: 2,
            nome:   "sync_fase6",
            sql:    include_str!("../../../../database/migrations/sqlite/002_sync_fase6.sql"),
        },
        Migration {
            versao: 3,
            nome:   "venda_nucleo",
            sql:    include_str!("../../../../database/migrations/sqlite/003_venda_nucleo.sql"),
        },
        Migration {
            versao: 4,
            nome:   "venda_nucleo_correcao_financeira",
            sql:    include_str!("../../../../database/migrations/sqlite/004_venda_nucleo_correcao_financeira.sql"),
        },
        Migration {
            versao: 5,
            nome:   "pdv_operacional_fase8",
            sql:    include_str!("../../../../database/migrations/sqlite/005_pdv_operacional_fase8.sql"),
        },
        Migration {
            versao: 6,
            nome:   "pdv_operacional_fase8_cache",
            sql:    include_str!("../../../../database/migrations/sqlite/006_pdv_operacional_fase8_cache.sql"),
        },
        Migration {
            versao: 7,
            nome:   "pdv_gourmet_fase9",
            sql:    include_str!("../../../../database/migrations/sqlite/007_pdv_gourmet_fase9.sql"),
        },
        Migration {
            versao: 8,
            nome:   "fase10_delivery",
            sql:    include_str!("../../../../database/migrations/sqlite/008_fase10_delivery.sql"),
        },
        Migration {
            versao: 9,
            nome:   "fase11_estoque",
            sql:    include_str!("../../../../database/migrations/sqlite/009_fase11_estoque.sql"),
        },
        Migration {
            versao: 10,
            nome:   "fase12_compras",
            sql:    include_str!("../../../../database/migrations/sqlite/010_fase12_compras.sql"),
        },
        Migration {
            versao: 11,
            nome:   "fase13_financeiro",
            sql:    include_str!("../../../../database/migrations/sqlite/011_fase13_financeiro.sql"),
        },
        Migration {
            versao: 12,
            nome:   "fase16_fiscal_base",
            sql:    include_str!("../../../../database/migrations/sqlite/012_fase16_fiscal_base.sql"),
        },
        Migration {
            versao: 13,
            nome:   "fase17_sync_fiscal",
            sql:    include_str!("../../../../database/migrations/sqlite/013_fase17_sync_fiscal.sql"),
        },
        Migration {
            versao: 14,
            nome:   "fase20_licenciamento",
            sql:    include_str!("../../../../database/migrations/sqlite/014_fase20_licenciamento.sql"),
        },
        Migration {
            versao: 15,
            nome:   "fase21_usuarios_permissoes",
            sql:    include_str!("../../../../database/migrations/sqlite/015_fase21_usuarios_permissoes.sql"),
        },
    ]
}

/// Executa todas as migrations pendentes na ordem correta.
pub fn executar_migrations(conn: &Connection) -> Result<(), AureonError> {
    info!(
        componente = "aureon-infra::migrations",
        "Verificando migrations SQLite pendentes"
    );

    // Garante que a tabela de controle existe
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations_local (
            versao     INTEGER PRIMARY KEY,
            nome       TEXT NOT NULL,
            aplicado_em TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )
    .map_err(|e| AureonError::Migracao(e.to_string()))?;

    for m in migrations() {
        let ja_aplicada: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM schema_migrations_local WHERE versao = ?1",
                [m.versao],
                |r| r.get(0),
            )
            .unwrap_or(false);

        if ja_aplicada {
            warn!(
                componente = "aureon-infra::migrations",
                versao = m.versao,
                nome   = m.nome,
                "Migration já aplicada — pulando"
            );
            continue;
        }

        info!(
            componente = "aureon-infra::migrations",
            versao = m.versao,
            nome   = m.nome,
            "Aplicando migration SQLite"
        );

        conn.execute_batch(m.sql)
            .map_err(|e| AureonError::Migracao(format!("Falha na migration v{}: {e}", m.versao)))?;

        conn.execute(
            "INSERT INTO schema_migrations_local (versao, nome) VALUES (?1, ?2)",
            rusqlite::params![m.versao, m.nome],
        )
        .map_err(|e| AureonError::Migracao(e.to_string()))?;

        if m.versao == 15 {
            seed_fase21_usuarios(conn)?;
        }

        info!(
            componente = "aureon-infra::migrations",
            versao = m.versao,
            nome   = m.nome,
            "Migration SQLite aplicada com sucesso"
        );
    }

    Ok(())
}

fn seed_fase21_usuarios(conn: &Connection) -> Result<(), AureonError> {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
        Argon2,
    };
    use uuid::Uuid;

    let count: i32 = conn.query_row("SELECT COUNT(*) FROM perfis_local", [], |r| r.get(0)).unwrap_or(0);
    if count > 0 { return Ok(()); }

    let admin_id = Uuid::new_v4().to_string();
    let super_id = Uuid::new_v4().to_string();
    let oper_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute_batch(&format!("
        INSERT INTO perfis_local (id, codigo, nome, sistema, ativo, criado_em, atualizado_em) VALUES 
        ('{}', 'ADMIN', 'Administrador', 1, 1, '{}', '{}'),
        ('{}', 'SUPERVISOR', 'Supervisor', 1, 1, '{}', '{}'),
        ('{}', 'OPERADOR', 'Operador de Caixa', 1, 1, '{}', '{}');
    ", admin_id, now, now, super_id, now, now, oper_id, now, now)).map_err(|e| AureonError::Migracao(e.to_string()))?;

    let permissoes = vec![
        ("CAIXA_ABRIR", "CAIXA", "ABRIR", "Permite abrir caixa"),
        ("CAIXA_FECHAR", "CAIXA", "FECHAR", "Permite fechar caixa"),
        ("VENDA_CRIAR", "VENDA", "CRIAR", "Permite criar venda"),
        ("VENDA_FINALIZAR", "VENDA", "FINALIZAR", "Permite finalizar venda"),
        ("VENDA_CANCELAR", "VENDA", "CANCELAR", "Permite cancelar venda"),
        ("ITEM_CANCELAR", "VENDA", "ITEM_CANCELAR", "Permite cancelar item da venda"),
        ("DESCONTO_APLICAR", "VENDA", "DESCONTO", "Permite dar desconto"),
        ("SANGRIA_REALIZAR", "CAIXA", "SANGRIA", "Permite fazer sangria"),
        ("SUPRIMENTO_REALIZAR", "CAIXA", "SUPRIMENTO", "Permite fazer suprimento"),
        ("FINANCEIRO_ACESSAR", "FINANCEIRO", "ACESSAR", "Permite acessar financeiro"),
        ("ESTOQUE_AJUSTAR", "ESTOQUE", "AJUSTAR", "Permite ajuste de estoque"),
        ("BACKUP_RESTAURAR", "SISTEMA", "BACKUP_RESTAURAR", "Permite restaurar backup"),
        ("LICENCA_GERENCIAR", "SISTEMA", "LICENCA_GERENCIAR", "Permite gerenciar licenca"),
        ("FISCAL_PREVIEW_ACESSAR", "FISCAL", "PREVIEW", "Permite acessar dicionarios fiscais"),
        ("USUARIOS_GERENCIAR", "SISTEMA", "USUARIOS_GERENCIAR", "Permite gerenciar usuários"),
        ("AUTORIZAR_SUPERVISOR", "SISTEMA", "AUTORIZAR", "Permite autorizar bloqueios"),
    ];

    for (cod, mod_, acao, desc) in permissoes {
        let p_id = Uuid::new_v4().to_string();
        conn.execute("INSERT INTO permissoes_local (id, codigo, modulo, acao, descricao, risco, criado_em) VALUES (?1, ?2, ?3, ?4, ?5, 'NORMAL', ?6)",
            rusqlite::params![p_id, cod, mod_, acao, desc, now]).unwrap();
        
        // Admin ganha tudo
        conn.execute("INSERT INTO perfil_permissoes_local (id, perfil_id, permissao_id, permitido, criado_em) VALUES (?1, ?2, ?3, 1, ?4)",
            rusqlite::params![Uuid::new_v4().to_string(), admin_id, p_id, now]).unwrap();
    }

    let user_id = Uuid::new_v4().to_string();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(b"admin123", &salt).map_err(|e| AureonError::Migracao(e.to_string()))?.to_string();

    conn.execute("INSERT INTO usuarios_local (id, nome, login, senha_hash, senha_algoritmo, ativo, exige_troca_senha, criado_em, atualizado_em) VALUES (?1, 'Administrador DEV', 'admin', ?2, 'ARGON2ID', 1, 1, ?3, ?4)",
        rusqlite::params![user_id, hash, now, now]).unwrap();

    conn.execute("INSERT INTO usuario_perfis_local (id, usuario_id, perfil_id, criado_em) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![Uuid::new_v4().to_string(), user_id, admin_id, now]).unwrap();

    info!(componente = "aureon-infra::migrations", "Seed base de usuários aplicado (admin/admin123)");
    Ok(())
}
