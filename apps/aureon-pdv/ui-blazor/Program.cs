using Microsoft.AspNetCore.Components.Web;
using Microsoft.AspNetCore.Components.WebAssembly.Hosting;
using AureonPdvUi;
using AureonPdvUi.Services;

var builder = WebAssemblyHostBuilder.CreateDefault(args);

builder.RootComponents.Add<App>("#app");
builder.RootComponents.Add<HeadOutlet>("head::after");

// Registra o serviço de interoperabilidade com o Tauri
builder.Services.AddScoped<TauriService>();

// HTTP client base (para chamadas futuras à API Local)
builder.Services.AddScoped(sp =>
    new HttpClient { BaseAddress = new Uri("http://localhost:7070") });

await builder.Build().RunAsync();
