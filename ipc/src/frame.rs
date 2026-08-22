use std::io::{self, Read, Write};
use serde::{Serialize, de::DeserializeOwned};

const MAX_FRAME: usize = 1024 * 1024;

pub fn write_frame<W: Write, T: Serialize>(w: &mut W, msg: &T) -> io::Result<()> {
    let data = serde_json::to_vec(msg).map_err(io::Error::other)?;
    if data.len() > MAX_FRAME {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "frame too large"));
    }

    w.write_all(&(data.len() as u32).to_le_bytes())?;
    w.write_all(&data)?;

    w.flush()
}

pub fn read_frame<R: Read, T: DeserializeOwned>(r: &mut R) -> io::Result<T> {
    let mut len_buf = [0u8; 4];
    let mut got = 0;

    while got < 4 {
        match r.read(&mut len_buf[got..])? {
            // Pure connection close is different from a truncated header
            0 if got == 0 => {
                return Err(io::Error::new(io::ErrorKind::ConnectionReset, "peer closed"));
            }
            0 => return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "truncated header")),
            n => got += n,
        }
    }

    let len = u32::from_le_bytes(len_buf) as usize;
    if len > MAX_FRAME {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "frame too large"));
    }

    let mut data = vec![0u8; len];
    r.read_exact(&mut data)?;
    serde_json::from_slice(&data).map_err(io::Error::other)
}
