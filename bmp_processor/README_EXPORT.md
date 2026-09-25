# BMP Processor v1.0 - Export

**Git Commit:** 661a7b7 - "Add STDOUT binary output functionality"  
**Export Date:** 02.02.2026  
**Status:** ✅ Kompletní funkční verze

## Co je v tomto exportu

### Hlavní funkce
- ✅ BMP zpracování s zero dependencies
- ✅ Podpora 8bpp grayscale a 24bpp RGB formátů
- ✅ Kreslení zelených obdélníků podle souřadnic
- ✅ Strukturovaný binární formát (`--struct` režim)
- ✅ Pipeline podpora se stdin/stdout
- ✅ Timestampové výstupní soubory

### Podporované režimy
```bash
# Základní zpracování
bmp_processor file.bmp "x1-y1_x2-y2_x3-y3_x4-y4"

# Strukturovaný vstup ze stdin
bmp_processor --struct

# Výstup na stdout  
bmp_processor --struct --stdout
```

### Pipeline integrace
```bash
ftp_bmp_downloader server user pass bmp --struct | \
  bmp_processor --struct --stdout > output.bmp
```

### Příjem z keyence_collector přes socket

Pokud se `bmp_processor.exe` spustí bez argumentů, poslouchá na TCP portu `9100`.
Na Windows zároveň automaticky spustí `keyence_collector.exe` ze stejné složky,
pokud tento proces ještě neběží. Obě EXE a `keyence-collector.conf` proto umístěte
do jednoho adresáře.
`keyence_collector` mu předává dva délkové rámce:

1. BMP data.
2. Název BMP souboru ve formátu obsahujícím čtyři body, například
   `scan.BMP100-100_600-100_600-400_100-400.BMP`.

Processor podle názvu vykreslí zelený čtyřúhelník a uloží výsledek jako
`read_data_matching_level.bmp` do aktuálního adresáře.

Po uložení processor odešle zpracovaný BMP také do GUI aplikace na
`127.0.0.1:9200`. GUI musí na tomto portu poslouchat. Adresu lze změnit:

```powershell
$env:BMP_GUI_ADDRESS = "192.168.210.50:9200"
\.\bmp_processor.exe
```

GUI přijme BMP rámec a za ním metadata rámec. Každý rámec má 8 bajtů s délkou
jako unsigned 64-bit big-endian a potom přesně tolik dat. Metadata obsahují
například `readdata:matchinglevel` a zobrazí se v titulku okna.
Pokud GUI neběží, processor pouze vypíše upozornění a
zpracovaný soubor zůstane uložený na disku.

Socketový průchod collectoru a processoru je diskless. Dočasný `incoming.bmp`
se nevytváří. Zpracovaný BMP se uloží pouze při nastavení například:

```powershell
$env:BMP_LOG_DIR = "C:\tiskarna\bmp-log"
```

### Samostatné GUI

Projekt obsahuje také `bmp_gui.exe`, které zobrazuje poslední zpracovaný obrázek.
Spusťte ho před processoru:

```powershell
.\bmp_gui.exe
.\bmp_processor.exe
```

GUI poslouchá na `127.0.0.1:9200` a po každém přijatém BMP obnoví obraz v okně.
Adresu lze změnit stejně jako u processoru pomocí proměnné `BMP_GUI_ADDRESS`.

```powershell
.\bmp_processor.exe
```

## Kompilace a spuštění

```bash
cargo build --release
./target/release/bmp_processor.exe [options]
```

## Testování

Export byl úspěšně zkompilován a otestován.

---
**Poznámka:** Toto je čistý export bez git historie - obsahuje pouze zdrojové soubory z konkrétního commitu.