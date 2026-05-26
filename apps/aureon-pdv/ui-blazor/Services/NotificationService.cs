using System.Threading.Tasks;
using Microsoft.JSInterop;

namespace AureonPdvUi.Services
{
    public class NotificationService
    {
        private readonly IJSRuntime _jsRuntime;
        
        public NotificationService(IJSRuntime jsRuntime)
        {
            _jsRuntime = jsRuntime;
        }

        public async void ShowSuccess(string message) 
        {
            try { await _jsRuntime.InvokeVoidAsync("alert", $"SUCESSO: {message}"); } catch {}
        }

        public async void ShowError(string message) 
        {
            try { await _jsRuntime.InvokeVoidAsync("alert", $"ERRO: {message}"); } catch {}
        }
    }
}
