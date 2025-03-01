//! Huffc Core - Huffman Encoding and Decoding
//!
//! This module implements the Huffman encoding and decoding logic,
//! including frequency analysis, tree construction, and bitwise encoding.
//!
//! ## Features
//! - Computes symbol frequencies efficiently
//! - Builds a Huffman tree to encode data optimally
//! - Encodes and decodes data using bitwise representations
//! - Supports serialization and deserialization of Huffman-encoded data
pub mod cli;
pub mod fs;

use std::collections::{HashMap, VecDeque};

use bitvec::{order::Msb0, slice::BitSlice, vec::BitVec};

#[derive(Debug)]
pub struct FrequencyBuffer(pub [u64; 256]);

pub fn tally_frequency(bytes: &[u8]) -> FrequencyBuffer {
    let mut fb = FrequencyBuffer([0; 256]);
    bytes.iter().for_each(|byte| unsafe {
        *fb.0.as_mut_ptr().add(*byte as usize) += 1;
    });
    fb
}

// Returns the index of the minimum value, Option because when all values are zero there is nothing
// to pop
type Idx = u8;
type Freq = u64;
pub fn find_and_pop_min(freq_buf: &mut [u64]) -> Option<(Idx, Freq)> {
    let mut min_value_idx = None;
    let mut min_value = None;
    freq_buf.iter_mut().enumerate().for_each(|(idx, byte)| {
        if *byte == 0 {
            return;
        }

        match min_value {
            None => {
                min_value = Some(*byte);
                min_value_idx = Some(idx as u8);
            }
            Some(v) if *byte < v => {
                min_value = Some(*byte);
                min_value_idx = Some(idx as u8);
            }
            _ => (),
        }
    });

    if let Some(idx) = min_value_idx {
        let byte = unsafe { freq_buf.get_unchecked_mut(idx as usize) };
        *byte = 0;
    }

    match (min_value_idx, min_value) {
        (Some(idx), Some(value)) => Some((idx, value)),
        _ => None,
    }
}

pub fn huff_encode_bitvec(bytes: &[u8], encoded_map: &HashMap<u8, Encoded>) -> (Vec<u8>, u64) {
    let mut ctr = 0;
    let mut final_bits: BitVec<u8, Msb0> = BitVec::with_capacity(bytes.len() / 2);
    for byte in bytes {
        let encoded = encoded_map.get(byte).unwrap();

        let num_bits = encoded.num_bits_sequence;
        // Check if it's the least frequent bit
        let last_value_1 = encoded.bits[encoded.bits.len() - 1] != 0;

        (0..num_bits - 1).for_each(|_| {
            final_bits.push(false);
        });

        if last_value_1 {
            final_bits.push(true);
        } else {
            final_bits.push(false);
        }

        ctr += 1;

        if ctr % 10000 == 0 {
            println!(
                "PCT Complete: {:.2}",
                (ctr as f64 / bytes.len() as f64) * 100.0
            )
        }
    }

    let total_bits = final_bits.len();
    println!("Total bits: {}", total_bits);
    (final_bits.into(), total_bits as u64)
}

#[derive(Debug, PartialEq, Eq)]
pub struct Encoded {
    bits: Vec<u8>,
    /// Number of bits in the sequence
    num_bits_sequence: u8,
    original_value: u8,
}

fn u64_to_u8(value: u64) -> [u8; 8] {
    [
        (value >> 56) as u8,
        (value >> 48) as u8,
        (value >> 40) as u8,
        (value >> 32) as u8,
        (value >> 24) as u8,
        (value >> 16) as u8,
        (value >> 8) as u8,
        value as u8,
    ]
}

