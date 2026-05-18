using Microsoft.JSInterop;
using System.Text.Json;

namespace AureonPdvUi.Services;

/// <summary>
/// Serviço de comunicação entre Blazor e os Tauri Commands (Rust).
/// Toda chamada ao Rust passa por aqui — nunca direto da página.
/// </summary>
public class TauriService
{
    private readonly IJSRuntime _js;

    public TauriService(IJSRuntime js)
    {
        _js = js;
    }

    /// <summary>
    /// Chama um Tauri Command e retorna o resultado tipado.
    /// Retorna null em caso de falha.
    /// </summary>
    public async Task<T?> InvocarAsync<T>(string comando, object? args = null)
    {
        try
        {
            var resultado = await _js.InvokeAsync<JsonElement>(
                "aureon.invocar",
                comando,
                args
            );
            return JsonSerializer.Deserialize<T>(
                resultado.GetRawText(),
                new JsonSerializerOptions { PropertyNameCaseInsensitive = true }
            );
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[TauriService] Falha ao invocar '{comando}': {ex.Message}");
            return default;
        }
    }

    /// <summary>Verifica se o app está rodando dentro do Tauri</summary>
    public async Task<bool> IsTauriAsync()
        => await _js.InvokeAsync<bool>("aureon.isTauri");
}

/// <summary>Padrão de resposta dos Tauri Commands (espelha RespostaBase do Rust)</summary>
public record RespostaBase<T>(
    bool Sucesso,
    string Mensagem,
    T? Dados,
    DetalhesErro? Erro
);

public record DetalhesErro(string Codigo, string Detalhe);
