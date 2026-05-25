using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace AureonRetaguardaUi.Services
{
    public class RespostaBase<T>
    {
        [JsonPropertyName("sucesso")]
        public bool Sucesso { get; set; }
        [JsonPropertyName("mensagem")]
        public string Mensagem { get; set; } = string.Empty;
        [JsonPropertyName("dados")]
        public T? Dados { get; set; }
    }

    public class FiscalEmpresaConfig
    {
        [JsonPropertyName("id")]
        public string Id { get; set; } = string.Empty;
        [JsonPropertyName("empresa_id")]
        public string? EmpresaId { get; set; }
        [JsonPropertyName("filial_id")]
        public string? FilialId { get; set; }
        [JsonPropertyName("pais_fiscal")]
        public string PaisFiscal { get; set; } = "BR";
        [JsonPropertyName("regime_fiscal")]
        public string? RegimeFiscal { get; set; }
        [JsonPropertyName("ambiente")]
        public string Ambiente { get; set; } = "HOMOLOGACAO";
        [JsonPropertyName("forma_emissao")]
        public string FormaEmissao { get; set; } = "NORMAL";
        [JsonPropertyName("certificado_alias")]
        public string? CertificadoAlias { get; set; }
        [JsonPropertyName("ativo")]
        public bool Ativo { get; set; } = true;
    }

    public class FiscalDicionarioBase
    {
        [JsonPropertyName("id")]
        public string Id { get; set; } = string.Empty;
        [JsonPropertyName("codigo")]
        public string Codigo { get; set; } = string.Empty;
        [JsonPropertyName("descricao")]
        public string Descricao { get; set; } = string.Empty;
        [JsonPropertyName("ativo")]
        public bool Ativo { get; set; } = true;
        [JsonPropertyName("criado_em")]
        public string? CriadoEm { get; set; }
    }

    public class FiscalNcm : FiscalDicionarioBase { }

    public class FiscalCfop : FiscalDicionarioBase
    {
        [JsonPropertyName("tipo_operacao")]
        public string? TipoOperacao { get; set; } // ENTRADA, SAIDA
    }

    public class FiscalCstCsosn : FiscalDicionarioBase
    {
        [JsonPropertyName("tipo")]
        public string Tipo { get; set; } = "CST"; // CST, CSOSN
    }

    public class FiscalIva : FiscalDicionarioBase
    {
        [JsonPropertyName("pais_fiscal")]
        public string PaisFiscal { get; set; } = "PY";
        [JsonPropertyName("aliquota_escala6")]
        public long AliquotaEscala6 { get; set; }
    }

    public class FiscalRegraTributaria
    {
        [JsonPropertyName("id")]
        public string Id { get; set; } = string.Empty;
        [JsonPropertyName("empresa_id")]
        public string? EmpresaId { get; set; }
        [JsonPropertyName("filial_id")]
        public string? FilialId { get; set; }
        [JsonPropertyName("pais_fiscal")]
        public string PaisFiscal { get; set; } = "BR";
        [JsonPropertyName("tipo_operacao")]
        public string TipoOperacao { get; set; } = "SAIDA";
        [JsonPropertyName("uf_origem")]
        public string? UfOrigem { get; set; }
        [JsonPropertyName("uf_destino")]
        public string? UfDestino { get; set; }
        
        [JsonPropertyName("ncm_id")]
        public string? NcmId { get; set; }
        [JsonPropertyName("cfop_id")]
        public string? CfopId { get; set; }
        [JsonPropertyName("cst_csosn_id")]
        public string? CstCsosnId { get; set; }
        [JsonPropertyName("iva_id")]
        public string? IvaId { get; set; }

        [JsonPropertyName("aliquota_icms_escala6")]
        public long? AliquotaIcmsEscala6 { get; set; }
        [JsonPropertyName("aliquota_pis_escala6")]
        public long? AliquotaPisEscala6 { get; set; }
        [JsonPropertyName("aliquota_cofins_escala6")]
        public long? AliquotaCofinsEscala6 { get; set; }
        [JsonPropertyName("aliquota_iva_escala6")]
        public long? AliquotaIvaEscala6 { get; set; }
        [JsonPropertyName("reducao_base_escala6")]
        public long? ReducaoBaseEscala6 { get; set; }

        [JsonPropertyName("prioridade")]
        public int Prioridade { get; set; } = 0;
        [JsonPropertyName("ativo")]
        public bool Ativo { get; set; } = true;
    }

    public class FiscalVersaoPublicacao
    {
        [JsonPropertyName("id")]
        public string Id { get; set; } = string.Empty;
        [JsonPropertyName("versao")]
        public string Versao { get; set; } = string.Empty;
        [JsonPropertyName("pais_fiscal")]
        public string PaisFiscal { get; set; } = "BR";
        [JsonPropertyName("status")]
        public string Status { get; set; } = "RASCUNHO"; // RASCUNHO, PUBLICADA, REPROCESSADA, CANCELADA
        [JsonPropertyName("total_registros")]
        public long TotalRegistros { get; set; }
        [JsonPropertyName("criado_em")]
        public string? CriadoEm { get; set; }
        [JsonPropertyName("publicado_em")]
        public string? PublicadoEm { get; set; }
    }

    public class FiscalVersaoPublicacaoItem
    {
        [JsonPropertyName("id")]
        public string Id { get; set; } = string.Empty;
        [JsonPropertyName("versao_fiscal_id")]
        public string VersaoFiscalId { get; set; } = string.Empty;
        [JsonPropertyName("entidade")]
        public string Entidade { get; set; } = string.Empty;
        [JsonPropertyName("entidade_id")]
        public string EntidadeId { get; set; } = string.Empty;
        [JsonPropertyName("operacao")]
        public string Operacao { get; set; } = string.Empty;
        [JsonPropertyName("payload_snapshot")]
        public string? PayloadSnapshot { get; set; }
    }

    public class FiscalAuditoria
    {
        [JsonPropertyName("id")]
        public string Id { get; set; } = string.Empty;
        [JsonPropertyName("entidade")]
        public string Entidade { get; set; } = string.Empty;
        [JsonPropertyName("entidade_id")]
        public string EntidadeId { get; set; } = string.Empty;
        [JsonPropertyName("acao")]
        public string Acao { get; set; } = string.Empty;
        [JsonPropertyName("usuario_id")]
        public string? UsuarioId { get; set; }
        [JsonPropertyName("detalhes")]
        public string? Detalhes { get; set; }
        [JsonPropertyName("criado_em")]
        public string CriadoEm { get; set; } = string.Empty;
    }

    public class PayloadFiscalDTO
    {
        [JsonPropertyName("versao_fiscal_id")]
        public string? VersaoFiscalId { get; set; }
        [JsonPropertyName("versao")]
        public string? Versao { get; set; }
        [JsonPropertyName("pais_fiscal")]
        public string? PaisFiscal { get; set; }
        [JsonPropertyName("blocos")]
        public Dictionary<string, object>? Blocos { get; set; }
        [JsonPropertyName("total_registros")]
        public long TotalRegistros { get; set; }
    }

    // ─── DTOs de Certificado (Fase 18 - Bloco 1) ───────────────────────────────
    public class ValidarCertificadoReq
    {
        [JsonPropertyName("caminho_pfx")]
        public string? CaminhoPfx { get; set; }
        [JsonPropertyName("pfx_base64")]
        public string? PfxBase64 { get; set; }
        [JsonPropertyName("senha")]
        public string Senha { get; set; } = string.Empty;
    }

    public class CertificadoMetadados
    {
        [JsonPropertyName("valido")]
        public bool Valido { get; set; }
        [JsonPropertyName("cn")]
        public string? Cn { get; set; }
        [JsonPropertyName("cnpj")]
        public string? Cnpj { get; set; }
        [JsonPropertyName("numero_serie")]
        public string? NumeroSerie { get; set; }
        [JsonPropertyName("validade_inicio")]
        public string? ValidadeInicio { get; set; }
        [JsonPropertyName("validade_fim")]
        public string? ValidadeFim { get; set; }
        [JsonPropertyName("dias_para_expirar")]
        public long? DiasParaExpirar { get; set; }
        [JsonPropertyName("expirado")]
        public bool Expirado { get; set; }
        [JsonPropertyName("alerta_expiracao")]
        public bool AlertaExpiracao { get; set; }
        [JsonPropertyName("mensagem")]
        public string? Mensagem { get; set; }
    }

    // ─── DTOs de Assinatura Preview (Fase 18 - Bloco 2) ───────────────────────
    public class AssinarPreviewReq
    {
        [JsonPropertyName("xml_conteudo")]
        public string XmlConteudo { get; set; } = string.Empty;
        [JsonPropertyName("pfx_base64")]
        public string? PfxBase64 { get; set; }
        [JsonPropertyName("senha")]
        public string? Senha { get; set; }
        [JsonPropertyName("ambiente")]
        public string Ambiente { get; set; } = "HOMOLOGACAO";
    }

    public class AssinarPreviewResp
    {
        [JsonPropertyName("sucesso")]
        public bool Sucesso { get; set; }
        [JsonPropertyName("xml_assinado")]
        public string? XmlAssinado { get; set; }
        [JsonPropertyName("resumo")]
        public string? Resumo { get; set; }
        [JsonPropertyName("ambiente")]
        public string? Ambiente { get; set; }
        [JsonPropertyName("mensagem")]
        public string? Mensagem { get; set; }
        [JsonPropertyName("warnings")]
        public List<string>? Warnings { get; set; }
    }

    // ─── DTOs de NFC-e/NF-e Preview (Fase 18 - Bloco 3) ──────────────────────
    public class MontarNfcePreviewReq
    {
        [JsonPropertyName("venda_id")]
        public string? VendaId { get; set; }
        [JsonPropertyName("modelo")]
        public string Modelo { get; set; } = "NFCE"; // NFCE ou NFE
        [JsonPropertyName("uf")]
        public string? Uf { get; set; }
        [JsonPropertyName("ambiente")]
        public string Ambiente { get; set; } = "HOMOLOGACAO";
        [JsonPropertyName("assinar")]
        public bool Assinar { get; set; } = false;
        [JsonPropertyName("pfx_base64")]
        public string? PfxBase64 { get; set; }
        [JsonPropertyName("senha")]
        public string? Senha { get; set; }
    }

    public class NfcePreviewResp
    {
        [JsonPropertyName("sucesso")]
        public bool Sucesso { get; set; }
        [JsonPropertyName("xml_preview")]
        public string? XmlPreview { get; set; }
        [JsonPropertyName("chave_preview")]
        public string? ChavePreview { get; set; }
        [JsonPropertyName("ambiente")]
        public string? Ambiente { get; set; }
        [JsonPropertyName("assinado")]
        public bool Assinado { get; set; }
        [JsonPropertyName("mensagem")]
        public string? Mensagem { get; set; }
        [JsonPropertyName("warnings")]
        public List<string>? Warnings { get; set; }
    }

    // ─── DTOs de SIFEN Preview (Fase 18 - Bloco 4) ────────────────────────────
    public class MontarSifenPreviewReq
    {
        [JsonPropertyName("venda_id")]
        public string? VendaId { get; set; }
        [JsonPropertyName("ambiente")]
        public string Ambiente { get; set; } = "HOMOLOGACAO";
    }

    public class SifenPreviewResp
    {
        [JsonPropertyName("sucesso")]
        public bool Sucesso { get; set; }
        [JsonPropertyName("json_preview")]
        public object? JsonPreview { get; set; }
        [JsonPropertyName("cdc_preview")]
        public string? CdcPreview { get; set; }
        [JsonPropertyName("ambiente")]
        public string? Ambiente { get; set; }
        [JsonPropertyName("mensagem")]
        public string? Mensagem { get; set; }
        [JsonPropertyName("warnings")]
        public List<string>? Warnings { get; set; }
    }

    // ─── DTOs de Validação Local (Fase 18 - Bloco 5) ──────────────────────────
    public class ValidarPreviewReq
    {
        [JsonPropertyName("tipo")]
        public string Tipo { get; set; } = "NFCE_XML"; // NFCE_XML, NFE_XML, SIFEN_JSON
        [JsonPropertyName("conteudo")]
        public string Conteudo { get; set; } = string.Empty;
        [JsonPropertyName("ambiente")]
        public string Ambiente { get; set; } = "HOMOLOGACAO";
    }

    public class ValidacaoPreviewErro
    {
        [JsonPropertyName("codigo")]
        public string? Codigo { get; set; }
        [JsonPropertyName("campo")]
        public string? Campo { get; set; }
        [JsonPropertyName("mensagem")]
        public string? Mensagem { get; set; }
        [JsonPropertyName("severidade")]
        public string? Severidade { get; set; }
    }

    public class ValidacaoPreviewResp
    {
        [JsonPropertyName("valido")]
        public bool Valido { get; set; }
        [JsonPropertyName("tipo")]
        public string? Tipo { get; set; }
        [JsonPropertyName("ambiente")]
        public string? Ambiente { get; set; }
        [JsonPropertyName("total_erros")]
        public int TotalErros { get; set; }
        [JsonPropertyName("erros")]
        public List<ValidacaoPreviewErro>? Erros { get; set; }
        [JsonPropertyName("warnings")]
        public List<ValidacaoPreviewErro>? Warnings { get; set; }
        [JsonPropertyName("mensagem")]
        public string? Mensagem { get; set; }
    }

    // ─── DTOs de QR Code Preview (Fase 18 - Bloco 6) ─────────────────────────
    public class GerarQrCodePreviewReq
    {
        [JsonPropertyName("tipo")]
        public string Tipo { get; set; } = "NFCE"; // NFCE, NFE, SIFEN
        [JsonPropertyName("chave_preview")]
        public string? ChavePreview { get; set; }
        [JsonPropertyName("cdc_preview")]
        public string? CdcPreview { get; set; }
        [JsonPropertyName("uf")]
        public string? Uf { get; set; }
        [JsonPropertyName("ambiente")]
        public string Ambiente { get; set; } = "HOMOLOGACAO";
        [JsonPropertyName("url_base_preview")]
        public string? UrlBasePreview { get; set; }
    }

    public class QrCodePreviewResp
    {
        [JsonPropertyName("sucesso")]
        public bool Sucesso { get; set; }
        [JsonPropertyName("tipo")]
        public string? Tipo { get; set; }
        [JsonPropertyName("ambiente")]
        public string? Ambiente { get; set; }
        [JsonPropertyName("conteudo_qr")]
        public string? ConteudoQr { get; set; }
        [JsonPropertyName("png_base64")]
        public string? PngBase64 { get; set; } // Na verdade SVG base64
        [JsonPropertyName("mensagem")]
        public string? Mensagem { get; set; }
        [JsonPropertyName("warnings")]
        public List<string>? Warnings { get; set; }
    }
}
