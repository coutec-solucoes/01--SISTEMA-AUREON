use std::fs;
use std::io::Write;
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;
use chrono::Local;
use tauri::State;
use tracing::info;

use aureon_core::{dtos::*, RespostaBase};
use crate::estado::EstadoApp;

/// Builder para comandos ESC/POS
pub struct EscPosBuilder {
    pub buffer: Vec<u8>,
    pub colunas: u8,
}

impl EscPosBuilder {
    pub fn new(colunas: u8) -> Self {
        Self {
            buffer: Vec::new(),
            colunas,
        }
    }

    /// Inicializa a impressora (ESC @)
    pub fn init(&mut self) -> &mut Self {
        self.buffer.extend_from_slice(&[0x1B, 0x40]);
        self
    }

    /// Define o alinhamento: 0=Esquerda, 1=Centro, 2=Direita (ESC a n)
    pub fn align(&mut self, alignment: u8) -> &mut Self {
        self.buffer.extend_from_slice(&[0x1B, 0x61, alignment]);
        self
    }

    /// Ativa/Desativa o negrito (ESC E n)
    pub fn bold(&mut self, enable: bool) -> &mut Self {
        let n = if enable { 1 } else { 0 };
        self.buffer.extend_from_slice(&[0x1B, 0x45, n]);
        self
    }

    /// Tamanho da fonte: normal (0x00) ou duplo (0x11 para altura e largura duplo)
    /// (GS ! n)
    pub fn size(&mut self, duplo: bool) -> &mut Self {
        let n = if duplo { 0x11 } else { 0x00 };
        self.buffer.extend_from_slice(&[0x1D, 0x21, n]);
        self
    }

    /// Imprime um texto com quebra de linha normal. Convertemos String para bytes simples.
    /// OBS: O ideal seria converter UTF-8 para CodePage da impressora (ex: CP850).
    /// Para manter simples, enviaremos apenas caracteres suportados e \n.
    pub fn text(&mut self, text: &str) -> &mut Self {
        // Conversão super simples substituindo caracteres especiais para ASCII.
        // Numa versão de produção usaríamos codepage mapping.
        let safe_text = text.replace("ç", "c").replace("ã", "a").replace("õ", "o").replace("á", "a").replace("é", "e").replace("í", "i").replace("ó", "o").replace("ú", "u").replace("Ç", "C").replace("Ã", "A").replace("Õ", "O").replace("Á", "A").replace("É", "E").replace("Í", "I").replace("Ó", "O").replace("Ú", "U");
        self.buffer.extend_from_slice(safe_text.as_bytes());
        self.buffer.push(0x0A); // LF
        self
    }

    /// Linha separadora de acordo com a largura em colunas
    pub fn separator(&mut self, char_type: char) -> &mut Self {
        let line: String = std::iter::repeat(char_type).take(self.colunas as usize).collect();
        self.text(&line)
    }

    /// Corta o papel (GS V m)
    pub fn cut(&mut self) -> &mut Self {
        self.buffer.extend_from_slice(&[0x1D, 0x56, 0x01]);
        self
    }

    /// Abre a gaveta de dinheiro no pino 2
    pub fn open_drawer(&mut self) -> &mut Self {
        // ESC p m t1 t2
        self.buffer.extend_from_slice(&[0x1B, 0x70, 0x00, 0x3C, 0x78]);
        self
    }
}

