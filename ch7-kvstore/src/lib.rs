use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crc;
// use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Seek, Write};
use std::{fs, io, path};

type ByteStr = [u8];
type ByteString = Vec<u8>;

const CRC32: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_CKSUM);

pub struct Store {
    f: fs::File,
    pub index: HashMap<ByteString, u64>, // Maps key to file-position
}

fn pack_data(data: &[&ByteStr]) -> ByteString {
    let total = data.iter().map(|x| x.len()).sum();
    let mut res: ByteString = vec![0; total];

    let mut counter = 0;
    for d in data.iter() {
        res[counter..counter + d.len()].copy_from_slice(d);
        counter += d.len();
    }

    res
}

impl Store {
    pub fn new(p: &path::Path) -> io::Result<Store> {
        let f = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .append(false)
            .truncate(false)
            .create_new(true)
            .open(p)?;
        let index = HashMap::new();
        Ok(Store { f, index })
    }

    pub fn open(p: &path::Path) -> io::Result<Store> {
        let f = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .append(false)
            .create(true)
            .open(p)?;
        let index = HashMap::new();
        Ok(Store { f, index })
    }

    pub fn insert(&mut self, key: &ByteStr, val: &ByteStr) -> io::Result<()> {
        let data = pack_data(&[key, val]);
        let checksum = CRC32.checksum(&data);

        let mut buf = io::BufWriter::new(&mut self.f);
        let insert_pos = buf.seek(io::SeekFrom::End(0))?;

        buf.write_u32::<LittleEndian>(checksum)?;
        buf.write_u32::<LittleEndian>(key.len() as u32)?;
        buf.write_u32::<LittleEndian>(val.len() as u32)?;
        buf.write_all(&data)?;

        self.index.insert(key.to_vec(), insert_pos);
        Ok(())
    }

    pub fn get(&mut self, key: &ByteStr) -> io::Result<Option<ByteString>> {
        let position = match self.index.get(key) {
            None => return Ok(None),
            Some(p) => *p,
        };

        let mut buf = io::BufReader::new(&mut self.f);
        buf.seek(io::SeekFrom::Start(position))?;

        let checksum_stored = buf.read_u32::<LittleEndian>()?;
        let key_len = buf.read_u32::<LittleEndian>()?;
        let val_len = buf.read_u32::<LittleEndian>()?;

        let mut data: ByteString = vec![0; (key_len + val_len) as usize];
        buf.read_exact(data.as_mut_slice())?;

        let checksum = CRC32.checksum(&data);
        if checksum != checksum_stored {
            return Err(io::Error::new(io::ErrorKind::Other, "Checksum mismatch"));
        }

        let val = data.split_off(key_len as usize);

        Ok(Some(val))
    }

    #[inline]
    pub fn update(&mut self, key: &ByteStr, val: &ByteStr) -> io::Result<()> {
        self.insert(key, val)
    }

    #[inline]
    pub fn delete(&mut self, key: &ByteStr) -> io::Result<()> {
        self.insert(key, b"")
    }
}

#[cfg(test)]
mod tests {
    use uuid;

    use super::*;

    fn random_path() -> path::PathBuf {
        let mut p = tempfile::tempdir().unwrap().into_path();
        p.push(uuid::Uuid::new_v4().to_string());
        p
    }

    #[test]
    fn test_create_read_single() {
        let mut store = Store::new(&random_path()).unwrap();

        store.insert(b"abc", b"123").expect("insert");

        assert_eq!(store.get(b"abc").expect("get"), Some(b"123".to_vec()));
    }

    #[test]
    fn test_read_none_existing() {
        let mut store = Store::new(&random_path()).unwrap();

        assert_eq!(store.get(b"123").unwrap(), None);
        store.insert(b"abc", b"123").unwrap();
        assert_eq!(store.get(b"123").unwrap(), None);
    }

    #[test]
    fn test_create_read_multiple() {
        let mut store = Store::new(&random_path()).unwrap();

        store.insert(b"abc", b"123").unwrap();
        store.insert(b"def", b"456").unwrap();
        store.insert(b"ghi", b"789").unwrap();

        assert_eq!(store.get(b"abc").unwrap(), Some(b"123".to_vec()));
        assert_eq!(store.get(b"def").unwrap(), Some(b"456".to_vec()));
        assert_eq!(store.get(b"ghi").unwrap(), Some(b"789".to_vec()));

        store.insert(b"jkl", b"111").unwrap();

        assert_eq!(store.get(b"abc").unwrap(), Some(b"123".to_vec()));
        assert_eq!(store.get(b"def").unwrap(), Some(b"456".to_vec()));
        assert_eq!(store.get(b"ghi").unwrap(), Some(b"789".to_vec()));
        assert_eq!(store.get(b"jkl").unwrap(), Some(b"111".to_vec()));
    }

    #[test]
    fn test_create_read_overwrite() {
        let mut store = Store::new(&random_path()).unwrap();

        store.insert(b"abc", b"123").unwrap();
        store.insert(b"def", b"456").unwrap();

        assert_eq!(store.get(b"abc").unwrap(), Some(b"123".to_vec()));
        assert_eq!(store.get(b"def").unwrap(), Some(b"456".to_vec()));

        store.insert(b"abc", b"789").unwrap();

        assert_eq!(store.get(b"abc").unwrap(), Some(b"789".to_vec()));
        assert_eq!(store.get(b"def").unwrap(), Some(b"456".to_vec()));
    }

    #[test]
    fn test_new_complains_about_existing_file() {
        let p = random_path();

        {
            Store::new(&p).unwrap();
        }

        match Store::new(&p) {
            Ok(_) => panic!("Store::new should fail with existing file"),
            Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => (),
            _ => panic!("Wrong error reported"),
        }
    }
}
