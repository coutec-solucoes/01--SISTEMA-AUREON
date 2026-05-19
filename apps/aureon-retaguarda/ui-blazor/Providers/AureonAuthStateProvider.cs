using System.Security.Claims;
using System.Text.Json;
using Microsoft.AspNetCore.Components.Authorization;
using Microsoft.JSInterop;

namespace AureonRetaguardaUi.Providers;

public class AureonAuthStateProvider : AuthenticationStateProvider
{
    private readonly IJSRuntime _jsRuntime;
    private readonly HttpClient _httpClient;

    public AureonAuthStateProvider(IJSRuntime jsRuntime, HttpClient httpClient)
    {
        _jsRuntime = jsRuntime;
        _httpClient = httpClient;
    }

    public override async Task<AuthenticationState> GetAuthenticationStateAsync()
    {
        try
        {
            var token = await _jsRuntime.InvokeAsync<string>("localStorage.getItem", "authToken");

            if (string.IsNullOrWhiteSpace(token))
            {
                return new AuthenticationState(new ClaimsPrincipal(new ClaimsIdentity()));
            }

            _httpClient.DefaultRequestHeaders.Authorization = new System.Net.Http.Headers.AuthenticationHeaderValue("Bearer", token);

            // Fetch user from /auth/me
            var response = await _httpClient.GetAsync("/auth/me");
            if (!response.IsSuccessStatusCode)
            {
                await _jsRuntime.InvokeVoidAsync("localStorage.removeItem", "authToken");
                return new AuthenticationState(new ClaimsPrincipal(new ClaimsIdentity()));
            }

            var content = await response.Content.ReadAsStringAsync();
            var result = JsonSerializer.Deserialize<JsonElement>(content);
            var dados = result.GetProperty("dados");
            
            var claims = new List<Claim>();
            claims.Add(new Claim(ClaimTypes.NameIdentifier, dados.GetProperty("id").GetString() ?? ""));
            claims.Add(new Claim(ClaimTypes.Name, dados.GetProperty("nome").GetString() ?? ""));
            claims.Add(new Claim("Login", dados.GetProperty("login").GetString() ?? ""));
            claims.Add(new Claim("PerfilId", dados.GetProperty("perfil_id").GetString() ?? ""));
            claims.Add(new Claim(ClaimTypes.Role, dados.GetProperty("perfil_nome").GetString() ?? ""));
            
            if (dados.GetProperty("is_admin").GetBoolean())
            {
                claims.Add(new Claim("IsAdmin", "true"));
            }

            var identity = new ClaimsIdentity(claims, "AureonAuth");
            var user = new ClaimsPrincipal(identity);

            return new AuthenticationState(user);
        }
        catch
        {
            return new AuthenticationState(new ClaimsPrincipal(new ClaimsIdentity()));
        }
    }

    public void NotifyUserAuthentication(string token)
    {
        var identity = new ClaimsIdentity(new[] { new Claim(ClaimTypes.Name, "Loading...") }, "AureonAuth");
        var authenticatedUser = new ClaimsPrincipal(identity);
        var authState = Task.FromResult(new AuthenticationState(authenticatedUser));
        NotifyAuthenticationStateChanged(authState);
    }

    public void NotifyUserLogout()
    {
        var authState = Task.FromResult(new AuthenticationState(new ClaimsPrincipal(new ClaimsIdentity())));
        NotifyAuthenticationStateChanged(authState);
    }
}
