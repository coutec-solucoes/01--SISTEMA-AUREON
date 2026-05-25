using System.Net.Http;
using System.Net.Http.Json;
using System.Threading.Tasks;
using System.Collections.Generic;

namespace AureonRetaguardaUi.Services
{
    public class FiscalApiClient
    {
        private readonly HttpClient _http;

        public FiscalApiClient(HttpClient http)
        {
            _http = http;
        }

        // --- Configurações ---
        public async Task<RespostaBase<FiscalEmpresaConfig>?> ObterConfiguracaoAsync()
        {
            return await _http.GetFromJsonAsync<RespostaBase<FiscalEmpresaConfig>>("/fiscal/configuracoes");
        }

        public async Task<RespostaBase<FiscalEmpresaConfig>?> SalvarConfiguracaoAsync(FiscalEmpresaConfig config)
        {
            var res = await _http.PostAsJsonAsync("/fiscal/configuracoes", config);
            return await res.Content.ReadFromJsonAsync<RespostaBase<FiscalEmpresaConfig>>();
        }

        public async Task<RespostaBase<FiscalEmpresaConfig>?> AtualizarConfiguracaoAsync(string id, FiscalEmpresaConfig config)
        {
            var res = await _http.PutAsJsonAsync($"/fiscal/configuracoes/{id}", config);
            return await res.Content.ReadFromJsonAsync<RespostaBase<FiscalEmpresaConfig>>();
        }

        // --- NCM ---
        public async Task<RespostaBase<List<FiscalNcm>>?> ListarNcmAsync()
        {
            return await _http.GetFromJsonAsync<RespostaBase<List<FiscalNcm>>>("/fiscal/dicionarios/ncm");
        }
        public async Task<RespostaBase<FiscalNcm>?> CriarNcmAsync(FiscalNcm dto)
        {
            var res = await _http.PostAsJsonAsync("/fiscal/dicionarios/ncm", dto);
            return await res.Content.ReadFromJsonAsync<RespostaBase<FiscalNcm>>();
        }
        public async Task<RespostaBase<FiscalNcm>?> AtualizarNcmAsync(string id, FiscalNcm dto)
        {
            var res = await _http.PutAsJsonAsync($"/fiscal/dicionarios/ncm/{id}", dto);
            return await res.Content.ReadFromJsonAsync<RespostaBase<FiscalNcm>>();
        }
        public async Task<RespostaBase<object>?> InativarNcmAsync(string id)
        {
            var res = await _http.PutAsync($"/fiscal/dicionarios/ncm/{id}/inativar", null);
            return await res.Content.ReadFromJsonAsync<RespostaBase<object>>();
        }

        // --- CFOP ---
        public async Task<RespostaBase<List<FiscalCfop>>?> ListarCfopAsync()
        {
            return await _http.GetFromJsonAsync<RespostaBase<List<FiscalCfop>>>("/fiscal/dicionarios/cfop");
        }
        public async Task<RespostaBase<FiscalCfop>?> CriarCfopAsync(FiscalCfop dto)
        {
            var res = await _http.PostAsJsonAsync("/fiscal/dicionarios/cfop", dto);
            return await res.Content.ReadFromJsonAsync<RespostaBase<FiscalCfop>>();
        }
        public async Task<RespostaBase<FiscalCfop>?> AtualizarCfopAsync(string id, FiscalCfop dto)
        {
            var res = await _http.PutAsJsonAsync($"/fiscal/dicionarios/cfop/{id}", dto);
            return await res.Content.ReadFromJsonAsync<RespostaBase<FiscalCfop>>();
        }
        public async Task<RespostaBase<object>?> InativarCfopAsync(string id)
        {
            var res = await _http.PutAsync($"/fiscal/dicionarios/cfop/{id}/inativar", null);
            return await res.Content.ReadFromJsonAsync<RespostaBase<object>>();
        }