/// Executa a impressão física ou simulada
pub fn executar_impressao(
    destino: &ImpressoraDestinoReq,
    payload: &[u8],
) -> Result<ImpressaoResultadoResp, String> {
    match destino.tipo_destino {
        TipoDestinoImpressao::TcpIp => {
            let ip = destino.endereco_ip.as_deref().unwrap_or("127.0.0.1");
            let porta = destino.porta.unwrap_or(9100);
            let address = format!("{}:{}", ip, porta);

            match TcpStream::connect_timeout(&address.parse().unwrap(), Duration::from_secs(3)) {
                Ok(mut stream) => {
                    stream
                        .write_all(payload)
                        .map_err(|e| format!("Falha ao enviar dados TCP: {}", e))?;
                    Ok(ImpressaoResultadoResp {
                        sucesso: true,
                        mensagem: "Impressão enviada com sucesso (TCP/IP).".to_string(),
                        destino_usado: address,
                        caminho_arquivo_simulado: None,
                        bytes_gerados: payload.len(),
                    })
                }
                Err(e) => Err(format!("Falha ao conectar na impressora de rede: {}", e)),
            }
        }
        TipoDestinoImpressao::Simulador => {
            let dir_base = destino
                .caminho_simulador
                .clone()
                .unwrap_or_else(|| "C:/Aureon/print-sim".to_string());
            
            if !Path::new(&dir_base).exists() {
                let _ = fs::create_dir_all(&dir_base);
            }

            let file_name = format!(
                "{}/cupom_simulado_{}.escpos.txt",
                dir_base,
                Local::now().format("%Y%m%d_%H%M%S")
            );

            // Tenta criar uma versão "legível" só para o simulador se guiar, sem caracteres ESC,
            // ou salva os bytes brutos e cria um arquivo .txt legível junto.
            // Para simplicidade, salvaremos um .txt com o conteúdo string aproximado (ignorando os ESC commands).
            let legivel: String = payload.iter().map(|&b| {
                if b >= 32 && b <= 126 { b as char } else if b == 10 { '\n' } else { ' ' }
            }).collect();

            let txt_path = file_name.replace(".escpos.txt", ".txt");
            fs::write(&txt_path, legivel)
                .map_err(|e| format!("Falha ao salvar simulador legível: {}", e))?;

            // Salva também o binário bruto
            fs::write(&file_name, payload)
                .map_err(|e| format!("Falha ao salvar simulador bruto: {}", e))?;

            Ok(ImpressaoResultadoResp {
                sucesso: true,
                mensagem: "Impressão simulada com sucesso.".to_string(),
                destino_usado: "Simulador de Arquivo".to_string(),
                caminho_arquivo_simulado: Some(txt_path),
                bytes_gerados: payload.len(),
            })
        }
        TipoDestinoImpressao::WindowsRaw => {
            // Em uma implementação completa usaríamos a crate `winspool` ou PInvoke no C#.
            // Como requisito do Bloco 1, isolamos a função de forma que não quebre o build no Linux.
            #[cfg(target_os = "windows")]
            {
                // Aqui iria o código de comunicação com o Spooler
                // Exemplo: usar std::process::Command para lpr ou biblioteca RawPrinterHelper
                Err("Windows RAW Spooler não implementado nesta etapa.".to_string())
            }
            
            #[cfg(not(target_os = "windows"))]
            {
                Err("Windows RAW Spooler só é suportado no Windows.".to_string())
            }
        }
    }
}

/// Command Tauri: Testa a conectividade com a impressora emitindo um cupom não fiscal básico
#[tauri::command]
pub async fn testar_impressora(req: TesteImpressoraReq) -> Result<ImpressaoResultadoResp, String> {
    let mut builder = EscPosBuilder::new(req.destino.largura_colunas);
    
    builder.init();
    builder.align(1); // Centro
    builder.bold(true);
    builder.size(true);
    builder.text("AUREON PDV");
    builder.size(false);
    builder.text("TESTE DE IMPRESSAO");
    builder.bold(false);
    
    builder.align(0); // Esquerda
    builder.separator('-');
    
    let dh = Local::now().format("%d/%m/%Y %H:%M:%S").to_string();
    builder.text(&format!("DATA/HORA : {}", dh));
    
    let dest_name = if req.destino.tipo_destino.clone() as u8 == TipoDestinoImpressao::TcpIp as u8 {
        format!("TCP/IP: {}", req.destino.endereco_ip.as_deref().unwrap_or("N/A"))
    } else {
        req.destino.nome.clone()
    };
    
    builder.text(&format!("DESTINO   : {}", dest_name));
    
    if let Some(txt) = req.texto_teste {
        builder.text(&format!("MENSAGEM  : {}", txt));
    }

    builder.separator('-');
    builder.align(1); // Centro
    builder.bold(true);
    builder.text("DOCUMENTO NAO FISCAL");
    builder.text("NAO E VALIDO COMO DOCUMENTO FISCAL");
    builder.bold(false);
    builder.text(" ");
    builder.text(" ");
    builder.text(" ");
    
    if req.destino.cortar_papel {
        builder.cut();
    }
    
    if req.destino.abrir_gaveta {
        builder.open_drawer();
    }

    let payload = builder.buffer.clone();
    
    executar_impressao(&req.destino, &payload)
}

