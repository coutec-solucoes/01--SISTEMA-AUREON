/**
 * tauri-interop.js — Ponte entre Blazor WASM e Tauri 2.0
 * 
 * No Tauri 2.0, os commands são chamados via window.__TAURI__.core.invoke().
 * Este arquivo fornece uma função segura que detecta se está rodando
 * dentro do Tauri ou no navegador (para desenvolvimento).
 */

/**
 * Verifica se o app está rodando dentro do Tauri
 */
window.aureon = window.aureon || {};

window.aureon.isTauri = function () {
    return typeof window.__TAURI__ !== 'undefined';
};

/**
 * Chama um Tauri Command de forma segura.
 * Em ambiente de browser (fora do Tauri), simula a resposta.
 * 
 * @param {string} comando - Nome do command Tauri (ex: "obter_status_local")
 * @param {object} args    - Argumentos do command (objeto JSON)
 * @returns {Promise<any>}
 */
window.aureon.invocar_json = async function (comando, jsonStr) {
    if (window.aureon.isTauri()) {
        try {
            const parsedArgs = JSON.parse(jsonStr);
            return await window.__TAURI__.core.invoke(comando, parsedArgs);
        } catch (erro) {
            console.error('[Aureon] Falha ao invocar command:', comando, erro);
            // Tauri errors can be strings, throw a JS Error to bubble up cleanly
            throw new Error(typeof erro === 'string' ? erro : JSON.stringify(erro));
        }
    } else {
        // Fora do Tauri: retorna resposta simulada para desenvolvimento
        console.warn('[Aureon] Fora do Tauri — retornando resposta simulada para:', comando);
        return {
            sucesso: false,
            mensagem: 'Executando fora do Tauri (ambiente de desenvolvimento).',
            dados: null,
            erro: {
                codigo: 'FORA_DO_TAURI',
                detalhe: 'Este command requer o ambiente Tauri para funcionar.'
            }
        };
    }
};

console.log('[Aureon] tauri-interop.js carregado. Tauri detectado:', window.aureon.isTauri());