        // --- CST/CSOSN ---
        public async Task<RespostaBase<List<FiscalCstCsosn>>?> ListarCstCsosnAsync()
        {
            return await _http.GetFromJsonAsync<RespostaBase<List<FiscalCstCsosn>>>("/fiscal/dicionarios/cst-csosn");
        }
        public async Task<RespostaBase<FiscalCstCsosn>?> CriarCstCsosnAsync(FiscalCstCsosn dto)
        {
            var res = await _http.PostAsJsonAsync("/fiscal/dicionarios/cst-csosn", dto);
            return await res.Content.ReadFromJsonAsync<RespostaBase<FiscalCstCsosn>>();
        }
        public async Task<RespostaBase<FiscalCstCsosn>?> AtualizarCstCsosnAsync(string id, FiscalCstCsosn dto)
        {
            var res = await _http.PutAsJsonAsync($"/fiscal/dicionarios/cst-csosn/{id}", dto);
            return await res.Content.ReadFromJsonAsync<RespostaBase<FiscalCstCsosn>>();
        }
        public async Task<RespostaBase<object>?> InativarCstCsosnAsync(string id)
        {
            var res = await _http.PutAsync($"/fiscal/dicionarios/cst-csosn/{id}/inativar", null);
            return await res.Content.ReadFromJsonAsync<RespostaBase<object>>();
        }

        // --- IVA ---
        public async Task<RespostaBase<List<FiscalIva>>?> ListarIvaAsync()
        {
            return await _http.GetFromJsonAsync<RespostaBase<List<FiscalIva>>>("/fiscal/dicionarios/iva");
        }
        public async Task<RespostaBase<FiscalIva>?> CriarIvaAsync(FiscalIva dto)
        {
            var res = await _http.PostAsJsonAsync("/fiscal/dicionarios/iva", dto);
            return await res.Content.ReadFromJsonAsync<RespostaBase<FiscalIva>>();
        }
        public async Task<RespostaBase<FiscalIva>?> AtualizarIvaAsync(string id, FiscalIva dto)
        {
            var res = await _http.PutAsJsonAsync($"/fiscal/dicionarios/iva/{id}", dto);
            return await res.Content.ReadFromJsonAsync<RespostaBase<FiscalIva>>();
        }
        public async Task<RespostaBase<object>?> InativarIvaAsync(string id)
        {
            var res = await _http.PutAsync($"/fiscal/dicionarios/iva/{id}/inativar", null);
            return await res.Content.ReadFromJsonAsync<RespostaBase<object>>();
        }

        // --- Regras Tributárias ---
        public async Task<RespostaBase<List<FiscalRegraTributaria>>?> ListarRegrasAsync()
        {
            return await _http.GetFromJsonAsync<RespostaBase<List<FiscalRegraTributaria>>>("/fiscal/regras");
        }
        public async Task<RespostaBase<FiscalRegraTributaria>?> CriarRegraAsync(FiscalRegraTributaria dto)
        {
            var res = await _http.PostAsJsonAsync("/fiscal/regras", dto);
            return await res.Content.ReadFromJsonAsync<RespostaBase<FiscalRegraTributaria>>();
        }
        public async Task<RespostaBase<FiscalRegraTributaria>?> AtualizarRegraAsync(string id, FiscalRegraTributaria dto)
        {
            var res = await _http.PutAsJsonAsync($"/fiscal/regras/{id}", dto);
            return await res.Content.ReadFromJsonAsync<RespostaBase<FiscalRegraTributaria>>();
        }
        public async Task<RespostaBase<object>?> InativarRegraAsync(string id)
        {
            var res = await _http.PutAsync($"/fiscal/regras/{id}/inativar", null);
            return await res.Content.ReadFromJsonAsync<RespostaBase<object>>();
        }