// ================================================================
// Lógica Auxiliar de Formatação
// ================================================================

fn formatar_moeda(valor_minor: i64) -> String {
    let inteiros = valor_minor / 100;
    let decimais = valor_minor % 100;
    format!("{}.{:02}", inteiros, decimais)
}

fn formatar_quantidade(qtd_escala3: i64) -> String {
    let inteiros = qtd_escala3 / 1000;
    let decimais = qtd_escala3 % 1000;
    if decimais == 0 {
        format!("{}", inteiros)
    } else {
        format!("{}.{:03}", inteiros, decimais)
    }
}

// ================================================================
// Command: imprimir_cupom_venda_nao_fiscal
// ================================================================

#[tauri::command]
pub async fn imprimir_cupom_venda_nao_fiscal(
    req: ImprimirVendaReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<ImpressaoResultadoResp>, String> {
    info!(
        componente = "aureon-pdv::commands_impressao",
        venda_id = %req.venda_id,
        "Chamada: imprimir_cupom_venda_nao_fiscal"
    );
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    
    match montar_cupom_venda(&conn, &req.venda_id, &req.destino, req.imprimir_itens_cancelados, false, None) {
        Ok(payload) => {
            let res = executar_impressao(&req.destino, &payload)?;
            Ok(RespostaBase::ok("Cupom impresso com sucesso", res))
        }
        Err(e) => Err(e),
    }
}

// ================================================================
// Command: reimprimir_cupom_venda_nao_fiscal
// ================================================================

#[tauri::command]
pub async fn reimprimir_cupom_venda_nao_fiscal(
    req: ReimprimirVendaReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<ImpressaoResultadoResp>, String> {
    info!(
        componente = "aureon-pdv::commands_impressao",
        venda_id = %req.venda_id,
        motivo = %req.motivo_reimpressao,
        "Chamada: reimprimir_cupom_venda_nao_fiscal"
    );
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;

    // Auditoria simplificada de reimpressão (não falhamos se a tabela não existir,
    // apenas logamos, pois a FASE 15 determina não alterar regras operacionais, 
    // mas pede auditoria SE existir fluxo)
    let _ = conn.execute(
        "INSERT INTO auditoria_pdv (id, acao, entidade, entidade_id, usuario_id, detalhes, criado_em)
         VALUES (?1, 'REIMPRESSAO', 'VENDA', ?2, ?3, ?4, ?5)",
        rusqlite::params![
            uuid::Uuid::new_v4().to_string(),
            req.venda_id,
            req.usuario_id,
            req.motivo_reimpressao,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
        ]
    );

    match montar_cupom_venda(&conn, &req.venda_id, &req.destino, true, true, Some(&req.motivo_reimpressao)) {
        Ok(payload) => {
            let res = executar_impressao(&req.destino, &payload)?;
            Ok(RespostaBase::ok("Cupom reimpresso com sucesso", res))
        }
        Err(e) => Err(e),
    }
}

fn montar_cupom_venda(
    conn: &rusqlite::Connection,
    venda_id: &str,
    destino: &ImpressoraDestinoReq,
    imprimir_cancelados: bool,
    is_reimpressao: bool,
    motivo_reimpressao: Option<&str>
) -> Result<Vec<u8>, String> {
    let mut builder = EscPosBuilder::new(destino.largura_colunas);

    // Buscar resumo da venda
    let (num_venda, usuario, cliente, sub, desc, acre, tot, criado_em) = conn.query_row(
        "SELECT v.numero_venda, v.usuario_id, c.nome,
                v.subtotal_minor, v.desconto_total_minor, v.acrescimo_total_minor, v.total_minor,
                v.criado_em
         FROM vendas v
         LEFT JOIN clientes_cache c ON c.id = v.cliente_id
         WHERE v.id = ?1",
        rusqlite::params![venda_id],
        |row| Ok((
            row.get::<_, Option<i64>>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, i64>(6)?,
            row.get::<_, String>(7)?,
        ))
    ).map_err(|_| "Venda não encontrada".to_string())?;

    builder.init();
    builder.align(1);
    builder.bold(true);
    builder.size(true);
    builder.text("AUREON PDV");
    builder.size(false);
    
    if is_reimpressao {
        builder.text("*** REIMPRESSAO ***");
        if let Some(motivo) = motivo_reimpressao {
            builder.text(&format!("MOTIVO: {}", motivo));
        }
    }
    
    builder.text("DOCUMENTO NAO FISCAL");
    builder.bold(false);
    builder.text("NAO E VALIDO COMO DOCUMENTO FISCAL");
    builder.separator('-');
    
    builder.align(0);
    if let Some(num) = num_venda {
        builder.text(&format!("CUPOM NO: {}", num));
    } else {
        builder.text(&format!("ID INTERNO: {}", &venda_id[..8]));
    }
    builder.text(&format!("DATA/HORA: {}", criado_em));
    if let Some(u) = usuario {
        builder.text(&format!("OPERADOR: {}", u));
    }
    if let Some(c) = cliente {
        builder.text(&format!("CLIENTE: {}", c));
    }
    
    builder.separator('-');
    builder.text("ITEM | DESC | QTD | UN | TOTAL");
    builder.separator('-');

    // Itens
    let mut stmt = conn.prepare(
        "SELECT id, descricao_produto, quantidade_escala3, preco_unitario_minor, total_item_minor, cancelado
         FROM venda_itens WHERE venda_id = ?1 ORDER BY criado_em ASC"
    ).map_err(|e| e.to_string())?;

    let iter = stmt.query_map(rusqlite::params![venda_id], |row| {
        Ok((
            row.get::<_, String>(1)?, // descricao
            row.get::<_, i64>(2)?,    // qtd
            row.get::<_, i64>(3)?,    // un
            row.get::<_, i64>(4)?,    // total
            row.get::<_, i32>(5)? == 1 // cancelado
        ))
    }).map_err(|e| e.to_string())?;

    let mut i = 1;
    for item in iter {
        if let Ok((desc_prod, qtd, un, tot_item, cancelado)) = item {
            if cancelado && !imprimir_cancelados { continue; }
            
            let mut prefix = format!("{:03} ", i);
            if cancelado {
                prefix.push_str("[CANC] ");
            }
            
            // Exemplo simples de formatação (numa versão de prod alinharíamos as colunas certinho)
            builder.text(&format!("{}{}", prefix, desc_prod));
            builder.text(&format!("    {} x {} = {}", formatar_quantidade(qtd), formatar_moeda(un), formatar_moeda(tot_item)));
            i += 1;
        }
    }

    builder.separator('-');
    builder.align(2); // Direita
    builder.text(&format!("SUBTOTAL: {}", formatar_moeda(sub)));
    if desc > 0 {
        builder.text(&format!("DESCONTO: {}", formatar_moeda(desc)));
    }
    if acre > 0 {
        builder.text(&format!("ACRESCIMO: {}", formatar_moeda(acre)));
    }
    builder.bold(true);
    builder.size(true);
    builder.text(&format!("TOTAL: {}", formatar_moeda(tot)));
    builder.size(false);
    builder.bold(false);

    // Pagamentos
    builder.align(0);
    builder.separator('-');
    builder.text("PAGAMENTOS");
    
    let mut stmt_pag = conn.prepare(
        "SELECT forma_pagamento, valor_informado_minor, troco_minor, moeda_codigo
         FROM venda_pagamentos WHERE venda_id = ?1 ORDER BY criado_em ASC"
    ).map_err(|e| e.to_string())?;

    let mut tot_troco = 0;
    let iter_pag = stmt_pag.query_map(rusqlite::params![venda_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, String>(3)?,
        ))
    }).map_err(|e| e.to_string())?;

    for p in iter_pag {
        if let Ok((forma, val, troco, moeda)) = p {
            builder.text(&format!("{:<15} {} {}", forma, formatar_moeda(val), moeda));
            tot_troco += troco;
        }
    }

    if tot_troco > 0 {
        builder.text(&format!("TROCO: {}", formatar_moeda(tot_troco)));
    }

    builder.separator('-');
    builder.align(1);
    builder.bold(true);
    builder.text("DOCUMENTO NAO FISCAL");
    builder.bold(false);
    builder.text(" ");
    builder.text(" ");
    builder.text(" ");
    builder.text(" ");

    if destino.cortar_papel {
        builder.cut();
    }
    
    // Abrimos a gaveta para testes controlados apenas se solicitado
    if destino.abrir_gaveta {
        builder.open_drawer();
    }

    Ok(builder.buffer.clone())
}


