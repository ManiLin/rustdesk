# RustDesk Setup EXE (service-only)

This is a small console bootstrapper that embeds the built MSI and runs it.

## Build (Windows)

1. Build `rustdesk.exe` (Release) and prepare a dist directory containing `RustDesk.exe` (case-insensitive on Windows).
2. Generate MSI sources:

```bat
cd res\msi
python preprocess.py --dist-dir "C:\path\to\dist" --service-only
```

3. Build MSI in Visual Studio: open `res\msi\msi.sln` → build **Release|x64**.
4. Build the EXE (this crate) and embed the MSI:

```bat
set RUSTDESK_MSI_PATH=C:\path\to\Package\bin\x64\Release\package.msi
cd res\msi\setup_exe
cargo build --release
```

Output: `res\msi\setup_exe\target\release\rustdesk-setup.exe`

## Usage

- Silent install (prints ID to stdout at the end):

```bat
rustdesk-setup.exe --silent --id-relay aup.tatnefturs.ru:10201 --access-pass Statusk371037
```

Defaults:
- `--access-pass Statusk371037`
- `--id-relay aup.tatnefturs.ru:10201` (relay auto = host:(port+3))

