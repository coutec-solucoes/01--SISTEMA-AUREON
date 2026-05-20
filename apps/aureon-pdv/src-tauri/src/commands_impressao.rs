use std::fs;
use std::io::Write;
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;
use chrono::Local;

use aureon_core::dtos::{
    ImpressaoResultadoResp, ImpressoraDestinoReq, TesteImpressoraReq, TipoDestinoImpressao,
};

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