pub fn serialize_huffman(
    encoded_map: &HashMap<u8, Encoded>,
    bit_buffer: Vec<u8>,
    total_bits: u64,
) -> Vec<u8> {
    let mut serialized_buffer = u64_to_u8(total_bits).to_vec();

    let mut tmp_buffer = Vec::new();

    for encoded in encoded_map.values() {
        tmp_buffer.push(encoded.original_value);
        tmp_buffer.push(encoded.num_bits_sequence);
        tmp_buffer.extend_from_slice(&encoded.bits);
    }

    let size_of_header_bytes = tmp_buffer.len() as u64;
    let size_of_header_arr = u64_to_u8(size_of_header_bytes);
    serialized_buffer.extend_from_slice(&size_of_header_arr);
    serialized_buffer.extend(tmp_buffer);
    serialized_buffer.extend(bit_buffer);

    serialized_buffer
}

#[derive(Debug)]
struct ValueBitMap<'a> {
    values: Vec<u8>,
    bits: Vec<&'a BitSlice<u8, Msb0>>,
    num_bits: Vec<u8>,
}

pub fn deserialze_huffman(huff_bytes: &[u8]) -> Vec<u8> {
    let total_bits = u8_to_u64(&huff_bytes[0..8]);
    let header_num_bytes = u8_to_u64(&huff_bytes[8..16]);

    let mut encoded_map: HashMap<u8, (u8, &[u8])> = HashMap::new();

    let mut value_bit_map = ValueBitMap {
        values: Vec::new(),
        bits: Vec::new(),
        num_bits: Vec::new(),
    };
    let mut idx = 16;
    while idx as u64 - 16 < header_num_bytes {
        let value = huff_bytes[idx];
        let encoding_number_of_bits = huff_bytes[idx + 1];
        let idx_increment = encoding_number_of_bits.div_ceil(8);

        value_bit_map.values.push(value);
        value_bit_map.num_bits.push(encoding_number_of_bits);
        value_bit_map.bits.push(BitSlice::from_slice(
            &huff_bytes[idx + 2..idx + 2 + idx_increment as usize],
        ));
        encoded_map.insert(
            value,
            (
                encoding_number_of_bits,
                &huff_bytes[idx + 2..idx + 2 + idx_increment as usize],
            ),
        );
        idx += 2 + idx_increment as usize;
    }

    let mut decoded_buffer: Vec<u8> = Vec::new();
    let mut cursor = 0;
    let bit_vec: BitVec<u8, Msb0> = BitVec::from_slice(&huff_bytes[idx..]);

    while cursor < total_bits {
        for ((bits, value), num_bits) in value_bit_map
            .bits
            .iter()
            .zip(value_bit_map.values.iter())
            .zip(value_bit_map.num_bits.iter())
        {
            let bit_slice = &bits[..*num_bits as usize];
            if cursor + *num_bits as u64 > bit_vec.len() as u64 {
                continue;
            }

            let bit_vec_slice = &bit_vec[cursor as usize..cursor as usize + *num_bits as usize];
            if bit_slice == bit_vec_slice {
                decoded_buffer.push(*value);
                cursor += *num_bits as u64;
                break;
            }
        }
    }

    decoded_buffer
}

fn u8_to_u64(bytes: &[u8]) -> u64 {
    assert!(bytes.len() >= 8);
    (bytes[0] as u64) << 56
        | (bytes[1] as u64) << 48
        | (bytes[2] as u64) << 40
        | (bytes[3] as u64) << 32
        | (bytes[4] as u64) << 24
        | (bytes[5] as u64) << 16
        | (bytes[6] as u64) << 8
        | bytes[7] as u64
}

/// Build huffman array, this represents the huffman tree, the first index is encoded 1, the next
/// is 01, the next 001 and so on... until the last one which is encoded 0 (repeated n) where n is
/// the length of the vector, the least frequent is at the back the most frequent is at the front,
/// the actual frequency does not matter, only their relative frequency, which is represented by
/// their position in the buffer
pub fn build_huffman_array(mut freq_buffer: FrequencyBuffer) -> Vec<u8> {
    let mut buffer = VecDeque::new();
    while let Some((idx, _)) = find_and_pop_min(&mut freq_buffer.0) {
        buffer.push_front(idx);
    }
    buffer.into()
}

