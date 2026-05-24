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
}