        // --- Versões / Publicações ---
        public async Task<RespostaBase<List<FiscalVersaoPublicacao>>?> ListarVersoesAsync()
        {
            return await _http.GetFromJsonAsync<RespostaBase<List<FiscalVersaoPublicacao>>>("/fiscal/versoes");
        }
        public async Task<RespostaBase<FiscalVersaoPublicacao>?> CriarVersaoRascunhoAsync(object dto)
        {
            var res = await _http.PostAsJsonAsync("/fiscal/versoes/rascunho", dto);
            return await res.Content.ReadFromJsonAsync<RespostaBase<FiscalVersaoPublicacao>>();
        }
        public async Task<RespostaBase<object>?> CancelarVersaoAsync(string id)
        {
            var res = await _http.PutAsync($"/fiscal/versoes/{id}/cancelar", null);
            return await res.Content.ReadFromJsonAsync<RespostaBase<object>>();
        }
        public async Task<RespostaBase<List<FiscalVersaoPublicacaoItem>>?> ListarItensVersaoAsync(string id)
        {
            return await _http.GetFromJsonAsync<RespostaBase<List<FiscalVersaoPublicacaoItem>>>($"/fiscal/versoes/{id}/itens");
        }
        public async Task<RespostaBase<string>?> PublicarVersaoAsync(string id)
        {
            var res = await _http.PostAsync($"/fiscal/versoes/{id}/publicar", null);
            return await res.Content.ReadFromJsonAsync<RespostaBase<string>>();
        }
        public async Task<RespostaBase<string>?> ReprocessarVersaoAsync(string id)
        {
            var res = await _http.PostAsync($"/fiscal/versoes/{id}/reprocessar", null);
            return await res.Content.ReadFromJsonAsync<RespostaBase<string>>();
        }
        public async Task<RespostaBase<PayloadFiscalDTO>?> ObterPayloadVersaoAsync(string id)
        {
            return await _http.GetFromJsonAsync<RespostaBase<PayloadFiscalDTO>>($"/fiscal/versoes/{id}/payload");
        }

        // --- Auditoria ---
        public async Task<RespostaBase<List<FiscalAuditoria>>?> ListarAuditoriaAsync()
        {
            return await _http.GetFromJsonAsync<RespostaBase<List<FiscalAuditoria>>>("/fiscal/auditoria");
        }

        // --- Certificado (Fase 18 - Bloco 1) ---
        public async Task<CertificadoMetadados?> ValidarCertificadoAsync(ValidarCertificadoReq req)
        {
            var res = await _http.PostAsJsonAsync("/fiscal/certificado/validar", req);
            return await res.Content.ReadFromJsonAsync<CertificadoMetadados>();
        }

        public async Task<CertificadoMetadados?> ObterStatusCertificadoAsync()
        {
            return await _http.GetFromJsonAsync<CertificadoMetadados>("/fiscal/certificado/status");
        }

        // --- Assinatura Preview (Fase 18 - Bloco 2) ---
        public async Task<AssinarPreviewResp?> AssinarPreviewAsync(AssinarPreviewReq req)
        {
            var res = await _http.PostAsJsonAsync("/fiscal/assinatura/assinar-preview", req);
            return await res.Content.ReadFromJsonAsync<AssinarPreviewResp>();
        }

        // --- NFC-e/NF-e Preview (Fase 18 - Bloco 3) ---
        public async Task<NfcePreviewResp?> MontarNfcePreviewAsync(MontarNfcePreviewReq req)
        {
            var res = await _http.PostAsJsonAsync("/fiscal/nfce/preview/montar", req);
            return await res.Content.ReadFromJsonAsync<NfcePreviewResp>();
        }

        public async Task<NfcePreviewResp?> MontarAssinarNfcePreviewAsync(MontarNfcePreviewReq req)
        {
            var reqComAssinar = new MontarNfcePreviewReq
            {
                VendaId = req.VendaId, Modelo = req.Modelo, Uf = req.Uf,
                Ambiente = req.Ambiente, Assinar = true,
                PfxBase64 = req.PfxBase64, Senha = req.Senha
            };
            var res = await _http.PostAsJsonAsync("/fiscal/nfce/preview/montar-assinar", reqComAssinar);
            return await res.Content.ReadFromJsonAsync<NfcePreviewResp>();
        }

        public async Task<NfcePreviewResp?> ObterVendaNfcePreviewAsync(string vendaId)
        {
            return await _http.GetFromJsonAsync<NfcePreviewResp>($"/fiscal/nfce/preview/venda/{vendaId}");
        }

        // --- SIFEN Preview (Fase 18 - Bloco 4) ---
        public async Task<SifenPreviewResp?> MontarSifenPreviewAsync(MontarSifenPreviewReq req)
        {
            var res = await _http.PostAsJsonAsync("/fiscal/sifen/preview/montar", req);
            return await res.Content.ReadFromJsonAsync<SifenPreviewResp>();
        }

