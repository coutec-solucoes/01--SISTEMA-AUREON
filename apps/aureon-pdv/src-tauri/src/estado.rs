use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use aureon_infra::{ConexaoSqlite, executar_migrations};
use tracing::{info, error};

/// Estado global do app Tauri
pub struct EstadoApp {
    pub conn_sqlite: Arc<Mutex<Connection>>,
    pub dados_dir:   PathBuf,
}

/// Inicializa o estado do app: SQLite + migrations
pub fn inicializar_estado() -> EstadoApp {
    // Caminho de dados: C:/Aureon/data/ (produção) | AppData em dev
    let dados_dir = obter_dir_dados();
    std::fs::create_dir_all(&dados_dir).expect("Não foi possível criar diretório de dados");

    let db_path = dados_dir.join("aureon-local.db");

    info!(
        componente = "aureon-pdv::estado",
        caminho = %db_path.display(),
        "Inicializando SQLite"
    );

    let conexao = ConexaoSqlite::abrir(&db_path)
        .expect("Falha crítica: não foi possível abrir SQLite local");

    // Executa migrations
    if let Err(e) = executar_migrations(&conexao.conn) {
        error!(
            componente = "aureon-pdv::estado",
            erro = %e,
            "Falha ao executar migrations SQLite"
        );
        panic!("Falha crítica nas migrations: {e}");
    }

    info!(componente = "aureon-pdv::estado", "Estado inicializado com sucesso");

    let conn = Arc::new(Mutex::new(conexao.conn));

    EstadoApp { conn_sqlite: conn, dados_dir }
}

fn obter_dir_dados() -> PathBuf {
    // Em produção usa C:/Aureon/data; em dev usa diretório local
    if let Ok(dir) = std::env::var("AUREON_DADOS_DIR") {
        PathBuf::from(dir)
    } else {
        // Fallback: pasta "aureon-data" ao lado do executável
        let mut path = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("."))
            .parent()
            .unwrap_or(&PathBuf::from("."))
            .to_path_buf();
        path.push("aureon-data");
        path
    }
}
