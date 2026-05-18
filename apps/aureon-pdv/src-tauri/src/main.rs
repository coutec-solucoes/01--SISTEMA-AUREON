// main.rs — ponto de entrada para Windows (sem console)
// A lógica principal está em lib.rs para compatibilidade com mobile futuro

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    aureon_pdv_lib::run()
}
