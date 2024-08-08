use chrono::Utc;

use crate::format::{decode_header, decode_kv, encode_kv, KeyEntry, HEADER_SIZE};
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
};

pub struct DiskStore {
    file: File,
    write_pos: u64,
    key_dir: HashMap<String, KeyEntry>,
}

fn is_file_exists(filename: String) -> bool {
    match fs::metadata(filename) {
        Ok(met) => met.is_file(),
        Err(_) => false,
    }
}

impl DiskStore {
    pub fn new(self, file_name: &str) -> io::Result<DiskStore> {
        let mut ds = DiskStore {
            file: File::create(file_name)?,
            write_pos: 0,
            key_dir: HashMap::new(),
        };

        if is_file_exists(file_name.to_owned()) {
            ds.init_key_dir(file_name)?;
        }

        let file = OpenOptions::new()
            .append(true)
            .read(true)
            .write(true)
            .create(true)
            .open(file_name)?;

        ds.file = file;
        ds.write_pos = ds.file.seek(SeekFrom::End(0))?;
        Ok(ds)
    }

    pub fn init_key_dir(&mut self, file_name: &str) -> io::Result<()> {
        self.file.seek(SeekFrom::Start(0))?;

        loop {
            let mut header = vec![0u8; HEADER_SIZE];
            if self.file.read_exact(&mut header).is_err() {
                break;
            }
            let (timestamp, key_size, value_size) = decode_header(&header);
            let mut key = vec![0u8; key_size as usize];
            let mut value = vec![0u8; value_size as usize];

            if self.file.read_exact(&mut key).is_err() || self.file.read_exact(&mut value).is_err()
            {
                break;
            }

            let key_string = String::from_utf8(key).unwrap_or_default();
            let total_size = HEADER_SIZE + key_size as usize + value_size as usize;
            self.key_dir.insert(
                key_string.clone(),
                KeyEntry {
                    timestamp,
                    pos: self.write_pos,
                    total_size: total_size as u32,
                },
            );

            self.write_pos += total_size as u64;
            //Finally used this String::from_utf8_lossy
            println!(
                "Loaded key={}, value={}",
                key_string,
                String::from_utf8_lossy(&value)
            );
        }
        Ok(())
    }

    pub fn write(&mut self, data: &[u8]) -> io::Result<()> {
        self.file.write_all(data)?;
        self.file.sync_all()?;
        Ok(())
    }

    pub fn close(&mut self) -> io::Result<()> {
        self.file.sync_all()?;
        self.file.flush()?;
        //TODO: drop(self.file);
        Ok(())
    }

    pub fn set(mut self, key: String, value: String) -> io::Result<()> {
        let now = Utc::now();
        let ts = now.timestamp().try_into().unwrap();
        let (size, data) = encode_kv(ts, key.clone(), value);
        self.write(&data)?;
        self.key_dir
            .insert(key, KeyEntry::new(ts, self.write_pos, size as u32));
        Ok(())
    }

    pub fn get(&mut self, key: &str) -> String {
        let k_entry = match self.key_dir.get(key) {
            Some(entry) => entry,
            None => return String::new(),
        };

        if let Err(e) = self.file.seek(SeekFrom::Start(k_entry.pos)) {
            eprintln!("seek error : {:?}", e);
            return String::new();
        }

        let mut data = vec![0; k_entry.total_size as usize];

        if let Err(e) = self.file.read_exact(&mut data) {
            eprintln!("read error {:?}", e);
            return String::new();
        }

        let (_, _, value) = decode_kv(&data);
        value
    }
}
