use thiserror::Error;

#[derive(Error, Debug)]
pub enum RleCompressionError {
    #[error("Invalid input length: data is empty")]
    EmptyInput,
}

#[derive(Error, Debug)]
pub enum RleDecompressionError {
    #[error("Invalid input length: data is empty")]
    EmptyInput,
    #[error("Invalid input length: expected multiple of 2 bytes, got {0}")]
    InvalidInputLength(usize),
}

/// Combines RLE and delta encoding to compress byte data.
///
/// # Arguments
/// * `data` - Slice of bytes to compress
///
/// # Returns
/// * `Result<Vec<u8>, RleCompressionError>` - Compressed data or error
pub fn rle_delta_compression(data: &[u8]) -> Result<Vec<u8>, RleCompressionError> {
    if data.is_empty() {
        return Err(RleCompressionError::EmptyInput);
    }

    let mut encoded = Vec::with_capacity(data.len() / 2);
    let initial_value = data[0];
    encoded.push(initial_value); // Start with the initial value

    let mut count = 1u16; // Use u16 to support larger counts
    let mut prev_delta = data[1].wrapping_sub(initial_value);

    for i in 1..data.len() - 1 {
        let current_delta = data[i + 1].wrapping_sub(data[i]);

        if current_delta == prev_delta && count < u16::MAX {
            count += 1;
        } else {
            if prev_delta == 0 {
                // Handle large blocks of the same value more efficiently
                while count > 255 {
                    encoded.push(255);
                    encoded.push(0); // delta 0 for identical values
                    count -= 255;
                }
                encoded.push(count as u8);
                encoded.push(0); // delta 0 for identical values
            } else {
                // Encode the count and delta normally
                encoded.push(count as u8);
                encoded.push(prev_delta);
            }
            count = 1;
            prev_delta = current_delta;
        }
    }

    // Final block
    if prev_delta == 0 {
        while count > 255 {
            encoded.push(255);
            encoded.push(0);
            count -= 255;
        }
        encoded.push(count as u8);
        encoded.push(0);
    } else {
        encoded.push(count as u8);
        encoded.push(prev_delta);
    }

    Ok(encoded)
}

/// Decompresses data that was compressed using `rle_delta_compression`.
///
/// # Arguments
/// * `data` - Compressed data slice
///
/// # Returns
/// * `Result<Vec<u8>, RleCompressionError>` - Decompressed data or error
pub fn rle_delta_decompression(data: &[u8]) -> Result<Vec<u8>, RleDecompressionError> {
    if data.is_empty() {
        return Err(RleDecompressionError::EmptyInput);
    }
    if data.len() < 3 || data.len() % 2 != 1 {
        return Err(RleDecompressionError::InvalidInputLength(data.len()));
    }

    let initial_value = data[0];
    let mut decoded = Vec::with_capacity(data.len() * 2);
    decoded.push(initial_value);

    let mut current_value = initial_value;
    let mut i = 1;

    while i < data.len() {
        let count = data[i];
        let delta = data[i + 1];
        decoded.reserve(count as usize);

        for _ in 0..count {
            current_value = current_value.wrapping_add(delta);
            decoded.push(current_value);
        }

        i += 2;
    }

    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rle_empty_input() {
        assert!(matches!(
            rle_delta_compression(&[]),
            Err(RleCompressionError::EmptyInput)
        ));
    }

    #[test]
    fn test_rle_sequential_numbers() {
        let input = vec![1, 2, 3, 4, 5];
        let compressed = rle_delta_compression(&input).unwrap();
        let decompressed = rle_delta_decompression(&compressed).unwrap();
        assert_eq!(input, decompressed);
    }

    #[test]
    fn test_rle_repeated_values() {
        let input = vec![10, 10, 10, 10, 10, 10];
        let compressed = rle_delta_compression(&input).unwrap();
        let decompressed = rle_delta_decompression(&compressed).unwrap();
        assert_eq!(input, decompressed);
    }

    #[test]
    fn test_rle_wrapping_behavior() {
        let input = vec![255, 0, 1];
        let compressed = rle_delta_compression(&input).unwrap();
        let decompressed = rle_delta_decompression(&compressed).unwrap();
        assert_eq!(input, decompressed);
    }

    #[test]
    fn test_rle_alternating_pattern() {
        let input = vec![0, 1, 0, 1, 0, 1];
        let compressed = rle_delta_compression(&input).unwrap();
        let decompressed = rle_delta_decompression(&compressed).unwrap();
        assert_eq!(input, decompressed);
    }

    #[test]
    fn test_rle_invalid_compressed_data() {
        assert!(matches!(
            rle_delta_decompression(&[1, 2]),
            Err(RleDecompressionError::InvalidInputLength(2))
        ));
    }

    #[test]
    fn test_rle_radnom_compressed_data() {
        let input = vec![1, 2, 3, 3, 3, 3, 3, 3, 4, 5, 6, 7, 8, 9, 9, 9, 9, 99, 10];
        let compressed = rle_delta_compression(&input).unwrap();
        let decompressed = rle_delta_decompression(&compressed).unwrap();
        assert_eq!(input, decompressed);
    }

    #[test]
    fn test_rle_gradients() {
        let mut data = Vec::new();
        for i in 0..256 {
            data.extend_from_slice(&[i as u8, i as u8, i as u8, 255]);
        }

        let compressed = rle_delta_compression(&data).unwrap();
        let decompressed = rle_delta_decompression(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }
}