// ================================================================
// Command: imprimir_comprovante_baixa_financeira
// ================================================================

#[tauri::command]
pub async fn imprimir_comprovante_baixa_financeira(
    req: ImprimirBaixaFinanceiraReq,
    estado: State<'_, EstadoApp>,
) -> Result<RespostaBase<ImpressaoResultadoResp>, String> {
    info!(
        componente = "aureon-pdv::commands_impressao",
        lancamento_id = %req.lancamento_id,
        "Chamada: imprimir_comprovante_baixa_financeira"
    );
    let conn = estado.conn_sqlite.lock().map_err(|e| e.to_string())?;
    
    let mut builder = EscPosBuilder::new(req.destino.largura_colunas);

    // Buscar lancamento
    let (tipo_lanc, descricao, moeda, forma_pag, valor_minor, data_pag, sessao, obs, conta_pagar_id, conta_receber_id) = conn.query_row(
        "SELECT tipo_lancamento, descricao, moeda_codigo, forma_pagamento, 
                valor_informado_minor, data_pagamento, sessao_caixa_id, observacao,
                conta_pagar_id, conta_receber_id
         FROM financeiro_lancamentos 
         WHERE id = ?1",
        rusqlite::params![req.lancamento_id],
        |row| Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, Option<String>>(6)?,
            row.get::<_, Option<String>>(7)?,
            row.get::<_, Option<String>>(8)?,
            row.get::<_, Option<String>>(9)?,
        ))
    ).map_err(|_| "Lançamento financeiro não encontrado".to_string())?;

    builder.init();
    builder.align(1);
    builder.bold(true);
    builder.size(true);
    builder.text("AUREON PDV");
    builder.size(false);
    
    if tipo_lanc == "PAGAMENTO" {
        builder.text("COMPROVANTE DE PAGAMENTO");
    } else {
        builder.text("COMPROVANTE DE RECEBIMENTO");
    }
    
    builder.text("DOCUMENTO NAO FISCAL");
    builder.bold(false);
    builder.text("NAO E VALIDO COMO DOCUMENTO FISCAL");
    builder.separator('-');
    
    builder.align(0);
    builder.text(&format!("DATA PAGTO : {}", data_pag));
    builder.text(&format!("TIPO       : {}", tipo_lanc));
    builder.text(&format!("DESCRICAO  : {}", descricao));
    
    // Complementos da conta vinculada
    if let Some(cp_id) = conta_pagar_id {
        if let Ok(fornecedor) = conn.query_row(
            "SELECT fornecedor_nome_snapshot FROM contas_pagar WHERE id = ?1",
            rusqlite::params![cp_id],
            |r| r.get::<_, Option<String>>(0)
        ) {
            if let Some(f) = fornecedor {
                builder.text(&format!("FORNECEDOR : {}", f));
            }
        }
    }
    if let Some(cr_id) = conta_receber_id {
        if let Ok(cliente) = conn.query_row(
            "SELECT cliente_nome_snapshot FROM contas_receber WHERE id = ?1",
            rusqlite::params![cr_id],
            |r| r.get::<_, Option<String>>(0)
        ) {
            if let Some(c) = cliente {
                builder.text(&format!("CLIENTE    : {}", c));
            }
        }
    }

    if let Some(s) = sessao {
        builder.text(&format!("SESSAO CX  : {}", &s[..8]));
    }
    
    builder.separator('-');
    builder.bold(true);
    builder.text(&format!("VALOR: {} {}", formatar_moeda(valor_minor), moeda));
    builder.bold(false);
    builder.text(&format!("FORMA: {}", forma_pag));
    
    if let Some(o) = obs {
        if !o.is_empty() {
            builder.separator('-');
            builder.text(&format!("OBS: {}", o));
        }
    }

    builder.separator('-');
    builder.align(1);
    builder.text(" ");
    builder.text("_________________________________");
    builder.text("ASSINATURA DO RESPONSAVEL");
    builder.text(" ");
    builder.text(" ");
    builder.text(" ");
    builder.text(" ");

    if req.destino.cortar_papel {
        builder.cut();
    }
    
    if req.destino.abrir_gaveta {
        builder.open_drawer();
    }

    let payload = builder.buffer.clone();
    
    match executar_impressao(&req.destino, &payload) {
        Ok(res) => Ok(RespostaBase::ok("Comprovante financeiro impresso com sucesso", res)),
        Err(e) => Err(e),
    }
}

