use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

#[derive(Debug)]
struct Config {
    reader_address: SocketAddr,
    connect_timeout: Duration,
}

#[derive(Debug)]
struct ConfigError(String);

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for ConfigError {}

fn main() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let (config_path, command_parts) = if arguments
        .first()
        .is_some_and(|path| std::path::Path::new(path).is_file())
    {
        (arguments[0].clone(), arguments[1..].to_vec())
    } else {
        (find_config_path()?, arguments)
    };
    if command_parts.is_empty() {
        return Err("Chybí TCP příkaz. Použití: sr_sender.exe \"READ\"".into());
    }

    let config = Config::from_file(&config_path)?;
    let command = format!("{}\r", command_parts.join(" ").trim_end_matches('\r'));
    send_command(&config, command.as_bytes())?;
    println!(
        "Příkaz odeslán na {} ({} B)",
        config.reader_address,
        command.len()
    );
    Ok(())
}

fn find_config_path() -> Result<String, Box<dyn Error>> {
    let config_name = "sr_sender.conf";
    let mut candidates = Vec::new();
    if let Ok(current_dir) = env::current_dir() {
        candidates.push(current_dir.join(config_name));
    }
    if let Ok(executable) = env::current_exe() {
        if let Some(directory) = executable.parent() {
            let path = directory.join(config_name);
            if !candidates.contains(&path) {
                candidates.push(path);
            }
        }
    }
    candidates
        .into_iter()
        .find(|path| path.is_file())
        .map(|path| path.to_string_lossy().into_owned())
        .ok_or_else(|| "Soubor sr_sender.conf nebyl nalezen v aktuálním adresáři ani vedle EXE. Vytvořte ho podle sr_sender.conf.example.".into())
}

impl Config {
    fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let contents = fs::read_to_string(path)
            .map_err(|error| ConfigError(format!("nelze načíst konfiguraci '{path}': {error}")))?;
        let mut values = HashMap::new();
        for (line_number, line) in contents.lines().enumerate() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let (key, value) = line.split_once('=').ok_or_else(|| {
                ConfigError(format!("{path}:{}: očekáváno key=value", line_number + 1))
            })?;
            values.insert(key.trim().to_string(), value.trim().to_string());
        }

        let reader_address = required(&values, "reader_address")?
            .parse()
            .map_err(|error| ConfigError(format!("neplatná reader_address: {error}")))?;
        let timeout_ms: u64 = values
            .get("connect_timeout_ms")
            .map(|value| value.parse())
            .transpose()
            .map_err(|error| ConfigError(format!("neplatný connect_timeout_ms: {error}")))?
            .unwrap_or(5000);
        if timeout_ms == 0 {
            return Err(ConfigError("connect_timeout_ms musí být větší než 0".to_string()).into());
        }

        Ok(Self {
            reader_address,
            connect_timeout: Duration::from_millis(timeout_ms),
        })
    }
}

fn required(values: &HashMap<String, String>, key: &str) -> Result<String, Box<dyn Error>> {
    values
        .get(key)
        .filter(|value| !value.is_empty())
        .cloned()
        .ok_or_else(|| ConfigError(format!("chybí konfigurace {key}")).into())
}

fn send_command(config: &Config, command: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect_timeout(&config.reader_address, config.connect_timeout)
        .map_err(|error| {
            format!(
                "nelze se připojit ke čtečce {}: {error}",
                config.reader_address
            )
        })?;
    stream
        .write_all(command)
        .map_err(|error| format!("nelze odeslat příkaz: {error}"))?;
    stream
        .shutdown(std::net::Shutdown::Write)
        .map_err(|error| format!("nelze uzavřít TCP výstup: {error}"))?;
    Ok(())
}