        public async Task<SifenPreviewResp?> ObterVendaSifenPreviewAsync(string vendaId)
        {
            return await _http.GetFromJsonAsync<SifenPreviewResp>($"/fiscal/sifen/preview/venda/{vendaId}");
        }

        // --- Validação Local (Fase 18 - Bloco 5) ---
        public async Task<ValidacaoPreviewResp?> ValidarPreviewAsync(ValidarPreviewReq req)
        {
            var res = await _http.PostAsJsonAsync("/fiscal/preview/validar", req);
            return await res.Content.ReadFromJsonAsync<ValidacaoPreviewResp>();
        }

        // --- QR Code Preview (Fase 18 - Bloco 6) ---
        public async Task<QrCodePreviewResp?> GerarQrCodePreviewAsync(GerarQrCodePreviewReq req)
        {
            var res = await _http.PostAsJsonAsync("/fiscal/preview/qrcode", req);
            return await res.Content.ReadFromJsonAsync<QrCodePreviewResp>();
        }

        public async Task<QrCodePreviewResp?> GerarQrCodeNfcePreviewAsync(string chavePreview)
        {
            return await _http.GetFromJsonAsync<QrCodePreviewResp>($"/fiscal/preview/qrcode/nfce/{chavePreview}");
        }

        public async Task<QrCodePreviewResp?> GerarQrCodeSifenPreviewAsync(string cdcPreview)
        {
            return await _http.GetFromJsonAsync<QrCodePreviewResp>($"/fiscal/preview/qrcode/sifen/{cdcPreview}");
        }
        // --- Homologação Fiscal (Fase 19 - Bloco 7) ---
        public async Task<DiagnosticoFiscalHomologacaoResp?> ObterDiagnosticoHomologacaoAsync()
        {
            return await _http.GetFromJsonAsync<DiagnosticoFiscalHomologacaoResp>("/fiscal/homologacao/diagnostico");
        }

        public async Task<List<FiscalEndpointConfigResp>?> ListarEndpointsHomologacaoAsync()
        {
            return await _http.GetFromJsonAsync<List<FiscalEndpointConfigResp>>("/fiscal/homologacao/endpoints");
        }

        public async Task<TestarEndpointFiscalResp?> TestarEndpointHomologacaoAsync(TestarEndpointFiscalReq req)
        {
            var res = await _http.PostAsJsonAsync("/fiscal/homologacao/testar-endpoint", req);
            return await res.Content.ReadFromJsonAsync<TestarEndpointFiscalResp>();
        }

        public async Task<TestarEndpointFiscalResp?> ValidarBloqueioProducaoAsync(TestarEndpointFiscalReq req)
        {
            var res = await _http.PostAsJsonAsync("/fiscal/homologacao/validar-bloqueio-producao", req);
            return await res.Content.ReadFromJsonAsync<TestarEndpointFiscalResp>();
        }

        public async Task<TestarConectividadeFiscalResp?> TestarConectividadeHomologacaoAsync(TestarConectividadeFiscalReq req)
        {
            var res = await _http.PostAsJsonAsync("/fiscal/homologacao/testar-conectividade", req);
            return await res.Content.ReadFromJsonAsync<TestarConectividadeFiscalResp>();
        }

        public async Task<List<HistoricoHomologacaoFiscalResp>?> ListarHistoricoHomologacaoAsync(int limite = 50, int offset = 0)
        {
            return await _http.GetFromJsonAsync<List<HistoricoHomologacaoFiscalResp>>($"/fiscal/homologacao/historico?limite={limite}&offset={offset}");
        }

        public async Task<List<string>?> ListarTiposEventoHomologacaoAsync()
        {
            return await _http.GetFromJsonAsync<List<string>>("/fiscal/homologacao/historico/tipos");
        }

        public async Task<HistoricoHomologacaoFiscalResp?> ObterHistoricoHomologacaoAsync(string id)
        {
            return await _http.GetFromJsonAsync<HistoricoHomologacaoFiscalResp>($"/fiscal/homologacao/historico/{id}");
        }
    }
}
