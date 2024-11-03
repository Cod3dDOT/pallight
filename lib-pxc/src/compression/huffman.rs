use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HuffmanError {
    #[error("Input data is empty")]
    EmptyInput,
    #[error("Failed to create Huffman tree")]
    TreeCreationFailed,
}

// Node structure for Huffman tree
#[derive(Debug)]
struct HuffmanNode {
    frequency: usize,
    value: Option<u8>,
    left: Option<Box<HuffmanNode>>,
    right: Option<Box<HuffmanNode>>,
}

impl PartialEq for HuffmanNode {
    fn eq(&self, other: &Self) -> bool {
        self.frequency == other.frequency
    }
}

impl Eq for HuffmanNode {}

impl PartialOrd for HuffmanNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.frequency.cmp(&other.frequency))
    }
}

impl Ord for HuffmanNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.frequency.cmp(&other.frequency)
    }
}

pub struct HuffmanCode {
    pub encoding_map: HashMap<u8, Vec<bool>>,
    pub encoded_data: Vec<u8>,
    pub padding_bits: u8,
}
pub fn huffman_encode(data: &[u8]) -> Result<HuffmanCode, HuffmanError> {
    if data.is_empty() {
        return Err(HuffmanError::EmptyInput);
    }

    // Count frequencies
    let mut frequencies: HashMap<u8, usize> = HashMap::new();
    for &byte in data {
        *frequencies.entry(byte).or_insert(0) += 1;
    }

    // Special case: if there's only one unique symbol
    if frequencies.len() == 1 {
        let (&symbol, _) = frequencies.iter().next().unwrap();
        let mut encoding_map = HashMap::new();
        encoding_map.insert(symbol, vec![false]); // Use a single bit for the only symbol

        // Pack the encoded data
        let data_length = data.len();
        let full_bytes = data_length / 8;
        let remaining_bits = data_length % 8;

        let mut encoded_data = vec![
            0;
            if remaining_bits > 0 {
                full_bytes + 1
            } else {
                full_bytes
            }
        ];

        // For single-symbol case, we just need to pack the same bit value
        for i in 0..full_bytes {
            encoded_data[i] = 0; // All bits are 0 since we used false above
        }

        if remaining_bits > 0 {
            encoded_data[full_bytes] = 0;
        }

        return Ok(HuffmanCode {
            encoding_map,
            encoded_data,
            padding_bits: if remaining_bits > 0 {
                8 - remaining_bits as u8
            } else {
                0
            },
        });
    }

    // Create priority queue with nodes
    let mut heap = BinaryHeap::new();
    for (&value, &freq) in &frequencies {
        heap.push(Reverse(HuffmanNode {
            frequency: freq,
            value: Some(value),
            left: None,
            right: None,
        }));
    }

    // Build Huffman tree
    while heap.len() > 1 {
        let left = Box::new(heap.pop().unwrap().0);
        let right = Box::new(heap.pop().unwrap().0);

        let combined_freq = left.frequency + right.frequency;
        heap.push(Reverse(HuffmanNode {
            frequency: combined_freq,
            value: None,
            left: Some(left),
            right: Some(right),
        }));
    }

    let root = heap.pop().ok_or(HuffmanError::TreeCreationFailed)?.0;

    // Generate encoding map
    let mut encoding_map = HashMap::new();
    generate_codes(&root, &mut Vec::new(), &mut encoding_map);

    // Encode the data
    let mut encoded_bits: Vec<bool> = Vec::new();
    for &byte in data {
        encoded_bits.extend(encoding_map.get(&byte).unwrap());
    }

    // Pack bits into bytes
    let mut encoded_data = Vec::new();
    let mut current_byte = 0u8;
    let mut bit_count = 0;

    for &bit in &encoded_bits {
        current_byte = (current_byte << 1) | (bit as u8);
        bit_count += 1;

        if bit_count == 8 {
            encoded_data.push(current_byte);
            current_byte = 0;
            bit_count = 0;
        }
    }

    // Handle padding for last byte
    let padding_bits = if bit_count > 0 {
        let padding = 8 - bit_count;
        current_byte <<= padding;
        encoded_data.push(current_byte);
        padding as u8
    } else {
        0
    };

    Ok(HuffmanCode {
        encoding_map,
        encoded_data,
        padding_bits,
    })
}

