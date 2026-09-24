fn main() {
    println!("cargo:rerun-if-env-changed=OMERA_UPDATE_PUBLIC_KEY");
    println!("cargo:rerun-if-env-changed=BERRY_UPDATE_PUBLIC_KEY");
    tauri_build::build()
}
