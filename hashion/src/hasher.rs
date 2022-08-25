use std::io::{BufRead, Result};
use std::path::Path;
use std::{fs, io};

type MD5Checksum = [u8; 16];

pub struct Hasher {
    block_size: usize,
}

impl Hasher {
    pub fn new() -> Hasher {
        Hasher {
            block_size: 1 << 30,
        }
    }

    pub fn hash_file(&self, path: &Path) -> Result<MD5Checksum> {
        let f = fs::OpenOptions::new().read(true).open(&path)?;
        let mut reader = io::BufReader::with_capacity(self.block_size, f);
        let mut context = md5::Context::new();

        loop {
            let data = reader.fill_buf()?;
            let bytes_read = data.len();
            context.consume(&data);
            if bytes_read < reader.capacity() {
                break;
            }
            reader.consume(bytes_read);
        }

        let md5::Digest(checksum) = context.compute();
        Ok(checksum)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::{fmt::Write, num::ParseIntError};

    fn decode_hex(s: &str) -> std::result::Result<Vec<u8>, ParseIntError> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
            .collect()
    }

    #[test]
    fn test_hash_file() {
        let output = Command::new("md5")
            .arg("-r")
            .arg("Cargo.toml")
            .output()
            .expect("Failure running md5 command");
        if !output.status.success() {
            panic!("md5sum did not finish correctly");
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let tgt_hex =
            std::str::from_utf8(stdout.split_whitespace().next().unwrap().as_bytes()).unwrap();
        let tgt = decode_hex(&tgt_hex).unwrap();

        let hasher = Hasher::new();
        let checksum = hasher.hash_file(Path::new("Cargo.toml")).unwrap();

        assert_eq!(tgt, checksum);
    }
}
