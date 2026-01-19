use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=RUSTDESK_MSI_PATH");

    let msi_path = env::var("RUSTDESK_MSI_PATH").unwrap_or_else(|_| {
        panic!(
            "RUSTDESK_MSI_PATH is not set.\n\
Build the MSI first, then set:\n\
  cmd.exe:       set RUSTDESK_MSI_PATH=C:\\path\\to\\Package.msi\n\
  PowerShell:    $env:RUSTDESK_MSI_PATH = \"C:\\path\\to\\Package.msi\"\n\
and build this crate again."
        )
    });

    println!("cargo:rerun-if-changed={}", msi_path);

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR not set"));
    let out_msi = out_dir.join("rustdesk.msi");

    let bytes = fs::read(&msi_path).unwrap_or_else(|e| {
        panic!("Failed to read MSI from RUSTDESK_MSI_PATH={}: {}", msi_path, e)
    });
    fs::write(&out_msi, bytes).unwrap_or_else(|e| {
        panic!("Failed to write embedded MSI to {}: {}", out_msi.display(), e)
    });
}

