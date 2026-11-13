fn main() {
    // Relance la compilation si l'identifiant Microsoft change (lu via option_env! dans account_manager).
    println!("cargo:rerun-if-env-changed=MS_CLIENT_ID");
    tauri_build::build()
}
