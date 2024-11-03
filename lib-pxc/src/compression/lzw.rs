use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug)]
struct LZWEntry {
    prefix: Option<u16>,
    suffix: u8,
}

#[derive(Error, Debug)]
pub enum LzwCompressionError {
    #[error("dictionary overflow: reached maximum code value of 65535")]
    DictionaryOverflow,
    #[error("input data is too large to process")]
    InputTooLarge,
}

#[derive(Error, Debug)]
pub enum LzwDecompressionError {
    #[error("invalid input data: incomplete code at position {position}")]
    IncompleteCode { position: usize },
    #[error("invalid code: code value {code} exceeds dictionary size {dict_size}")]
    InvalidCode { code: usize, dict_size: usize },
    #[error("dictionary overflow: reached maximum code value of 65535")]
    DictionaryOverflow,
}

pub fn lzw_compression(data: &[u8]) -> Result<Vec<u8>, LzwCompressionError> {
    let mut dictionary = HashMap::new();
    let mut result = Vec::new();
    let mut next_code = 256u16; // Start after single byte values

    // Initialize dictionary with single bytes
    for i in 0..256 {
        dictionary.insert(vec![i as u8], i as u16);
    }

    if data.is_empty() {
        return Ok(result);
    }

    let mut current = vec![data[0]];

    for &byte in &data[1..] {
        let mut next = current.clone();
        next.push(byte);

        if dictionary.contains_key(&next) {
            current = next;
        } else {
            // Output code for current sequence
            if let Some(&code) = dictionary.get(&current) {
                result.extend_from_slice(&code.to_le_bytes());
            }

            // Add new sequence to dictionary if we haven't hit the limit
            if next_code < 65535 {
                dictionary.insert(next, next_code);
                next_code += 1;
            } else {
                // We can either return an error here or continue without adding new entries
                // Here we choose to return an error to inform the caller
                return Err(LzwCompressionError::DictionaryOverflow);
            }

            current = vec![byte];
        }
    }

    // Output code for final sequence
    if let Some(&code) = dictionary.get(&current) {
        result.extend_from_slice(&code.to_le_bytes());
    }

    Ok(result)
}

pub fn lzw_decompression(data: &[u8]) -> Result<Vec<u8>, LzwDecompressionError> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    let mut dictionary = Vec::with_capacity(65535);
    let mut result = Vec::new();

    // Initialize dictionary with single bytes
    for i in 0..256 {
        dictionary.push(LZWEntry {
            prefix: None,
            suffix: i as u8,
        });
    }

    // Process first code
    if data.len() < 2 {
        return Err(LzwDecompressionError::IncompleteCode { position: 0 });
    }

    let mut previous_code = u16::from_le_bytes([data[0], data[1]]) as usize;
    if previous_code >= dictionary.len() {
        return Err(LzwDecompressionError::InvalidCode {
            code: previous_code,
            dict_size: dictionary.len(),
        });
    }

    let mut previous_string = get_string(&dictionary, previous_code);
    result.extend(&previous_string);

    let mut next_code = 256u16;

    // Process remaining codes
    for (chunk_index, chunk) in data[2..].chunks(2).enumerate() {
        if chunk.len() < 2 {
            return Err(LzwDecompressionError::IncompleteCode {
                position: 2 + chunk_index * 2,
            });
        }

        let current_code = u16::from_le_bytes([chunk[0], chunk[1]]) as usize;

        // Get the current string
        let current_string = if current_code < dictionary.len() {
            get_string(&dictionary, current_code)
        } else if current_code == dictionary.len() && next_code < 65535 {
            // Special case: current code is next code to be added
            let mut s = previous_string.clone();
            s.push(previous_string[0]);
            s
        } else {
            return Err(LzwDecompressionError::InvalidCode {
                code: current_code,
                dict_size: dictionary.len(),
            });
        };

        result.extend(&current_string);

        // Add new code to dictionary if we haven't hit the limit
        if next_code < 65535 {
            dictionary.push(LZWEntry {
                prefix: Some(previous_code as u16),
                suffix: current_string[0],
            });
            next_code += 1;
        } else {
            return Err(LzwDecompressionError::DictionaryOverflow);
        }

        previous_code = current_code;
        previous_string = current_string;
    }

    Ok(result)
}

fn get_string(dictionary: &[LZWEntry], mut code: usize) -> Vec<u8> {
    let mut result = Vec::new();

    while let Some(entry) = dictionary.get(code) {
        result.push(entry.suffix);
        if let Some(prefix) = entry.prefix {
            code = prefix as usize;
        } else {
            break;
        }
    }

    result.reverse();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lzw_comp_empty_input() {
        let result = lzw_compression(&[]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_lzw_decomp_empty_input() {
        let result = lzw_decompression(&[]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_lzw_decomp_incomplete_code() {
        let result = lzw_decompression(&[0]);
        assert!(matches!(
            result,
            Err(LzwDecompressionError::IncompleteCode { position: 0 })
        ));
    }

    #[test]
    fn test_lzw_decomp_invalid_code() {
        let result = lzw_decompression(&[0xFF, 0xFF]);
        assert!(matches!(
            result,
            Err(LzwDecompressionError::InvalidCode { code: 65535, .. })
        ));
    }

    #[test]
    fn test_lzw_string() {
        let original = b"Hello, World!".to_vec();
        let compressed = lzw_compression(&original).unwrap();
        let decompressed = lzw_decompression(&compressed).unwrap();
        assert_eq!(original, decompressed);
    }

    #[test]
    fn test_lzw_empty() {
        let data = vec![];
        let compressed = lzw_compression(&data).unwrap();
        let decompressed = lzw_decompression(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_lzw_single_byte() {
        let data = vec![42];
        let compressed = lzw_compression(&data).unwrap();
        let decompressed = lzw_decompression(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_lzw_repeated_sequence() {
        let data = vec![1, 2, 3, 1, 2, 3, 1, 2, 3];
        let compressed = lzw_compression(&data).unwrap();
        let decompressed = lzw_decompression(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_lzw_no_repetition() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        let compressed = lzw_compression(&data).unwrap();
        let decompressed = lzw_decompression(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_lzw_simple_pattern() {
        // Test with a simple repeating pattern
        let data = vec![1, 1, 1, 1, 1];
        let compressed = lzw_compression(&data).unwrap();
        let decompressed = lzw_decompression(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_lzw_alternating_pattern() {
        // Test with an alternating pattern
        let data = vec![1, 2, 1, 2, 1, 2];
        let compressed = lzw_compression(&data).unwrap();
        let decompressed = lzw_decompression(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_lzw_long_sequence() {
        // Test with a longer sequence
        let data: Vec<u8> = (0..=255).collect();
        let compressed = lzw_compression(&data).unwrap();
        let decompressed = lzw_decompression(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_lzw_large_data() {
        // Test with a larger amount of data
        let data = vec![1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5];
        let compressed = lzw_compression(&data).unwrap();
        let decompressed = lzw_decompression(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_lzw_gradients() {
        let mut data = Vec::new();
        for i in 0..256 {
            data.extend_from_slice(&[i as u8, i as u8, i as u8, 255]);
        }

        let compressed = lzw_compression(&data).unwrap();
        let decompressed = lzw_decompression(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }
}
