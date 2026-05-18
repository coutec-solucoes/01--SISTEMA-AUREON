use aureon_core::{RespostaBase, AureonError};
use sqlx::{postgres::PgConnectOptions, ConnectOptions, Connection};

// ================================================================
// DTOs PARA CONFIGURAÇÃO
// ================================================================

#[derive(serde::Deserialize)]
pub struct ConfigPostgresDto {
    pub host: String,
    pub porta: u16,
    pub usuario: String,
    pub senha_plana: String,
}

#[derive(serde::Serialize)]
pub struct StatusConexaoDto {
    pub conectado: bool,
    pub permissao_create_db: bool,
    pub mensagem: String,
}

// ================================================================
// COMMANDS DE CONFIGURAÇÃO (FASE 1)
// ================================================================

/// Teste real de conexão PostgreSQL com validação de privilégios CREATEDB ou Superuser
#[tauri::command]
pub async fn testar_postgres(
    dados: ConfigPostgresDto,
) -> Result<RespostaBase<StatusConexaoDto>, AureonError> {
    let options = PgConnectOptions::new()
        .host(&dados.host)
        .port(dados.porta)
        .username(&dados.usuario)
        .password(&dados.senha_plana)
        .database("postgres")
        .disable_statement_logging(); // banco root genérico para teste

    let mut conn = match sqlx::PgConnection::connect_with(&options).await {
        Ok(c) => c,
        Err(_) => return Ok(RespostaBase::falha_manual(
            "Falha de autenticação",
            "ERRO_CONEXAO_POSTGRESQL",
            "Não foi possível conectar ao PostgreSQL. Verifique host, porta, usuário e senha.",
        )),
    };

    // Valida se o usuário pode criar banco (CREATEDB ou usesuper)
    let query = "
        SELECT rolcreatedb, rolsuper 
        FROM pg_roles 
        WHERE rolname = $1
    ";

    let row: (bool, bool) = sqlx::query_as(query)
        .bind(&dados.usuario)
        .fetch_one(&mut conn)
        .await
        .unwrap_or((false, false));

    let tem_permissao = row.0 || row.1;

    if tem_permissao {
        Ok(RespostaBase::ok(
            "Conexão realizada com sucesso. Usuário possui privilégios para criar o banco.",
            StatusConexaoDto {
                conectado: true,
                permissao_create_db: true,
                mensagem: "Conexão OK. Permissão validada.".to_string(),
            }
        ))
    } else {
        Ok(RespostaBase::falha_manual(
            "Permissão insuficiente",
            "ERRO_PERMISSAO_CREATEDB",
            "Usuário conectado não possui permissão CREATEDB ou Superuser. O AUREON Config precisa dessa permissão para criar o banco da empresa.",
        ))
    }
}

fn normalizar_nome_banco(nome: &str) -> String {
    let db_name = nome.to_lowercase()
        .replace("á", "a").replace("à", "a").replace("ã", "a").replace("â", "a")
        .replace("é", "e").replace("ê", "e")
        .replace("í", "i")
        .replace("ó", "o").replace("õ", "o").replace("ô", "o")
        .replace("ú", "u")
        .replace("ç", "c")
        .replace(" ", "_");

    let mut safe_name = db_name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect::<String>();

    if safe_name.is_empty() {
        safe_name = "default_empresa".to_string();
    }

    safe_name.truncate(40); // Limita tamanho seguro no PostgreSQL

    if !safe_name.ends_with("_bd") {
        safe_name.push_str("_bd");
    }

    safe_name
}

/// Cria o banco de dados da empresa no PostgreSQL
#[tauri::command]
pub async fn criar_banco_empresa(
    dados: ConfigPostgresDto,
    nome_empresa: String,
) -> Result<RespostaBase<String>, AureonError> {
    let db_name = normalizar_nome_banco(&nome_empresa);

    let options = PgConnectOptions::new()
        .host(&dados.host)
        .port(dados.porta)
        .username(&dados.usuario)
        .password(&dados.senha_plana)
        .database("postgres")
        .disable_statement_logging();

    let mut conn = sqlx::PgConnection::connect_with(&options).await
        .map_err(|_| AureonError::ConexaoPostgres("Não foi possível conectar ao servidor PostgreSQL".into()))?;

    // Verifica se já existe
    let existe: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT datname FROM pg_catalog.pg_database WHERE datname = $1)")
        .bind(&db_name)
        .fetch_one(&mut conn)
        .await
        .unwrap_or((false,));

    if existe.0 {
        return Ok(RespostaBase::falha_manual(
            "Banco já existe",
            "ERRO_BANCO_EXISTENTE",
            format!("O banco '{}' já existe no PostgreSQL. Não iremos sobrescrever.", db_name),
        ));
    }

    // CREATE DATABASE não suporta prepared statements ($1), precisa ser injetado formatado.
    // O filtro alfanumérico acima já garante segurança contra SQL Injection básica.
    let create_sql = format!("CREATE DATABASE {}", db_name);
    
    // Executa
    sqlx::query(&create_sql)
        .execute(&mut conn)
        .await
        .map_err(|e| AureonError::ConexaoPostgres(format!("Falha ao criar o banco: {}", e)))?;

    Ok(RespostaBase::ok(
        "Banco de dados criado com sucesso",
        db_name,
    ))
}

#[derive(serde::Deserialize)]
pub struct InstalacaoCompletaDto {
    pub db: ConfigPostgresDto,
    pub nome_empresa: String,
    pub admin_nome: String,
    pub admin_email: String,
    pub admin_senha_plana: String,
}

