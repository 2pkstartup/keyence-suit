# Keyence Command Sender

Samostatná Windows/Rust aplikace pro odeslání jednoho příkazu do Keyence SR-750 přes TCP.
Po odeslání příkazu korektně ukončí výstupní polovinu spojení a skončí.

## Konfigurace

Vytvořte konfigurační soubor:

```powershell
Copy-Item .\sr_sender.conf.example .\sr_sender.conf
```

Příklad:

```ini
reader_address=192.168.210.200:9004
connect_timeout_ms=5000
```

`reader_address` je IP adresa a TCP port čtečky.
`connect_timeout_ms` je timeout připojení v milisekundách.

## Použití

```powershell
cargo run -- "READ"
```

Argumenty za cestou ke konfiguraci se spojí mezerou, takže lze použít i:

```powershell
cargo run -- READ NOW
```

Aplikace odešle text `READ NOW` zakončený jedním znakem CR (`\r`). Config `sr_sender.conf` se hledá v aktuálním adresáři nebo vedle EXE.

## Release build

```powershell
cargo build --release
.\target\release\sr_sender.exe "READ"
```
