// Convert a SimplePayment to a vec<u8> and from a Vec<u8> back to SimplePayment
use std::io::{Cursor, Read, Write};
use thiserror::Error;

fn main() {
    let some_payment = SimplePayment {
        from: "Bob".to_string(),
        to: "Alice".to_string(),
        amount: 1000,
    };
    let encoded_data = some_payment.encode().unwrap();
    let decoded_data = SimplePayment::decode(&encoded_data);
    println!("encoded value is {:?}", encoded_data);
    println!("decoded value is {:?}", decoded_data);
}
const MAX_LEN: u32 = 1024;

#[derive(Error, Debug)]
pub enum BitError {
    /// Wrapper for all std::io errors
    #[error("could not encode")]
    Io(String),
    /// Failed to parse UTF-8 string
    #[error("Failed to parse UTF-8 string")]
    Utf8(String),
    /// Data buffer was too short
    #[error("Data buffer was too short")]
    InsufficientData,
    /// Data buffer had extra, unexpected bytes
    #[error("Data buffer had extra, unexpected bytes")]
    TrailingData,
    /// A string length was declared that exceeds our safe limit
    #[error("A string length was declared that exceeds our safe limit")]
    StringTooLong,
}

#[derive(Debug, Default)]
pub struct SimplePayment {
    from: String,
    to: String,
    amount: u64,
}

impl SimplePayment {
    fn encode(&self) -> Result<Vec<u8>, BitError> {
        let mut from_buffer = Vec::new();
        let mut to_buffer = Vec::new();
        let mut amount_buffer = Vec::new();

        let mut self_buffer = Vec::new();

        let from = &self.from;
        let to = &self.to;
        let amount = self.amount;

        let from_len = from.len() as u32;
        let from_len_bytes = from_len.to_be_bytes();
        let from_bytes = from.as_bytes();

        let to_len = to.len() as u32;
        let to_len_bytes = to_len.to_be_bytes();
        let to_bytes = to.as_bytes();

        let amount_bytes = amount.to_be_bytes();

        from_buffer
            .write_all(&from_len_bytes)
            .map_err(|e| BitError::Io(e.to_string()))?;
        from_buffer
            .write_all(from_bytes)
            .map_err(|e| BitError::Io(e.to_string()))?;

        to_buffer
            .write_all(&to_len_bytes)
            .map_err(|e| BitError::Io(e.to_string()))?;
        to_buffer
            .write_all(to_bytes)
            .map_err(|e| BitError::Io(e.to_string()))?;

        amount_buffer
            .write_all(&amount_bytes)
            .map_err(|e| BitError::Io(e.to_string()))?;

        self_buffer
            .write_all(&from_buffer)
            .map_err(|e| BitError::Io(e.to_string()))?;
        self_buffer
            .write_all(&to_buffer)
            .map_err(|e| BitError::Io(e.to_string()))?;
        self_buffer
            .write_all(&amount_buffer)
            .map_err(|e| BitError::Io(e.to_string()))?;

        Ok(self_buffer)
    }

    fn decode(bytes: &[u8]) -> Result<Self, BitError> {
        let encoded_data = bytes;

        let mut cursor = Cursor::new(encoded_data);

        let from = read_string_from_cursor(&mut cursor)?;

        let to = read_string_from_cursor(&mut cursor)?;

        let mut amount_bytes = [0u8; 8];

        cursor
            .read_exact(&mut amount_bytes)
            .map_err(|_| BitError::InsufficientData)?;

        let amount = u64::from_be_bytes(amount_bytes);

        if cursor.position() < encoded_data.len() as u64 {
            return Err(BitError::TrailingData);
        }
        Ok(Self { from, to, amount })
    }
}

fn read_string_from_cursor(cursor: &mut Cursor<&[u8]>) -> Result<String, BitError> {
    let mut len_bytes = [0u8; 4];

    cursor
        .read_exact(&mut len_bytes)
        .map_err(|_| BitError::InsufficientData)?;

    let len = u32::from_be_bytes(len_bytes);

    if len > MAX_LEN {
        return Err(BitError::StringTooLong);
    }

    let mut string_buf = vec![0u8; len as usize];

    cursor
        .read_exact(&mut string_buf)
        .map_err(|_| BitError::InsufficientData)?;

    String::from_utf8(string_buf).map_err(|e| BitError::Utf8(e.to_string()))
}
