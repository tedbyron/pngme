use crate::Error;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ChunkType([u8; 4]);

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = Error;

    fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
        if !value.iter().all(u8::is_ascii_alphabetic) {
            Err("Invalid chunk type: contains non-ASCII chars".into())
        } else {
            Ok(Self(value))
        }
    }
}

impl std::str::FromStr for ChunkType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {
            Err("Invalid chunk type length".into())
        } else {
            let s: [u8; 4] = s.as_bytes()[..4].try_into()?;
            Ok(Self::try_from(s)?)
        }
    }
}

impl std::fmt::Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(std::str::from_utf8(self.as_bytes()).map_err(|_| std::fmt::Error)?)
    }
}

impl ChunkType {
    pub const fn as_bytes(&self) -> &[u8; 4] {
        &self.0
    }
    pub const fn is_valid(&self) -> bool {
        self.is_reserved_bit_valid()
    }
    pub const fn is_critical(&self) -> bool {
        self.0[0].is_ascii_uppercase()
    }
    pub const fn is_public(&self) -> bool {
        self.0[1].is_ascii_uppercase()
    }
    pub const fn is_reserved_bit_valid(&self) -> bool {
        self.0[2].is_ascii_uppercase()
    }
    pub const fn is_safe_to_copy(&self) -> bool {
        self.0[3].is_ascii_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = &[82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.as_bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