pub fn encode_huffman_array(huffman_array: &[u8]) -> HashMap<u8, Encoded> {
    huffman_array
        .iter()
        .enumerate()
        .map(|(idx, value)| {
            if idx == huffman_array.len() - 1 {
                return (
                    *value,
                    Encoded {
                        bits: vec![0; idx.div_ceil(8)],
                        num_bits_sequence: idx as u8,
                        original_value: *value,
                    },
                );
            }

            let div_v = idx / 8;

            if div_v == 0 {
                (
                    *value,
                    Encoded {
                        bits: vec![1 << (7 - idx)],
                        num_bits_sequence: idx as u8 + 1,
                        original_value: *value,
                    },
                )
            } else {
                let shift_amount = idx - (div_v * 8);
                (
                    *value,
                    Encoded {
                        bits: vec![1 << (7 - shift_amount)],
                        num_bits_sequence: idx as u8 + 1,
                        original_value: *value,
                    },
                )
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded_buffer_to_string(buffer: &[u8]) -> String {
        buffer
            .iter()
            .map(|b| format!("{:08b}", b))
            .collect::<String>()
    }

    #[test]
    fn find_and_pop_min_not_empty() {
        let mut bytes = [4, 8, 1];
        let result = find_and_pop_min(&mut bytes);

        assert!(result == Some((2, 1)));
        assert!(bytes == [4, 8, 0]);
    }

    #[test]
    fn find_and_pop_min_empty() {
        let mut bytes = [0, 0, 0];
        let result = find_and_pop_min(&mut bytes);
        assert!(result.is_none());
    }

    #[test]
    fn build_small_huffman_array() {
        let bytes = [1, 2, 1, 1, 1, 1, 1, 1, 1, 3, 1];
        let freq_buff = tally_frequency(&bytes);
        let actual = build_huffman_array(freq_buff);
        let expected = vec![1, 3, 2];
        assert_eq!(actual, expected)
    }

    #[test]
    fn build_encoded_map_from_huffman_array() {
        let huff_arr = vec![1, 3, 2];
        let actual = encode_huffman_array(&huff_arr);

        let mut expected = HashMap::new();
        expected.insert(
            1,
            Encoded {
                bits: vec![1 << 7],
                num_bits_sequence: 1,
                original_value: 1,
            },
        );
        expected.insert(
            3,
            Encoded {
                bits: vec![1 << 6],
                num_bits_sequence: 2,
                original_value: 3,
            },
        );
        expected.insert(
            2,
            Encoded {
                bits: vec![0],
                num_bits_sequence: 2,
                original_value: 2,
            },
        );

        assert_eq!(actual, expected)
    }

    #[test]
    fn build_huffman_tree_test_simple() {
        let bytes = [1, 2, 1, 1, 1, 1, 1, 1, 1, 3, 1];
        let freq_buff = tally_frequency(&bytes);
        let huffnode = build_huffman_array(freq_buff);
        let encode_map = encode_huffman_array(&huffnode);
        let (encoded_buffer, total_bits) = huff_encode_bitvec(&bytes, &encode_map);
        let expected_buffer = "1001111111011000";
        assert_eq!(encoded_buffer_to_string(&encoded_buffer), expected_buffer);
        let expected_total_bits = 13;
        assert_eq!(total_bits, expected_total_bits);
    }

    #[test]
    fn test_deserialize_huffman() {
        let target = [1, 2, 1, 1, 1, 1, 1, 1, 1, 3, 1];

        let serialized_bytes = [
            0, 0, 0, 0, 0, 0, 0, 13, 0, 0, 0, 0, 0, 0, 0, 9, 2, 2, 64, 3, 2, 0, 1, 1, 128, 191, 200,
        ];

        let actual = deserialze_huffman(&serialized_bytes);

        assert_eq!(actual, target);
    }
}
