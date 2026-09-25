# Keyence Suite

Cargo workspace pro aplikace kolem Keyence SR-750 a zpracování BMP obrazů.
Aplikace jsou samostatné binárky, ale sdílejí crate `keyence_protocol`, který
určuje framing a verzi interní komunikace collector -> processor.

## Projekty

| Projekt | Účel | Výstup |
| --- | --- | --- |
| `keyence_collector` | FTP příjem BMP ze čtečky a předání parent procesu | `keyence_collector.exe` |
| `bmp_processor` | Zpracování BMP, vykreslení zeleného čtyřúhelníku a předání GUI | `bmp_processor.exe` |
| `bmp_gui` | Zobrazení posledního zpracovaného BMP | `bmp_gui.exe` |
| `sr_sender` | Jednorázové odeslání TCP příkazu do SR-750 | `sr_sender.exe` |
| `keyence_protocol` | Společné rámce a kontrola verze protokolu | knihovna |

## Build a testy

Z kořene workspace:

```powershell
cargo fmt --all -- --check
cargo test --workspace --offline
cargo check --workspace --offline
cargo build --workspace --release --offline
```

Parametr `--offline` je vhodný v síti, kde Cargo nemůže ověřit TLS certifikát
nebo nemá přístup na `crates.io`. Vyžaduje dependencies v lokální Cargo cache.

## Provozní tok

```text
Keyence SR-750 --FTP--> keyence_collector --parent socket--> bmp_processor --GUI socket--> bmp_gui
```

Collector přijme BMP příkazem FTP `STOR`. Souřadnice se čtou z názvu souboru,
například:

```text
6845415012252701020042700011:68|246-294_343-287_351-383_254-391.BMP
```

Část před `|` obsahuje metadata `readdata:matchinglevel`. Čtyři dvojice za `|`
jsou souřadnice čtyřúhelníku.

## Parent protokol v1

Collector posílá processoru přes `parent_socket` tři length-prefixed rámce:

1. Verze protokolu: payload jsou 2 bajty `u16` big-endian.
2. BMP data.
3. Název BMP souboru jako UTF-8.

Každý rámec má formát:

```text
8 bajtů: délka payloadu jako unsigned 64-bit big-endian
N bajtů: payload
```

Maximální velikost jednoho rámce je 64 MiB. Verzi a čtení/zápis rámců implementuje
výhradně `keyence_protocol`; aplikace si je nekopírují samostatně.

Po změně protokolu je nutné nasadit současně kompatibilní verze collectoru a
processoru. Nekompatibilní verze odmítne processor kontrolou `PROTOCOL_VERSION`.

## Windows nasazení

Do jedné provozní složky zkopírujte kompatibilní EXE a konfiguraci:

```text
keyence_collector.exe
bmp_processor.exe
bmp_gui.exe
sr_sender.exe
keyence-collector.conf
sr_sender.conf
```

Doporučené spuštění:

```powershell
.\bmp_gui.exe
.\bmp_processor.exe
```

Processor na Windows při startu bez argumentů poslouchá na parent TCP portu
`9100` a může automaticky spustit `keyence_collector.exe` ze stejné složky.
GUI standardně poslouchá na `127.0.0.1:9200`.

## Síťová konfigurace SR-750

Pro typické zapojení:

```text
PC:      192.168.210.1
SR-750:  192.168.210.200
FTP PC:  192.168.210.1:21
```

V `keyence-collector.conf` musí být `ftp_bind=0.0.0.0` a
`ftp_advertise=192.168.210.1`, pokud se čtečka připojuje přes LAN.
Povolte příslušný FTP port ve Windows Firewallu.

## Volitelné logování BMP

Socketový tok je diskless. BMP se ukládá pouze při nastavení:

```powershell
$env:BMP_LOG_DIR = "C:\tiskarna\bmp-log"
.\bmp_processor.exe
```

Bez této proměnné zůstává zpracovaný obrázek v paměti a je předán pouze GUI.

## Verze

Verze protokolu je oddělená od verzí jednotlivých aplikací. Při změně pořadí
nebo významu rámců zvyšte `PROTOCOL_VERSION` a sestavte společně všechny
komunikující binárky.
