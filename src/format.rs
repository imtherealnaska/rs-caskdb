pub const DEFAULT_WHENCE: usize = 0;
pub(crate) const HEADER_SIZE: usize = 12;

pub struct KeyEntry {
    pub(crate) timestamp: u32,
    pub(crate) pos: u64,
    pub(crate) total_size: u32,
}
impl KeyEntry {
    /// Creates a new KeyEntry.
    pub fn new(timestamp: u32, pos: u64, total_size: u32) -> Self {
        Self {
            timestamp,
            pos,
            total_size,
        }
    }
}

pub(crate) fn decode_header(header: &[u8]) -> (u32, u32, u32) {
    let timestamp = u32::from_le_bytes(header[0..4].try_into().unwrap());
    let key_size = u32::from_le_bytes(header[4..8].try_into().unwrap());
    let value_size = u32::from_le_bytes(header[8..12].try_into().unwrap());
    (timestamp, key_size, value_size)
}

pub fn encode_header(timestamp: u32, key_size: u32, value_size: u32) -> [u8; 12] {
    let mut header = [0u8; 12];
    header[0..4].copy_from_slice(&timestamp.to_le_bytes());
    header[4..8].copy_from_slice(&key_size.to_le_bytes());
    header[8..12].copy_from_slice(&value_size.to_le_bytes());
    header
}

pub fn encode_kv(timestamp: u32, key: String, value: String) -> (usize, Vec<u8>) {
    let key_bytes = key.as_bytes();
    let value_bytes = value.as_bytes();
    let header = encode_header(timestamp, key_bytes.len() as u32, value_bytes.len() as u32);
    let mut data = Vec::with_capacity(header.len() + key_bytes.len() + value_bytes.len());
    data.extend_from_slice(&header);
    data.extend_from_slice(key_bytes);
    data.extend_from_slice(value_bytes);
    (data.len(), data)
}

pub fn decode_kv(data: &[u8]) -> (u32, String, String) {
    let (timestamp, key_size, value_size) = decode_header(&data[0..HEADER_SIZE]);
    let key_start = HEADER_SIZE;
    let key_end = key_start + key_size as usize;
    let value_start = key_end;
    let value_end = value_start + value_size as usize;

    let key = String::from_utf8(data[key_start..key_end].to_vec()).expect("Invalid UTF-8 in key");
    let value =
        String::from_utf8(data[value_start..value_end].to_vec()).expect("Invalid UTF-8 in value");

    (timestamp, key, value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_header() {
        let tests = vec![(10, 10, 10), (0, 0, 0), (10000, 10000, 10000)];

        for (timestamp, key_size, value_size) in tests {
            let data = encode_header(timestamp, key_size, value_size);
            let (decoded_timestamp, decoded_key_size, decoded_value_size) = decode_header(&data);

            assert_eq!(decoded_timestamp, timestamp, "timestamp mismatch");
            assert_eq!(decoded_key_size, key_size, "key_size mismatch");
            assert_eq!(decoded_value_size, value_size, "value_size mismatch");
        }
    }

    #[test]
    fn test_encode_kv() {
        let tests = vec![
            (10, "hello", "world", HEADER_SIZE + 10),
            (0, "", "", HEADER_SIZE),
            (100, "🔑", "", HEADER_SIZE + 4),
        ];

        for (timestamp, key, value, expected_size) in tests {
            let (size, data) = encode_kv(timestamp, key.into(), value.into());
            let (decoded_timestamp, decoded_key, decoded_value) = decode_kv(&data);

            assert_eq!(decoded_timestamp, timestamp, "timestamp mismatch");
            assert_eq!(decoded_key, key, "key mismatch");
            assert_eq!(decoded_value, value, "value mismatch");
            assert_eq!(size, expected_size, "size mismatch");
        }
    }
}
