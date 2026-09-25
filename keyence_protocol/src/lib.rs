use std::io::{self, Read, Write};

/// Version of the collector-to-processor parent protocol.
pub const PROTOCOL_VERSION: u16 = 1;
/// Upper bound for one length-prefixed payload.
pub const MAX_FRAME_SIZE: u64 = 64 * 1024 * 1024;

/// Writes the protocol version as the first length-prefixed frame.
pub fn write_protocol_version<W: Write>(writer: &mut W) -> io::Result<()> {
    write_frame(writer, &PROTOCOL_VERSION.to_be_bytes())
}

/// Reads and validates the first protocol-version frame.
pub fn read_protocol_version<R: Read>(reader: &mut R) -> io::Result<()> {
    let version = read_frame(reader)?;
    if version.len() != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid protocol version frame",
        ));
    }
    let version = u16::from_be_bytes([version[0], version[1]]);
    if version != PROTOCOL_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported protocol version: {version}"),
        ));
    }
    Ok(())
}

/// Writes one frame: an 8-byte big-endian length followed by its payload.
pub fn write_frame<W: Write>(writer: &mut W, data: &[u8]) -> io::Result<()> {
    let length = u64::try_from(data.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "frame is too large"))?;
    if length > MAX_FRAME_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "frame exceeds the protocol limit",
        ));
    }
    writer.write_all(&length.to_be_bytes())?;
    writer.write_all(data)
}

/// Reads one frame and rejects payloads larger than `MAX_FRAME_SIZE`.
pub fn read_frame<R: Read>(reader: &mut R) -> io::Result<Vec<u8>> {
    let mut length_bytes = [0_u8; 8];
    reader.read_exact(&mut length_bytes)?;
    let length = u64::from_be_bytes(length_bytes);
    if length > MAX_FRAME_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame exceeds the protocol limit",
        ));
    }
    let mut data = vec![0_u8; length as usize];
    reader.read_exact(&mut data)?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_frame() {
        let mut encoded = Vec::new();
        write_frame(&mut encoded, b"hello").unwrap();
        assert_eq!(read_frame(&mut encoded.as_slice()).unwrap(), b"hello");
    }

    #[test]
    fn supports_multiple_frames() {
        let mut encoded = Vec::new();
        write_frame(&mut encoded, b"bmp").unwrap();
        write_frame(&mut encoded, b"name").unwrap();
        let mut reader = encoded.as_slice();
        assert_eq!(read_frame(&mut reader).unwrap(), b"bmp");
        assert_eq!(read_frame(&mut reader).unwrap(), b"name");
    }

    #[test]
    fn rejects_oversized_frame_header() {
        let header = (MAX_FRAME_SIZE + 1).to_be_bytes();
        let mut reader = header.as_slice();
        let error = read_frame(&mut reader).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn round_trips_protocol_version() {
        let mut encoded = Vec::new();
        write_protocol_version(&mut encoded).unwrap();
        read_protocol_version(&mut encoded.as_slice()).unwrap();
    }
}
