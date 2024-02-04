use std::sync::OnceLock;

use crate::{chunk_type::ChunkType, Error};

static CRC32_TABLE: OnceLock<[u32; 256]> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct Chunk {
    length: u32,
    r#type: ChunkType,
    data: Box<[u8]>,
    crc: u32,
}

impl TryFrom<&[u8]> for Chunk {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let length = u32::from_be_bytes(value.get(..4).ok_or("Invalid chunk length")?.try_into()?);
        if length > 2_u32.pow(31) || length != value.len() as u32 - 12 {
            return Err(Error::from("Invalid chunk length"));
        }
        let type_bytes: [u8; 4] = value
            .get(4..8)
            .ok_or("Invalid chunk length: chunk type")?
            .try_into()?;
        let r#type = ChunkType::try_from(type_bytes)?;
        let crc_offset = value.len() - 4;
        let data = value
            .get(8..crc_offset)
            .ok_or("Invalid chunk length: chunk data")?
            .into();
        let crc = u32::from_be_bytes(
            value
                .get(crc_offset..)
                .ok_or("Invalid chunk length: chunk CRC")?
                .try_into()?,
        );
        let calculated_crc = crc32(type_bytes, &data);

        if crc != calculated_crc {
            Err(format!("Invalid chunk CRC: read: {crc}, calculated: {calculated_crc}").into())
        } else {
            Ok(Self {
                length,
                r#type,
                data,
                crc,
            })
        }
    }
}

impl std::fmt::Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:#?}"))
    }
}

impl Chunk {
    pub fn new(r#type: ChunkType, data: impl AsRef<[u8]>) -> Self {
        let data: Box<[u8]> = data.as_ref().into();
        let length = data.len() as u32;
        let crc = crc32(r#type.as_bytes(), &data);

        Self {
            length,
            r#type,
            data,
            crc,
        }
    }
    pub const fn length(&self) -> u32 {
        self.length
    }
    pub const fn r#type(&self) -> &ChunkType {
        &self.r#type
    }
    pub const fn data(&self) -> &[u8] {
        &self.data
    }
    pub const fn crc(&self) -> u32 {
        self.crc
    }
    pub fn data_as_string(&self) -> Result<String, Error> {
        Ok(std::str::from_utf8(&self.data)?.to_owned())
    }
    pub fn bytes(&self) -> Vec<u8> {
        self.length
            .to_be_bytes()
            .into_iter()
            .chain(*self.r#type.as_bytes())
            .chain(self.data.iter().copied())
            .chain(self.crc.to_be_bytes())
            .collect()
    }
}

fn crc32(r#type: impl AsRef<[u8]>, data: impl AsRef<[u8]>) -> u32 {
    let table = CRC32_TABLE.get_or_init(|| {
        std::array::from_fn(|i| {
            (0..8).fold(i as u32, |c, _| match c & 1 {
                1 => c >> 1 ^ 0xedb88320,
                _ => c >> 1,
            })
        })
    });

    !r#type
        .as_ref()
        .iter()
        .copied()
        .chain(data.as_ref().iter().copied())
        .fold(u32::MAX, |c, octet| {
            c >> 8 ^ table[((c ^ octet as u32) & 0xff) as usize]
        })
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = b"RuSt";
        let message_bytes = b"This is where your secret message will be!";
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_crc32() {
        let chunk_type = b"The ";
        let data = b"quick brown fox jumps over the lazy dog".to_vec();
        assert_eq!(crc32(chunk_type, data), 0x414fa339);
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = b"This is where your secret message will be!".to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.r#type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 0xabd1d84e);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = b"RuSt";
        let message_bytes = b"This is where your secret message will be!";
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.r#type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 0xabd1d84e);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = b"RuSt";
        let message_bytes = b"This is where your secret message will be!";
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = b"RuSt";
        let message_bytes = b"This is where your secret message will be!";
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