#[derive(serde::Serialize)]
pub struct AppServerConfig {
    pub postgres_host: String,
    pub postgres_porta: u16,
    pub postgres_usuario: String,
    pub postgres_senha: String,
    pub postgres_banco: String,
}

const MIGRATIONS_PG: [(&str, &str); 3] = [
    ("001_schema_inicial", include_str!("../../../../database/migrations/postgresql/001_schema_inicial.sql")),
    ("002_tabelas_fase1", include_str!("../../../../database/migrations/postgresql/002_tabelas_fase1.sql")),
    ("003_seeds_iniciais", include_str!("../../../../database/migrations/postgresql/003_seeds_iniciais.sql")),
];

#[tauri::command]
pub async fn finalizar_instalacao(
    dados: InstalacaoCompletaDto,
) -> Result<RespostaBase<bool>, AureonError> {
    let db_name = normalizar_nome_banco(&dados.nome_empresa);

    let options = PgConnectOptions::new()
        .host(&dados.db.host)
        .port(dados.db.porta)
        .username(&dados.db.usuario)
        .password(&dados.db.senha_plana)
        .database(&db_name)
        .disable_statement_logging();

    let mut conn = sqlx::PgConnection::connect_with(&options).await
        .map_err(|_| AureonError::ConexaoPostgres(format!("Não foi possível conectar ao banco recém-criado: {}", db_name)))?;

    // 1. Criar tabela de controle de migrations
    sqlx::query("CREATE TABLE IF NOT EXISTS schema_migrations (version VARCHAR(255) PRIMARY KEY, applied_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP)")
        .execute(&mut conn).await.map_err(|e| AureonError::Migracao(e.to_string()))?;

    // 2. Rodar migrations idempotentes
    for (versao, sql) in MIGRATIONS_PG.iter() {
        let aplicada: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = $1)")
            .bind(versao)
            .fetch_one(&mut conn).await.unwrap_or((false,));

        if !aplicada.0 {
            sqlx::query(sql).execute(&mut conn).await
                .map_err(|e| AureonError::Migracao(format!("Falha na migration {}: {}", versao, e)))?;
            
            sqlx::query("INSERT INTO schema_migrations (version) VALUES ($1)")
                .bind(versao)
                .execute(&mut conn).await.unwrap();
        }
    }

    // 3. Criar Admin Inicial e Tesouraria
    // Hash da senha com bcrypt
    let hash = bcrypt::hash(&dados.admin_senha_plana, bcrypt::DEFAULT_COST)
        .map_err(|e| AureonError::Validacao(format!("Erro ao gerar hash da senha: {}", e)))?;

    let admin_exists: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM usuarios WHERE email = $1)")
        .bind(&dados.admin_email)
        .fetch_one(&mut conn).await.unwrap_or((false,));

    if !admin_exists.0 {
        // Busca perfil admin
        let perfil_id: Option<(uuid::Uuid,)> = sqlx::query_as("SELECT id FROM perfis WHERE nome = 'ADMINISTRADOR'")
            .fetch_optional(&mut conn).await.unwrap_or(None);
            
        if let Some((p_id,)) = perfil_id {
            sqlx::query("INSERT INTO usuarios (perfil_id, nome, email, senha_hash) VALUES ($1, $2, $3, $4)")
                .bind(p_id)
                .bind(&dados.admin_nome)
                .bind(&dados.admin_email)
                .bind(&hash)
                .execute(&mut conn).await
                .map_err(|e| AureonError::ConexaoPostgres(format!("Falha ao criar admin: {}", e)))?;
        }

        // Cria Tesouraria Central padrão atrelada ao BRL
        let moeda_brl: Option<(uuid::Uuid,)> = sqlx::query_as("SELECT id FROM moedas WHERE codigo = 'BRL'")
            .fetch_optional(&mut conn).await.unwrap_or(None);
            
        if let Some((m_id,)) = moeda_brl {
            sqlx::query("INSERT INTO tesourarias (nome, moeda_id) VALUES ('Tesouraria Central', $1) ON CONFLICT DO NOTHING")
                .bind(m_id)
                .execute(&mut conn).await.unwrap_or_default();
        }
    }

    // 4. Gravar o arquivo server.config.enc
    let config_path = std::path::Path::new("C:/Aureon/config/server.config.enc");
    let keystore_path = std::path::Path::new("C:/Aureon/config/.keystore");

    if !keystore_path.exists() {
        aureon_shared::crypto::gerar_e_salvar_keystore(keystore_path)
            .map_err(|e| AureonError::Configuracao(format!("Erro ao criar keystore: {:?}", e)))?;
    }

    let app_config = AppServerConfig {
        postgres_host: dados.db.host,
        postgres_porta: dados.db.porta,
        postgres_usuario: dados.db.usuario,
        postgres_senha: dados.db.senha_plana, // Será gravado criptografado via AES
        postgres_banco: db_name,
    };

    aureon_shared::config_store::salvar_config_criptografada(&app_config, config_path, keystore_path)
        .map_err(|e| AureonError::Configuracao(format!("Falha ao gravar configuração: {:?}", e)))?;

    Ok(RespostaBase::ok("Instalação concluída com sucesso!", true))
}

#[tauri::command]
pub async fn inicializar_keystore() -> Result<RespostaBase<bool>, AureonError> {
    let keystore_path = std::path::Path::new("C:/Aureon/config/.keystore");
    
    // Se não existe, cria a chave e restringe acesso (DT-010)
    if !keystore_path.exists() {
        aureon_shared::crypto::gerar_e_salvar_keystore(keystore_path)
            .map_err(|e| AureonError::Configuracao(format!("Erro ao criar keystore: {:?}", e)))?;
    }
    
    Ok(RespostaBase::ok("Keystore inicializado com sucesso", true))
}
