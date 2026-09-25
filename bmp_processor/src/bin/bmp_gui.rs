use minifb::{Key, Window, WindowOptions};
use std::env;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

const DEFAULT_ADDRESS: &str = "127.0.0.1:9200";
const MAX_IMAGE_SIZE: u64 = 64 * 1024 * 1024;

fn main() -> Result<(), String> {
    let address = env::var("BMP_GUI_ADDRESS").unwrap_or_else(|_| DEFAULT_ADDRESS.to_string());
    let listener = TcpListener::bind(&address)
        .map_err(|error| format!("Nelze otevřít GUI port {address}: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("Nelze nastavit GUI listener: {error}"))?;

    let mut window = Window::new("Keyence BMP Viewer", 376, 240, WindowOptions::default())
        .map_err(|error| format!("Nelze vytvořit GUI okno: {error}"))?;
    window.set_target_fps(30);
    let mut buffer = vec![0_u32; 376 * 240];

    println!("BMP GUI poslouchá na {address}");
    while window.is_open() && !window.is_key_down(Key::Escape) {
        match listener.accept() {
            Ok((mut stream, peer)) => {
                if let Err(error) = stream.set_nonblocking(false) {
                    eprintln!("GUI: nelze nastavit blokující spojení od {peer}: {error}");
                    continue;
                }
                match read_frame(&mut stream) {
                    Ok(image) => match read_frame(&mut stream).and_then(|metadata| {
                        String::from_utf8(metadata)
                            .map_err(|error| format!("metadata nejsou UTF-8: {error}"))
                    }) {
                        Ok(metadata) => match decode_bmp(&image) {
                            Ok((width, height, pixels)) => {
                                if (width, height) != window.get_size() {
                                    eprintln!(
                                        "GUI: obrázek má rozměr {width}x{height}, očekáváno {}x{}",
                                        window.get_size().0,
                                        window.get_size().1
                                    );
                                    continue;
                                }
                                window.set_title("Načítání nového BMP...");
                                buffer.fill(0x202020);
                                window
                                    .update_with_buffer(&buffer, width, height)
                                    .map_err(|error| format!("Nelze zobrazit refresh: {error}"))?;
                                thread::sleep(Duration::from_millis(80));
                                window.set_title(&format!("{metadata}"));
                                buffer = pixels;
                                println!("GUI: přijat BMP od {peer} ({width}x{height}), metadata: {metadata}");
                            }
                            Err(error) => eprintln!("GUI: neplatný BMP: {error}"),
                        },
                        Err(error) => eprintln!("GUI: chyba metadat od {peer}: {error}"),
                    },
                    Err(error) => eprintln!("GUI: chyba příjmu od {peer}: {error}"),
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(error) => return Err(format!("GUI listener selhal: {error}")),
        }

        window
            .update_with_buffer(&buffer, window.get_size().0, window.get_size().1)
            .map_err(|error| format!("Nelze vykreslit BMP: {error}"))?;
    }
    Ok(())
}

fn read_frame(stream: &mut TcpStream) -> Result<Vec<u8>, String> {
    let mut length_bytes = [0_u8; 8];
    stream
        .read_exact(&mut length_bytes)
        .map_err(|error| format!("nelze načíst délku BMP: {error}"))?;
    let length = u64::from_be_bytes(length_bytes);
    if length == 0 || length > MAX_IMAGE_SIZE {
        return Err(format!("neplatná velikost BMP: {length}"));
    }
    let mut image = vec![0_u8; length as usize];
    stream
        .read_exact(&mut image)
        .map_err(|error| format!("nelze načíst BMP data: {error}"))?;
    Ok(image)
}

fn decode_bmp(data: &[u8]) -> Result<(usize, usize, Vec<u32>), String> {
    if data.len() < 54 || &data[..2] != b"BM" {
        return Err("chybí BMP hlavička".to_string());
    }
    let offset = read_u32(data, 10)? as usize;
    let width = read_u32(data, 18)? as usize;
    let signed_height = i32::from_le_bytes(data[22..26].try_into().unwrap());
    let height = signed_height.unsigned_abs() as usize;
    let bits = u16::from_le_bytes(data[28..30].try_into().unwrap());
    if width == 0 || height == 0 || !matches!(bits, 24 | 32) {
        return Err(format!("nepodporovaný BMP: {width}x{height}, {bits} bpp"));
    }

    let bytes_per_pixel = (bits / 8) as usize;
    let row_size = (width * bytes_per_pixel + 3) & !3;
    let mut pixels = vec![0_u32; width * height];
    for y in 0..height {
        let source_y = if signed_height > 0 { height - 1 - y } else { y };
        let row_start = offset + source_y * row_size;
        for x in 0..width {
            let index = row_start + x * bytes_per_pixel;
            if index + 2 >= data.len() {
                return Err("BMP obsahuje méně pixelů, než uvádí hlavička".to_string());
            }
            let blue = data[index] as u32;
            let green = data[index + 1] as u32;
            let red = data[index + 2] as u32;
            pixels[y * width + x] = (red << 16) | (green << 8) | blue;
        }
    }
    Ok((width, height, pixels))
}

fn read_u32(data: &[u8], offset: usize) -> Result<u32, String> {
    let bytes = data
        .get(offset..offset + 4)
        .ok_or_else(|| "neúplná BMP hlavička".to_string())?;
    Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
}
