fn main() {
    println!("cargo:rerun-if-env-changed=BERRY_UPDATE_PUBLIC_KEY");
    tauri_build::build()
}
