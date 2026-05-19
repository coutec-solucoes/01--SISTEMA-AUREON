using Microsoft.AspNetCore.Components;
using System.Net;

namespace AureonRetaguardaUi.Providers;

public class AureonHttpInterceptor : DelegatingHandler
{
    private readonly NavigationManager _navigationManager;

    public AureonHttpInterceptor(NavigationManager navigationManager)
    {
        _navigationManager = navigationManager;
    }

    protected override async Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken)
    {
        var response = await base.SendAsync(request, cancellationToken);

        if (response.StatusCode == HttpStatusCode.Unauthorized)
        {
            // O token expirou ou é inválido. A limpeza será feita no AuthService/AuthStateProvider ou aqui mesmo via rotina.
            // Apenas redireciona para login e anexa uma query string (opcional) para alertar o usuário
            _navigationManager.NavigateTo("/login");
        }

        return response;
    }
}
