using System.Net.Http.Json;
using System.Text.Json;
using Microsoft.AspNetCore.Components.Authorization;
using Microsoft.JSInterop;
using AureonRetaguardaUi.Providers;

namespace AureonRetaguardaUi.Services;

public class LoginRequest
{
    public string login { get; set; } = string.Empty;
    public string senha { get; set; } = string.Empty;
}

public class AuthService
{
    private readonly HttpClient _httpClient;
    private readonly AuthenticationStateProvider _authStateProvider;
    private readonly IJSRuntime _jsRuntime;

    public AuthService(HttpClient httpClient, AuthenticationStateProvider authStateProvider, IJSRuntime jsRuntime)
    {
        _httpClient = httpClient;
        _authStateProvider = authStateProvider;
        _jsRuntime = jsRuntime;
    }

    public async Task<string?> LoginAsync(string login, string senha)
    {
        var request = new LoginRequest { login = login, senha = senha };
        var response = await _httpClient.PostAsJsonAsync("/auth/login", request);

        if (response.IsSuccessStatusCode)
        {
            var content = await response.Content.ReadAsStringAsync();
            var result = JsonSerializer.Deserialize<JsonElement>(content);
            
            if (result.TryGetProperty("dados", out var dados) && dados.TryGetProperty("token", out var tokenProp))
            {
                var token = tokenProp.GetString();
                if (!string.IsNullOrEmpty(token))
                {
                    await _jsRuntime.InvokeVoidAsync("localStorage.setItem", "authToken", token);
                    ((AureonAuthStateProvider)_authStateProvider).NotifyUserAuthentication(token);
                    _httpClient.DefaultRequestHeaders.Authorization = new System.Net.Http.Headers.AuthenticationHeaderValue("Bearer", token);
                    return null; // success
                }
            }
            return "Resposta de token inválida do servidor.";
        }

        var errorContent = await response.Content.ReadAsStringAsync();
        try
        {
            var errorResult = JsonSerializer.Deserialize<JsonElement>(errorContent);
            return errorResult.GetProperty("mensagem").GetString();
        }
        catch
        {
            return "Erro ao realizar login. Verifique suas credenciais.";
        }
    }

    public async Task LogoutAsync()
    {
        try
        {
            await _httpClient.PostAsync("/auth/logout", null);
        }
        catch { /* ignora erro de rede no logout */ }
        
        await _jsRuntime.InvokeVoidAsync("localStorage.removeItem", "authToken");
        _httpClient.DefaultRequestHeaders.Authorization = null;
        ((AureonAuthStateProvider)_authStateProvider).NotifyUserLogout();
    }
}