fn generate_codes(
    node: &HuffmanNode,
    current_code: &mut Vec<bool>,
    encoding_map: &mut HashMap<u8, Vec<bool>>,
) {
    if let Some(value) = node.value {
        encoding_map.insert(value, current_code.clone());
        return;
    }

    if let Some(ref left) = node.left {
        current_code.push(false);
        generate_codes(left, current_code, encoding_map);
        current_code.pop();
    }

    if let Some(ref right) = node.right {
        current_code.push(true);
        generate_codes(right, current_code, encoding_map);
        current_code.pop();
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_huffman_basic_encoding() {
        let input = vec![1, 1, 1, 2, 2, 3];
        let result = huffman_encode(&input).unwrap();

        // Check that we got some encoded data
        assert!(!result.encoded_data.is_empty());

        // Check that the most frequent symbol (1) has the shortest code
        let code_1 = result.encoding_map.get(&1).unwrap();
        let code_2 = result.encoding_map.get(&2).unwrap();
        let code_3 = result.encoding_map.get(&3).unwrap();

        assert!(code_1.len() <= code_2.len());
        assert!(code_1.len() <= code_3.len());
    }

    #[test]
    fn test_huffman_single_byte() {
        // Test with a single byte repeated multiple times
        let input = vec![5, 5, 5, 5];
        let result = huffman_encode(&input).unwrap();

        // Check the encoding map
        assert_eq!(result.encoding_map.len(), 1);
        assert!(result.encoding_map.contains_key(&5));

        // The encoding for the single symbol should be a single bit
        let code = result.encoding_map.get(&5).unwrap();
        assert_eq!(code.len(), 1);

        // Check that we got compressed data
        assert!(!result.encoded_data.is_empty());

        // For 4 symbols encoded with 1 bit each, we should have 1 byte of data
        // (4 bits of data + 4 bits of padding = 1 byte)
        assert_eq!(result.encoded_data.len(), 1);
        assert_eq!(result.padding_bits, 4);
    }

    #[test]
    fn test_huffman_single_occurrence() {
        // Test with a single occurrence of a byte
        let input = vec![42];
        let result = huffman_encode(&input).unwrap();

        // Check the encoding map
        assert_eq!(result.encoding_map.len(), 1);
        assert!(result.encoding_map.contains_key(&42));

        // The encoding should be a single bit
        let code = result.encoding_map.get(&42).unwrap();
        assert_eq!(code.len(), 1);

        // Check the encoded data
        assert_eq!(result.encoded_data.len(), 1);
        assert_eq!(result.padding_bits, 7); // 1 bit of data + 7 bits of padding
    }

    #[test]
    fn test_huffman_single_byte_long_sequence() {
        // Test with a longer sequence of the same byte
        let input = vec![7; 100];
        let result = huffman_encode(&input).unwrap();

        // Check the encoding map
        assert_eq!(result.encoding_map.len(), 1);
        assert!(result.encoding_map.contains_key(&7));

        // The encoding should be a single bit
        let code = result.encoding_map.get(&7).unwrap();
        assert_eq!(code.len(), 1);

        // Calculate expected output size (100 bits packed into bytes)
        let expected_bytes = (100 + 7) / 8; // Round up to nearest byte
        assert_eq!(result.encoded_data.len(), expected_bytes);

        // Calculate expected padding
        let expected_padding = (expected_bytes * 8) - 100;
        assert_eq!(result.padding_bits, expected_padding as u8);
    }

    #[test]
    fn test_huffman_empty_input() {
        let input: Vec<u8> = vec![];
        let result = huffman_encode(&input);

        assert!(matches!(result, Err(HuffmanError::EmptyInput)));
    }

    #[test]
    fn test_huffman_prefix_property() {
        let input = vec![1, 1, 2, 2, 3, 3, 4, 4, 5];
        let result = huffman_encode(&input).unwrap();

        // Check that no code is a prefix of another code
        let codes: Vec<&Vec<bool>> = result.encoding_map.values().collect();
        for (i, code1) in codes.iter().enumerate() {
            for (j, code2) in codes.iter().enumerate() {
                if i != j {
                    assert!(!is_prefix(code1, code2));
                }
            }
        }
    }

    fn is_prefix(shorter: &[bool], longer: &[bool]) -> bool {
        if shorter.len() >= longer.len() {
            return false;
        }
        shorter.iter().zip(longer.iter()).all(|(a, b)| a == b)
    }
}
