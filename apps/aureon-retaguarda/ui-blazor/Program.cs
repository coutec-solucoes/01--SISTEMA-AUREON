using Microsoft.AspNetCore.Components.Web;
using Microsoft.AspNetCore.Components.WebAssembly.Hosting;
using Microsoft.AspNetCore.Components.Authorization;
using AureonRetaguardaUi;
using AureonRetaguardaUi.Providers;
using AureonRetaguardaUi.Services;

var builder = WebAssemblyHostBuilder.CreateDefault(args);

builder.RootComponents.Add<App>("#app");
builder.RootComponents.Add<HeadOutlet>("head::after");

// Configura o HttpClient padrão para apontar para a API Local Rust na porta 7070
builder.Services.AddScoped(sp =>
    new HttpClient { BaseAddress = new Uri("http://localhost:7070") });

builder.Services.AddAuthorizationCore();
builder.Services.AddScoped<AuthenticationStateProvider, AureonAuthStateProvider>();
builder.Services.AddScoped<AuthService>();
builder.Services.AddScoped<FiscalApiClient>();

await builder.Build().RunAsync();
