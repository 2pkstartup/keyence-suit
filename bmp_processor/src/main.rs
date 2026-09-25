/*
 * MINIMÁLNÍ BMP PROCESSOR - ZERO DEPENDENCIES!
 * =============================================
 *
 * Vlastní BMP čtení/zápis - pouze std knihovna!
 * BMP je nejjednodušší formát - žádné komprese, přímé pixely
 */

use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

// Konstantní rozměry pro maximální optimalizaci
const IMAGE_WIDTH: u32 = 376;
const IMAGE_HEIGHT: u32 = 240;
const DEFAULT_GUI_ADDRESS: &str = "127.0.0.1:9200";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        if let Err(error) = run_socket_processor("0.0.0.0:9100") {
            eprintln!("Socket processor error: {}", error);
            std::process::exit(1);
        }
        return;
    }

    if args.len() < 2 || args.len() > 3 {
        print_help();
        std::process::exit(1);
    }

    let bmp_file = &args[1];
    let use_stdout = args.len() == 3 && args[2] == "--stdout";

    if !use_stdout {
        println!("BMP PROCESSOR v3.0 - ZERO DEPENDENCIES!");
        println!("Processing file: {}", bmp_file);
    }

    let result = if use_stdout {
        process_bmp_to_stdout(bmp_file)
    } else {
        process_bmp_image_minimal(bmp_file)
    };

    match result {
        Ok(()) => {
            if !use_stdout {
                println!("✅ BMP processing completed successfully!");
            }
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn process_bmp_image_minimal(input_file: &str) -> Result<(), String> {
    // Parse coordinates from filename
    let points = parse_coordinates_from_filename(input_file)?;
    println!("Parsed {} points: {:?}", points.len(), points);

    // Create output filename with timestamp only
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let output_file = format!("{}.bmp", timestamp);

    // Process BMP with zero dependencies
    process_bmp_native(input_file, &output_file, &points)?;
    println!("Added green rectangle and saved as: {}", output_file);

    Ok(())
}

fn process_bmp_to_stdout(input_file: &str) -> Result<(), String> {
    // Parse coordinates from filename
    let points = parse_coordinates_from_filename(input_file)?;

    // Process BMP and output to STDOUT
    process_bmp_native_to_stdout(input_file, &points)
}

fn process_bmp_native(
    input_file: &str,
    output_file: &str,
    points: &[(u32, u32)],
) -> Result<(), String> {
    // Číst BMP soubor raw
    let mut file = File::open(input_file).map_err(|e| format!("Cannot open input file: {}", e))?;

    // Číst BMP header (54 bajtů)
    let mut bmp_data = Vec::new();
    file.read_to_end(&mut bmp_data)
        .map_err(|e| format!("Cannot read BMP file: {}", e))?;

    process_bmp_data(&bmp_data, output_file, points)
}

fn process_bmp_data(
    bmp_data: &[u8],
    output_file: &str,
    points: &[(u32, u32)],
) -> Result<(), String> {
    if bmp_data.len() < 54 {
        return Err("Invalid BMP file - too short".to_string());
    }

    let processed = process_bmp_data_to_memory(bmp_data, points)?;
    File::create(output_file)
        .and_then(|mut file| file.write_all(&processed))
        .map_err(|error| format!("Cannot create output file: {error}"))?;
    Ok(())
}

fn process_bmp_data_to_memory(bmp_data: &[u8], points: &[(u32, u32)]) -> Result<Vec<u8>, String> {
    if bmp_data.len() < 54 {
        return Err("Invalid BMP file - too short".to_string());
    }

    if &bmp_data[0..2] != b"BM" {
        return Err("Not a valid BMP file".to_string());
    }

    // Získat offset dat z headeru (byte 10-13)
    let data_offset =
        u32::from_le_bytes([bmp_data[10], bmp_data[11], bmp_data[12], bmp_data[13]]) as usize;

    // Získat bits per pixel (byte 28-29)
    let bits_per_pixel = u16::from_le_bytes([bmp_data[28], bmp_data[29]]);

    println!(
        "BMP info: data_offset={}, bits_per_pixel={}",
        data_offset, bits_per_pixel
    );

    // Pro grayscale BMP (8 bpp), převést na RGB (24 bpp)
    if bits_per_pixel == 8 {
        convert_grayscale_to_rgb_bmp(bmp_data, points, data_offset)
    } else if bits_per_pixel == 24 {
        process_rgb_bmp(bmp_data, points, data_offset)
    } else {
        Err(format!(
            "Unsupported BMP format: {} bits per pixel",
            bits_per_pixel
        ))
    }
}

fn process_bmp_native_to_stdout(input_file: &str, points: &[(u32, u32)]) -> Result<(), String> {
    // Číst BMP soubor raw
    let mut file = File::open(input_file).map_err(|e| format!("Cannot open input file: {}", e))?;

    // Číst BMP data
    let mut bmp_data = Vec::new();
    file.read_to_end(&mut bmp_data)
        .map_err(|e| format!("Cannot read BMP file: {}", e))?;

    if bmp_data.len() < 54 {
        return Err("Invalid BMP file - too short".to_string());
    }

    // Validace BMP signature
    if &bmp_data[0..2] != b"BM" {
        return Err("Not a valid BMP file".to_string());
    }

    // Získat offset dat z headeru (byte 10-13)
    let data_offset =
        u32::from_le_bytes([bmp_data[10], bmp_data[11], bmp_data[12], bmp_data[13]]) as usize;

    // Získat bits per pixel (byte 28-29)
    let bits_per_pixel = u16::from_le_bytes([bmp_data[28], bmp_data[29]]);

    // Pro grayscale BMP (8 bpp), převést na RGB (24 bpp)
    if bits_per_pixel == 8 {
        convert_grayscale_to_rgb_bmp_stdout(&bmp_data, points, data_offset)
    } else if bits_per_pixel == 24 {
        process_rgb_bmp_stdout(&bmp_data, points, data_offset)
    } else {
        Err(format!(
            "Unsupported BMP format: {} bits per pixel",
            bits_per_pixel
        ))
    }
}

fn convert_grayscale_to_rgb_bmp(
    input_data: &[u8],
    points: &[(u32, u32)],
    data_offset: usize,
) -> Result<Vec<u8>, String> {
    // Vytvořit RGB pixely z grayscale
    let mut rgb_pixels = vec![0u8; (IMAGE_WIDTH * IMAGE_HEIGHT * 3) as usize];

    // Padding pro řádky (BMP řádky musí být zarovnané na 4 bajty)
    let input_row_size = ((IMAGE_WIDTH + 3) & !3) as usize; // 8bpp grayscale s paddingem
    let _output_row_size = ((IMAGE_WIDTH * 3 + 3) & !3) as usize; // 24bpp RGB s paddingem

    // Převést grayscale na RGB
    for y in 0..IMAGE_HEIGHT {
        for x in 0..IMAGE_WIDTH {
            let input_offset =
                data_offset + (IMAGE_HEIGHT - 1 - y) as usize * input_row_size + x as usize;
            let output_offset = (y * IMAGE_WIDTH + x) as usize * 3;

            if input_offset < input_data.len() {
                let gray = input_data[input_offset];
                rgb_pixels[output_offset] = gray; // B
                rgb_pixels[output_offset + 1] = gray; // G
                rgb_pixels[output_offset + 2] = gray; // R
            }
        }
    }

    // Nakreslit zelený obdélník
    draw_green_rectangle(&mut rgb_pixels, points)?;

    // Uložit RGB BMP
    Ok(encode_rgb_bmp(&rgb_pixels))
}

fn process_rgb_bmp(
    input_data: &[u8],
    points: &[(u32, u32)],
    data_offset: usize,
) -> Result<Vec<u8>, String> {
    // Kopírovat RGB pixely
    let input_row_size = ((IMAGE_WIDTH * 3 + 3) & !3) as usize;
    let mut rgb_pixels = vec![0u8; (IMAGE_WIDTH * IMAGE_HEIGHT * 3) as usize];

    for y in 0..IMAGE_HEIGHT {
        for x in 0..IMAGE_WIDTH {
            let input_offset =
                data_offset + (IMAGE_HEIGHT - 1 - y) as usize * input_row_size + x as usize * 3;
            let output_offset = (y * IMAGE_WIDTH + x) as usize * 3;

            if input_offset + 2 < input_data.len() {
                rgb_pixels[output_offset] = input_data[input_offset]; // B
                rgb_pixels[output_offset + 1] = input_data[input_offset + 1]; // G
                rgb_pixels[output_offset + 2] = input_data[input_offset + 2]; // R
            }
        }
    }

    // Nakreslit zelený obdélník
    draw_green_rectangle(&mut rgb_pixels, points)?;

    // Uložit RGB BMP
    Ok(encode_rgb_bmp(&rgb_pixels))
}

fn draw_green_rectangle(rgb_pixels: &mut [u8], points: &[(u32, u32)]) -> Result<(), String> {
    let green = (0, 255, 0); // RGB zelená
    let line_thickness = 2u32;

    // Nakreslit čáry mezi po sobě jdoucími body
    for i in 0..4 {
        let (x1, y1) = points[i];
        let (x2, y2) = points[(i + 1) % 4];

        draw_thick_line_rgb(
            rgb_pixels,
            x1 as i32,
            y1 as i32,
            x2 as i32,
            y2 as i32,
            green,
            line_thickness,
        );
    }

    Ok(())
}

fn draw_thick_line_rgb(
    pixels: &mut [u8],
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    color: (u8, u8, u8),
    thickness: u32,
) {
    // Bresenham's line algorithm
    let mut x = x1;
    let mut y = y1;
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let x_inc = if x1 < x2 { 1 } else { -1 };
    let y_inc = if y1 < y2 { 1 } else { -1 };
    let mut error = dx - dy;

    let half_thick = (thickness as i32) / 2;

    loop {
        // Thick pixel drawing
        for dx in -half_thick..=half_thick {
            for dy in -half_thick..=half_thick {
                let px = x + dx;
                let py = y + dy;

                if px >= 0 && py >= 0 && (px as u32) < IMAGE_WIDTH && (py as u32) < IMAGE_HEIGHT {
                    let offset = (py as u32 * IMAGE_WIDTH + px as u32) as usize * 3;
                    pixels[offset] = color.2; // R
                    pixels[offset + 1] = color.1; // G
                    pixels[offset + 2] = color.0; // B
                }
            }
        }

        if x == x2 && y == y2 {
            break;
        }

        let error2 = error * 2;
        if error2 > -dy {
            error -= dy;
            x += x_inc;
        }
        if error2 < dx {
            error += dx;
            y += y_inc;
        }
    }
}

fn encode_rgb_bmp(rgb_pixels: &[u8]) -> Vec<u8> {
    let row_size = ((IMAGE_WIDTH * 3 + 3) & !3) as usize;
    let image_size = (row_size * IMAGE_HEIGHT as usize) as u32;
    let file_size = 54 + image_size;
    let mut data = Vec::with_capacity(file_size as usize);

    data.extend_from_slice(b"BM");
    data.extend_from_slice(&file_size.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(&54u32.to_le_bytes());
    data.extend_from_slice(&40u32.to_le_bytes());
    data.extend_from_slice(&(IMAGE_WIDTH as i32).to_le_bytes());
    data.extend_from_slice(&(IMAGE_HEIGHT as i32).to_le_bytes());
    data.extend_from_slice(&1u16.to_le_bytes());
    data.extend_from_slice(&24u16.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(&image_size.to_le_bytes());
    data.extend_from_slice(&2835u32.to_le_bytes());
    data.extend_from_slice(&2835u32.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());

    let padding = row_size - IMAGE_WIDTH as usize * 3;
    for y in (0..IMAGE_HEIGHT).rev() {
        for x in 0..IMAGE_WIDTH {
            let offset = (y * IMAGE_WIDTH + x) as usize * 3;
            data.extend_from_slice(&[
                rgb_pixels[offset + 2],
                rgb_pixels[offset + 1],
                rgb_pixels[offset],
            ]);
        }
        data.resize(data.len() + padding, 0);
    }
    data
}

fn save_rgb_bmp_to_stdout(rgb_pixels: &[u8]) -> Result<(), String> {
    use std::io::{self, BufWriter};

    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    let row_size = ((IMAGE_WIDTH * 3 + 3) & !3) as usize; // 4-byte alignment
    let image_size = (row_size * IMAGE_HEIGHT as usize) as u32;
    let file_size = 54 + image_size; // 54 = headers size

    // BMP File Header (14 bytes)
    writer
        .write_all(b"BM")
        .map_err(|e| format!("STDOUT write error: {}", e))?;
    writer
        .write_all(&file_size.to_le_bytes())
        .map_err(|e| format!("STDOUT write error: {}", e))?;
    writer
        .write_all(&0u32.to_le_bytes())
        .map_err(|e| format!("STDOUT write error: {}", e))?;
    writer
        .write_all(&54u32.to_le_bytes())
        .map_err(|e| format!("STDOUT write error: {}", e))?;

    // BMP Info Header (40 bytes)
    writer
        .write_all(&40u32.to_le_bytes())
        .map_err(|e| format!("STDOUT write error: {}", e))?;
    writer
        .write_all(&(IMAGE_WIDTH as i32).to_le_bytes())
        .map_err(|e| format!("STDOUT write error: {}", e))?;
    writer
        .write_all(&(IMAGE_HEIGHT as i32).to_le_bytes())
        .map_err(|e| format!("STDOUT write error: {}", e))?;
    writer
        .write_all(&1u16.to_le_bytes())
        .map_err(|e| format!("STDOUT write error: {}", e))?;
    writer
        .write_all(&24u16.to_le_bytes())
        .map_err(|e| format!("STDOUT write error: {}", e))?;
    writer
        .write_all(&0u32.to_le_bytes())
        .map_err(|e| format!("STDOUT write error: {}", e))?;
    writer
        .write_all(&image_size.to_le_bytes())
        .map_err(|e| format!("STDOUT write error: {}", e))?;
    writer
        .write_all(&2835u32.to_le_bytes())
        .map_err(|e| format!("STDOUT write error: {}", e))?;
    writer
        .write_all(&2835u32.to_le_bytes())
        .map_err(|e| format!("STDOUT write error: {}", e))?;
    writer
        .write_all(&0u32.to_le_bytes())
        .map_err(|e| format!("STDOUT write error: {}", e))?;
    writer
        .write_all(&0u32.to_le_bytes())
        .map_err(|e| format!("STDOUT write error: {}", e))?;

    // Pixel data (bottom-up, left-to-right, with padding)
    let padding = row_size - (IMAGE_WIDTH as usize * 3);
    for y in (0..IMAGE_HEIGHT).rev() {
        // BMP is bottom-up
        for x in 0..IMAGE_WIDTH {
            let offset = (y * IMAGE_WIDTH + x) as usize * 3;
            writer
                .write_all(&[
                    rgb_pixels[offset + 2],
                    rgb_pixels[offset + 1],
                    rgb_pixels[offset],
                ])
                .map_err(|e| format!("STDOUT write error: {}", e))?; // BGR
        }
        // Add padding
        for _ in 0..padding {
            writer
                .write_all(&[0])
                .map_err(|e| format!("STDOUT write error: {}", e))?;
        }
    }

    writer
        .flush()
        .map_err(|e| format!("STDOUT flush error: {}", e))?;
    Ok(())
}

fn parse_coordinates_from_filename(filename: &str) -> Result<Vec<(u32, u32)>, String> {
    // SR-750 format: metadata|x1-y1_x2-y2_x3-y3_x4-y4.BMP.
    let file_name = std::path::Path::new(filename)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(filename);

    let coordinates = file_name
        .split_once('|')
        .map(|(_, coordinates)| coordinates)
        .ok_or_else(|| format!("Filename does not contain '|': {}", file_name))?;
    let stem = coordinates
        .strip_suffix(".BMP")
        .or_else(|| coordinates.strip_suffix(".bmp"))
        .unwrap_or(coordinates);
    let coordinate_tokens: Vec<&str> = stem.split('_').collect();
    if coordinate_tokens.len() != 4 {
        return Err(format!("Expected 4 coordinate pairs: {}", file_name));
    }

    let mut points = Vec::new();
    for point_str in coordinate_tokens {
        let (x_str, y_str) = point_str
            .split_once('-')
            .ok_or_else(|| format!("Invalid coordinate pair: {}", point_str))?;
        let x_orig: f32 = x_str
            .parse()
            .map_err(|_| format!("Invalid X coordinate: {}", x_str))?;
        let y_orig: f32 = y_str
            .parse()
            .map_err(|_| format!("Invalid Y coordinate: {}", y_str))?;
        points.push(((x_orig * 0.5) as u32, (y_orig * 0.5) as u32));
    }
    Ok(points)
}

#[cfg(test)]
mod tests {
    use super::{output_filename, parse_coordinates_from_filename};

    #[test]
    fn parses_sr750_filename_coordinates_after_separator() {
        let points = parse_coordinates_from_filename(
            "6845415012252701020042700011|66246-295_344-287_351-384_254-391.BMP",
        )
        .unwrap();
        assert_eq!(
            points,
            vec![(33123, 147), (172, 143), (175, 192), (127, 195)]
        );
    }

    #[test]
    fn rejects_filename_without_coordinate_separator() {
        assert!(parse_coordinates_from_filename("image.BMP").is_err());
    }

    #[test]
    fn preserves_read_data_and_matching_level_in_output_name() {
        let output = output_filename("readdata:matchinglevel|10-20_30-20_30-40_10-40.BMP").unwrap();
        assert_eq!(output, "readdata_matchinglevel.bmp");
    }
}

fn print_help() {
    println!("BMP PROCESSOR v3.0 - ZERO DEPENDENCIES EDITION!");
    println!("===============================================");
    println!();
    println!("POUŽITÍ:");
    println!("  bmp_processor.exe <bmp_soubor>");
    println!("  bmp_processor.exe <bmp_soubor> --stdout");
    println!();
    println!("VÝHODY:");
    println!("  ✅ ZERO external dependencies (jen std)");
    println!("  ✅ Extrémně rychlé (native BMP I/O)");
    println!("  ✅ Malý binary (200KB místo 6MB)");
    println!("  ✅ Podporuje grayscale i RGB BMP");
    println!("  ✅ 31x menší než image-crate verze");
    println!("  ✅ STDOUT output pro pipe operace");
    println!();
    println!("PŘÍKLADY:");
    println!("  bmp_processor.exe \"test.BMP123-456_789-123_456-789_111-222.BMP\"");
    println!("  -> Výstup: timestamp.bmp s ZELENÝM obdélníkem");
    println!();
    println!(
        "  bmp_processor.exe \"test.BMP123-456_789-123_456-789_111-222.BMP\" --stdout > output.bmp"
    );
    println!("  -> Výstup: binární BMP data na STDOUT (pro pipes)");
    println!("  -> Původní soubor zůstane nezměněn");
}

fn convert_grayscale_to_rgb_bmp_stdout(
    input_data: &[u8],
    points: &[(u32, u32)],
    data_offset: usize,
) -> Result<(), String> {
    // Vytvořit RGB pixely z grayscale
    let mut rgb_pixels = vec![0u8; (IMAGE_WIDTH * IMAGE_HEIGHT * 3) as usize];

    // Padding pro řádky (BMP řádky musí být zarovnané na 4 bajty)
    let input_row_size = ((IMAGE_WIDTH + 3) & !3) as usize; // 8bpp grayscale s paddingem

    // Převést grayscale na RGB
    for y in 0..IMAGE_HEIGHT {
        for x in 0..IMAGE_WIDTH {
            let input_offset =
                data_offset + (IMAGE_HEIGHT - 1 - y) as usize * input_row_size + x as usize;
            let output_offset = (y * IMAGE_WIDTH + x) as usize * 3;

            if input_offset < input_data.len() {
                let gray = input_data[input_offset];
                rgb_pixels[output_offset] = gray; // B
                rgb_pixels[output_offset + 1] = gray; // G
                rgb_pixels[output_offset + 2] = gray; // R
            }
        }
    }

    // Nakreslit zelený obdélník
    draw_green_rectangle(&mut rgb_pixels, points)?;

    // Výstup na STDOUT
    save_rgb_bmp_to_stdout(&rgb_pixels)
}

fn process_rgb_bmp_stdout(
    input_data: &[u8],
    points: &[(u32, u32)],
    data_offset: usize,
) -> Result<(), String> {
    // Kopírovat RGB pixely
    let input_row_size = ((IMAGE_WIDTH * 3 + 3) & !3) as usize;
    let mut rgb_pixels = vec![0u8; (IMAGE_WIDTH * IMAGE_HEIGHT * 3) as usize];

    for y in 0..IMAGE_HEIGHT {
        for x in 0..IMAGE_WIDTH {
            let input_offset =
                data_offset + (IMAGE_HEIGHT - 1 - y) as usize * input_row_size + x as usize * 3;
            let output_offset = (y * IMAGE_WIDTH + x) as usize * 3;

            if input_offset + 2 < input_data.len() {
                rgb_pixels[output_offset] = input_data[input_offset]; // B
                rgb_pixels[output_offset + 1] = input_data[input_offset + 1]; // G
                rgb_pixels[output_offset + 2] = input_data[input_offset + 2]; // R
            }
        }
    }

    // Nakreslit zelený obdélník
    draw_green_rectangle(&mut rgb_pixels, points)?;

    // Výstup na STDOUT
    save_rgb_bmp_to_stdout(&rgb_pixels)
}

fn run_socket_processor(address: &str) -> Result<(), String> {
    let listener = TcpListener::bind(address)
        .map_err(|error| format!("Cannot bind socket {address}: {error}"))?;
    println!("BMP processor listening on {address}");
    start_collector_if_needed()?;

    for connection in listener.incoming() {
        let mut stream = connection.map_err(|error| format!("Accept error: {error}"))?;
        if let Err(error) = process_socket_connection(&mut stream) {
            eprintln!("Socket request failed: {error}");
        }
    }
    Ok(())
}

fn start_collector_if_needed() -> Result<(), String> {
    #[cfg(not(windows))]
    {
        println!("Automatic collector start is available only on Windows");
        return Ok(());
    }

    #[cfg(windows)]
    {
        if collector_is_running() {
            println!("keyence_collector.exe is already running");
            return Ok(());
        }

        let processor_dir = env::current_exe()
            .map_err(|error| format!("Cannot locate BMP processor: {error}"))?
            .parent()
            .map(PathBuf::from)
            .ok_or_else(|| "BMP processor directory is unavailable".to_string())?;
        let collector = processor_dir.join("keyence_collector.exe");
        if !collector.is_file() {
            return Err(format!(
                "keyence_collector.exe was not found next to BMP processor: {}",
                collector.display()
            ));
        }

        Command::new(&collector)
            .current_dir(&processor_dir)
            .spawn()
            .map_err(|error| format!("Cannot start {}: {error}", collector.display()))?;
        println!(
            "Started keyence_collector.exe from {}",
            processor_dir.display()
        );
        Ok(())
    }
}

#[cfg(windows)]
fn collector_is_running() -> bool {
    Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq keyence_collector.exe", "/NH"])
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .to_ascii_lowercase()
                .contains("keyence_collector.exe")
        })
        .unwrap_or(false)
}

fn process_socket_connection(stream: &mut TcpStream) -> Result<(), String> {
    let bmp = read_frame(stream)?;
    let name = String::from_utf8(read_frame(stream)?)
        .map_err(|error| format!("BMP filename is not UTF-8: {error}"))?;
    let points = parse_coordinates_from_filename(&name)?;
    let output_file = output_filename(&name)?;

    let processed_image = process_bmp_data_to_memory(&bmp, &points)?;
    save_processed_log(&output_file, &processed_image);
    send_processed_image_to_gui(&processed_image, &name);
    println!(
        "Processed {} ({} B), green rectangle is in memory{}",
        name,
        bmp.len(),
        if env::var_os("BMP_LOG_DIR").is_some() {
            " and was written to the log directory"
        } else {
            ""
        }
    );
    Ok(())
}

fn save_processed_log(output_file: &str, image: &[u8]) {
    let Ok(log_dir) = env::var("BMP_LOG_DIR") else {
        return;
    };
    let path = std::path::Path::new(&log_dir).join(output_file);
    if let Err(error) = std::fs::create_dir_all(&log_dir).and_then(|_| std::fs::write(&path, image))
    {
        eprintln!("BMP log zápis selhal pro {}: {error}", path.display());
    } else {
        println!("BMP log: uložen {}", path.display());
    }
}

fn send_processed_image_to_gui(image: &[u8], source_name: &str) {
    let address = env::var("BMP_GUI_ADDRESS").unwrap_or_else(|_| DEFAULT_GUI_ADDRESS.to_string());
    let address = match address.parse::<SocketAddr>() {
        Ok(address) => address,
        Err(error) => {
            eprintln!("GUI předání přeskočeno: neplatná adresa {address}: {error}");
            return;
        }
    };

    match TcpStream::connect_timeout(&address, std::time::Duration::from_secs(2)) {
        Ok(mut stream) => {
            let metadata = source_name
                .split_once('|')
                .map(|(metadata, _)| metadata)
                .unwrap_or(source_name);
            if let Err(error) = write_gui_frame(&mut stream, &image, metadata.as_bytes()) {
                eprintln!("GUI předání selhalo: {error}");
            } else {
                println!(
                    "GUI: odeslán zpracovaný BMP z {} ({} B)",
                    source_name,
                    image.len()
                );
            }
        }
        Err(error) => eprintln!(
            "GUI není dostupné na {address}; zpracovaný BMP zůstal pouze v paměti: {error}"
        ),
    }
}

fn write_gui_frame(stream: &mut TcpStream, image: &[u8], metadata: &[u8]) -> Result<(), String> {
    stream
        .write_all(&(image.len() as u64).to_be_bytes())
        .map_err(|error| format!("nelze odeslat délku BMP: {error}"))?;
    stream
        .write_all(image)
        .map_err(|error| format!("nelze odeslat BMP data: {error}"))?;
    stream
        .write_all(&(metadata.len() as u64).to_be_bytes())
        .map_err(|error| format!("nelze odeslat délku metadat: {error}"))?;
    stream
        .write_all(metadata)
        .map_err(|error| format!("nelze odeslat metadata: {error}"))?;
    stream
        .shutdown(std::net::Shutdown::Write)
        .map_err(|error| format!("nelze ukončit GUI spojení: {error}"))?;
    Ok(())
}

fn output_filename(filename: &str) -> Result<String, String> {
    let file_name = std::path::Path::new(filename)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(filename);
    let metadata = file_name
        .split_once('|')
        .map(|(metadata, _)| metadata)
        .ok_or_else(|| format!("Filename does not contain '|': {file_name}"))?;
    if metadata.is_empty() {
        return Err("Filename metadata before '|' is empty".to_string());
    }

    // Windows reserves ':' in file names; preserve both values with a safe separator.
    let safe_metadata = metadata.replace(':', "_");
    Ok(format!("{safe_metadata}.bmp"))
}

fn read_frame(stream: &mut TcpStream) -> Result<Vec<u8>, String> {
    let mut length = [0u8; 8];
    stream
        .read_exact(&mut length)
        .map_err(|error| format!("Cannot read frame length: {error}"))?;
    let length = u64::from_be_bytes(length);
    if length > 64 * 1024 * 1024 {
        return Err("Frame is too large".to_string());
    }
    let mut data = vec![0u8; length as usize];
    stream
        .read_exact(&mut data)
        .map_err(|error| format!("Cannot read frame: {error}"))?;
    Ok(data)
}
