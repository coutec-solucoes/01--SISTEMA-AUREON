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
    }
}
