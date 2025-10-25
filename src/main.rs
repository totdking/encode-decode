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

#[derive(Error, Debug, PartialEq)]
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

#[derive(Debug, Default, PartialEq)]
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
        let from_len_bytes: [u8; 4] = from_len.to_be_bytes();
        let from_bytes = from.as_bytes();

        let to_len = to.len() as u32;
        let to_len_bytes: [u8; 4] = to_len.to_be_bytes();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_success() {
        let payment = SimplePayment {
            from: "alice".to_string(),
            to: "bob".to_string(),
            amount: 12345,
        };
        let encoded = payment.encode().unwrap();
        let decoded = SimplePayment::decode(&encoded).unwrap();
        assert_eq!(payment, decoded);
    }
    
    #[test]
    fn test_decode_insufficient_data() {
        // Valid encoding of a payment
        let payment = SimplePayment { from: "a".to_string(), to: "b".to_string(), amount: 1 };
        let encoded = payment.encode().unwrap();
        
        // Truncate the buffer by 1 byte (removes part of the 'amount')
        let truncated_data = &encoded[..encoded.len() - 1];
        
        let err = SimplePayment::decode(truncated_data).unwrap_err();
        assert_eq!(err, BitError::InsufficientData);
    }
    
    #[test]
    fn test_decode_trailing_data() {
        let payment = SimplePayment { from: "a".to_string(), to: "b".to_string(), amount: 1 };
        let mut encoded = payment.encode().unwrap();
        
        // Add extra, unexpected bytes
        encoded.push(0xDE);
        encoded.push(0xAD);
        encoded.push(0xBE);
        encoded.push(0xEF);
        
        let err = SimplePayment::decode(&encoded).unwrap_err();
        assert_eq!(err, BitError::TrailingData);
    }
    
    #[test]
    fn test_decode_string_too_long() {
        // Manually craft a malicious packet
        let mut malicious_data = Vec::new();
        // 1. 'from' string: length = 2000 (which is > MAX_STRING_LEN)
        malicious_data.write_all(&2000u32.to_be_bytes()).unwrap();
        malicious_data.write_all(&vec![0u8; 2000]).unwrap();
        
        // 2. 'to' string: length = 1
        malicious_data.write_all(&1u32.to_be_bytes()).unwrap();
        malicious_data.write_all(b"a").unwrap();
        
        // 3. 'amount': 1
        malicious_data.write_all(&1u64.to_be_bytes()).unwrap();
        
        let err = SimplePayment::decode(&malicious_data).unwrap_err();
        assert_eq!(err, BitError::StringTooLong);
    }
}