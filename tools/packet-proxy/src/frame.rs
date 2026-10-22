use std::io::{self, Read, Write};

use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

const MAX_VAR_INT_BYTES: usize = 5;

pub fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

/// Reads a VarInt from the start of `bytes`: the value and how many bytes it took, or `None`
/// when `bytes` ends before the VarInt does.
pub fn read_var_int(bytes: &[u8]) -> io::Result<Option<(i32, usize)>> {
    let mut value: u32 = 0;
    for (index, byte) in bytes.iter().take(MAX_VAR_INT_BYTES).enumerate() {
        value |= u32::from(byte & 0x7f) << (7 * index);
        if byte & 0x80 == 0 {
            return Ok(Some((value as i32, index + 1)));
        }
    }

    if bytes.len() >= MAX_VAR_INT_BYTES {
        return Err(invalid("VarInt is longer than 5 bytes"));
    }
    Ok(None)
}

pub fn write_var_int(value: i32, output: &mut Vec<u8>) {
    let mut value = value as u32;
    while value >= 0x80 {
        output.push((value as u8 & 0x7f) | 0x80);
        value >>= 7;
    }
    output.push(value as u8);
}

/// Collects bytes from a stream and splits them into frames, the length-prefixed units every
/// packet travels in.
#[derive(Default)]
pub struct FrameReader {
    buffer: Vec<u8>,
    start: usize,
}

impl FrameReader {
    pub fn push(&mut self, bytes: &[u8]) {
        self.buffer.drain(..self.start);
        self.start = 0;
        self.buffer.extend_from_slice(bytes);
    }

    /// The next complete frame, without its length prefix.
    pub fn next_frame(&mut self) -> io::Result<Option<Vec<u8>>> {
        let pending = &self.buffer[self.start..];
        let Some((length, prefix)) = read_var_int(pending)? else {
            return Ok(None);
        };
        let length = usize::try_from(length)
            .map_err(|_| invalid(format!("negative frame length {length}")))?;
        if pending.len() < prefix + length {
            return Ok(None);
        }

        let frame = pending[prefix..prefix + length].to_vec();
        self.start += prefix + length;
        Ok(Some(frame))
    }
}

/// Splits a frame into packet id and payload, inflating it first when compression is on.
pub fn decode_packet(frame: &[u8], compressed: bool) -> io::Result<(i32, Vec<u8>)> {
    let data = if compressed {
        inflate(frame)?
    } else {
        frame.to_vec()
    };
    let (id, id_length) =
        read_var_int(&data)?.ok_or_else(|| invalid("frame ends inside the packet id"))?;
    Ok((id, data[id_length..].to_vec()))
}

// A compressed frame starts with the inflated length, or 0 when the packet was under the threshold.
fn inflate(frame: &[u8]) -> io::Result<Vec<u8>> {
    let (data_length, prefix) =
        read_var_int(frame)?.ok_or_else(|| invalid("frame ends inside the data length"))?;
    let body = &frame[prefix..];
    if data_length == 0 {
        return Ok(body.to_vec());
    }

    let data_length = usize::try_from(data_length)
        .map_err(|_| invalid(format!("negative data length {data_length}")))?;
    let mut data = Vec::with_capacity(data_length);
    ZlibDecoder::new(body).read_to_end(&mut data)?;
    if data.len() != data_length {
        return Err(invalid(format!(
            "inflated {} bytes, the frame announced {data_length}",
            data.len()
        )));
    }
    Ok(data)
}

/// A full frame, length prefix included. Like vanilla, packets at or above the compression
/// threshold are deflated.
pub fn encode_frame(id: i32, payload: &[u8], compression_threshold: Option<usize>) -> Vec<u8> {
    let mut data = Vec::with_capacity(payload.len() + MAX_VAR_INT_BYTES);
    write_var_int(id, &mut data);
    data.extend_from_slice(payload);

    let body = match compression_threshold {
        None => data,
        Some(threshold) if data.len() < threshold => {
            let mut body = vec![0];
            body.extend_from_slice(&data);
            body
        }
        Some(_) => {
            let mut body = Vec::new();
            write_var_int(data.len() as i32, &mut body);
            let mut encoder = ZlibEncoder::new(body, Compression::default());
            encoder
                .write_all(&data)
                .expect("writing to a Vec can't fail");
            encoder.finish().expect("writing to a Vec can't fail")
        }
    };

    let mut frame = Vec::with_capacity(body.len() + MAX_VAR_INT_BYTES);
    write_var_int(body.len() as i32, &mut frame);
    frame.extend_from_slice(&body);
    frame
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn var_ints_round_trip() {
        for value in [
            0,
            1,
            127,
            128,
            255,
            25565,
            2_097_151,
            i32::MAX,
            -1,
            i32::MIN,
        ] {
            let mut bytes = Vec::new();
            write_var_int(value, &mut bytes);
            assert_eq!(read_var_int(&bytes).unwrap(), Some((value, bytes.len())));
        }
    }

    #[test]
    fn var_ints_match_the_protocol_examples() {
        let mut bytes = Vec::new();
        write_var_int(-1, &mut bytes);
        assert_eq!(bytes, [0xff, 0xff, 0xff, 0xff, 0x0f]);
        assert_eq!(read_var_int(&[0xdd, 0xc7, 0x01]).unwrap(), Some((25565, 3)));
    }

    #[test]
    fn var_int_reports_incomplete_and_too_long() {
        assert_eq!(read_var_int(&[0x80, 0x80]).unwrap(), None);
        assert!(read_var_int(&[0x80; 6]).is_err());
    }

    #[test]
    fn frames_split_across_reads() {
        let first = encode_frame(1, b"hello", None);
        let second = encode_frame(2, &[7; 300], None);
        let stream: Vec<u8> = [first, second].concat();

        let mut reader = FrameReader::default();
        let mut frames = Vec::new();
        for chunk in stream.chunks(3) {
            reader.push(chunk);
            while let Some(frame) = reader.next_frame().unwrap() {
                frames.push(decode_packet(&frame, false).unwrap());
            }
        }
        assert_eq!(frames, [(1, b"hello".to_vec()), (2, vec![7; 300])]);
    }

    #[test]
    fn compressed_frames_round_trip_on_both_sides_of_the_threshold() {
        for payload in [vec![1; 10], vec![2; 1000]] {
            let frame = encode_frame(42, &payload, Some(256));
            let mut reader = FrameReader::default();
            reader.push(&frame);
            let body = reader.next_frame().unwrap().unwrap();
            assert_eq!(decode_packet(&body, true).unwrap(), (42, payload));
        }
    }
}
