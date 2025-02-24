use std::collections::HashMap;

use bitvec::{field::BitField, order::Msb0, slice::BitSlice, vec::BitVec};

#[derive(Debug)]
pub struct FrequencyBuffer([u64; 255]);

pub fn tally_frequency(bytes: &[u8]) -> FrequencyBuffer {
    let mut fb = FrequencyBuffer([0; 255]);
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

#[derive(Debug, Clone, Copy)]
enum Edge {
    Left,
    Right,
}

impl From<Edge> for u8 {
    fn from(value: Edge) -> Self {
        match value {
            Edge::Left => 0,
            Edge::Right => 1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HuffNode {
    count: u64,
    child_left_edge: Option<Edge>,
    child_right_edge: Option<Edge>,
    child_left: Option<*const HuffNode>,
    child_right: Option<*const HuffNode>,
    leaf_value: Option<u8>, // This is the index, i.e. the byte value to encode
    current_depth: u8,
}

impl HuffNode {
    fn is_leaf(&self) -> bool {
        self.leaf_value.is_some()
    }

    fn display_with_indent(&self, indent: String, last: bool) -> String {
        let mut display = String::new();

        display.push_str(&indent);
        display.push_str(if last { "└─ " } else { "├─ " });
        display.push_str(&format!("Count: {}", self.count));
        if let Some(leaf_value) = self.leaf_value {
            display.push_str(&format!(" (Leaf Value: {})", leaf_value));
        }
        display.push('\n');

        let new_indent = if last {
            format!("{}    ", indent)
        } else {
            format!("{}│   ", indent)
        };

        let mut children = Vec::new();
        if let Some(left_ptr) = self.child_left {
            let left_node = unsafe { &*left_ptr };
            children.push(left_node);
        }
        if let Some(right_ptr) = self.child_right {
            let right_node = unsafe { &*right_ptr };
            children.push(right_node);
        }

        let last_index = children.len().saturating_sub(1);
        for (i, child) in children.iter().enumerate() {
            let is_last = i == last_index;
            display.push_str(&child.display_with_indent(new_indent.clone(), is_last));
        }
        display
    }

    /// Updated display method that prints the tree structure.
    fn display(&self) -> String {
        self.display_with_indent(String::new(), true)
    }
}

pub fn build_huffman_tree(freq_buffer: FrequencyBuffer) -> Option<Box<HuffNode>> {
    let mut buffer = freq_buffer.0;

    let mut current_huff_node = None;
    while let Some((idx, freq)) = find_and_pop_min(&mut buffer) {
        let node = match current_huff_node.take() {
            None => {
                let huff_node = Box::new(HuffNode {
                    count: freq,
                    child_left_edge: None,
                    child_right_edge: None,
                    child_left: None,
                    child_right: None,
                    leaf_value: Some(idx),
                    current_depth: 1,
                });

                current_huff_node = Some(huff_node);
                continue;
            }
            Some(node) => node,
        };

        if node.is_leaf() {
            // Current node is a leaf node, so we need to build the right leaf node and create a parent
            let huff_node_right_leaf = Box::new(HuffNode {
                count: freq,
                child_left_edge: None,
                child_right_edge: None,
                child_left: None,
                child_right: None,
                leaf_value: Some(idx),
                current_depth: 1,
            });

            let depth = node.current_depth;
            let parent_leaf_node = HuffNode {
                count: freq + node.count,
                child_left_edge: Some(Edge::Left),
                child_right_edge: Some(Edge::Right),
                child_left: Some(Box::leak(node)),
                child_right: Some(Box::leak(huff_node_right_leaf)),
                leaf_value: None,
                current_depth: depth + 1,
            };

            current_huff_node = Some(Box::new(parent_leaf_node));
            continue;
        }

        // Current node is not a leaf node, so we need to build the left leaf node and create a parent
        let huff_node_left_leaf = Box::new(HuffNode {
            count: freq,
            child_left_edge: None,
            child_right_edge: None,
            child_left: None,
            child_right: None,
            leaf_value: Some(idx),
            current_depth: node.current_depth,
        });

        let depth = node.current_depth;
        let parent_leaf_node = HuffNode {
            count: freq + node.count,
            child_left_edge: Some(Edge::Left),
            child_right_edge: Some(Edge::Right),
            child_left: Some(Box::leak(huff_node_left_leaf)),
            child_right: Some(Box::leak(node)),
            leaf_value: None,
            current_depth: depth + 1,
        };

        current_huff_node = Some(Box::new(parent_leaf_node));
    }

    current_huff_node
}

pub fn huff_encode_bitvec(bytes: &[u8], encoded_map: &HashMap<u8, Encoded>) -> (Vec<u8>, u64) {
    let mut final_bits: BitVec<u8, Msb0> = BitVec::with_capacity(bytes.len() / 2);
    for byte in bytes {
        let encoded = encoded_map.get(byte).unwrap();
        let num_bytes = encoded.bits.len();

        let mut shift_amount = encoded.shift as u32;

        (0..num_bytes - 1).for_each(|_| {
            for _ in 0..u8::BITS {
                final_bits.push(false);
            }
            shift_amount -= u8::BITS;
        });

        (0..shift_amount).for_each(|_| {
            final_bits.push(false);
        });

        if encoded.bits[encoded.bits.len() - 1] == 0 {
            final_bits.push(false);
        } else {
            final_bits.push(true);
        }
    }

    let total_bits = final_bits.len();
    (final_bits.into(), total_bits as u64)
}

#[derive(Debug)]
pub struct Encoded {
    bits: Vec<u8>,
    shift: u8,
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
        tmp_buffer.push(encoded.shift + 1);
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
    println!("TOTAL HUFF BYTES: {}", huff_bytes.len());
    let total_bits = u8_to_u64(&huff_bytes[0..8]);
    let header_num_bytes = u8_to_u64(&huff_bytes[8..16]);

    println!("TOTAL BITS {}", total_bits);
    println!("HEADER NUM BYTES {}", header_num_bytes);
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

    println!("encoded_map: {:?}", value_bit_map);
    println!("final idx: {:?}", idx);

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
                // println!("cursor: {:?}", cursor);
                cursor += *num_bits as u64;
                // println!("cursor-post: {:?}", cursor);
                // println!("decoded_buffer mid: {:?}", decoded_buffer);
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

pub fn generate_encode_map(node: Box<HuffNode>) -> HashMap<u8, Encoded> {
    if let Some(value) = node.leaf_value {
        let bits: u8 = 1 << 7;
        let encoded = Encoded {
            bits: vec![bits],
            shift: 0,
            original_value: value,
        };
        return HashMap::from([(value, encoded)]);
    }

    let mut encode_map = HashMap::with_capacity(1024);
    let mut current_node = node;
    let mut shift = 7;
    let mut depth_level = 0;
    while current_node.child_right.is_some() {
        let left_child: Box<HuffNode> =
            unsafe { Box::from_raw(current_node.child_left.unwrap() as *mut HuffNode) };

        let mut byte_vec = Vec::with_capacity(depth_level / 8 + 1);
        if byte_vec.capacity() >= 2 {
            for _ in 0..(byte_vec.capacity() - 1) {
                byte_vec.push(0);
            }
        }
        byte_vec.push(1 << shift);

        let encoded = Encoded {
            bits: byte_vec,
            shift: depth_level as u8,
            original_value: left_child.leaf_value.unwrap(),
        };
        encode_map.insert(left_child.leaf_value.unwrap(), encoded);
        shift -= 1;
        if shift < 0 {
            shift = 7;
        }

        current_node = unsafe { Box::from_raw(current_node.child_right.unwrap() as *mut HuffNode) };
        depth_level += 1;
    }

    // Need to add the last right node
    let mut byte_vec = Vec::with_capacity(depth_level / 8 + 1);
    if byte_vec.capacity() > 2 {
        for _ in 0..(byte_vec.capacity() - 1) {
            byte_vec.push(0);
        }
    }
    byte_vec.push(0);
    let encoded = Encoded {
        bits: byte_vec,
        shift: (depth_level - 1) as u8,
        original_value: current_node.leaf_value.unwrap(),
    };
    encode_map.insert(current_node.leaf_value.unwrap(), encoded);

    encode_map
}

// 00100000, should return 2 for the index of 1, None is returned if no 1 is found
fn find_bit_index_1_position(byte: u8) -> Option<u8> {
    let mut idx = None;
    let mut cmp_byte: u8 = 128;

    for loop_idx in 0..8 {
        cmp_byte >>= loop_idx;
        if cmp_byte & byte == 0 {
            continue;
        }
        idx = Some(loop_idx);
        break;
    }

    idx
}

fn encoded_buffer_to_string(buffer: &[u8]) -> String {
    buffer
        .iter()
        .map(|b| format!("{:08b}", b))
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn build_huffman_tree_test_simple() {
        let bytes = [1, 2, 1, 1, 1, 1, 1, 1, 1, 3, 1];
        let freq_buff = tally_frequency(&bytes);
        let huffnode = build_huffman_tree(freq_buff);
        let encode_map = generate_encode_map(huffnode.unwrap());
        let (encoded_buffer, total_bits) = huff_encode_bitvec(&bytes, &encode_map);
        let expected_buffer = "1011111111001000";
        assert_eq!(encoded_buffer_to_string(&encoded_buffer), expected_buffer);
        let expected_total_bits = 13;
        assert_eq!(total_bits, expected_total_bits);
    }

    #[test]
    fn build_huffman_tree_test_bitvec() {
        let bytes = [
            1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1,
        ];
        let freq_buff = tally_frequency(&bytes);
        let huffnode = build_huffman_tree(freq_buff);
        let encoded_map = generate_encode_map(huffnode.unwrap());
        let (encoded_buffer, total_bits) = huff_encode_bitvec(&bytes, &encoded_map);

        let expected_buffer = "1100000000000000001000000000000000000000000000000001000000000000001000000000000010000000000001000000000001000000000010000000001000000001000000010000001000001000010001001011111111111111";
        assert_eq!(encoded_buffer_to_string(&encoded_buffer), expected_buffer);
        assert_eq!(total_bits, 184)
    }

    #[test]
    fn test_serialize_huffman() {
        // let bytes = [1, 2, 1, 1, 1, 1, 1, 1, 1, 3, 1];
        let bytes = [
            1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1,
        ];
        let freq_buff = tally_frequency(&bytes);
        let huffnode = build_huffman_tree(freq_buff);
        let encode_map = generate_encode_map(huffnode.unwrap());
        let (encoded_buffer, total_bits) = huff_encode_bitvec(&bytes, &encode_map);
        let serialized_huffman = serialize_huffman(&encode_map, encoded_buffer, total_bits);
        println!("{:?}", serialized_huffman);

        std::fs::write("./test_dump", &serialized_huffman);
        let bytes = std::fs::read("./test_dump").unwrap();
        println!("FINAL READ BYTES: {:?}", bytes);
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

    #[test]
    fn test_deserialize_huffman_complex() {
        let target = [
            1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 99,
        ];
        let freq_buff = tally_frequency(&target);
        let huffnode = build_huffman_tree(freq_buff);
        let encode_map = generate_encode_map(huffnode.unwrap());
        let (encoded_buffer, total_bits) = huff_encode_bitvec(&target, &encode_map);
        println!("{:?}", encoded_buffer_to_string(&encoded_buffer));
        let serialized_huffman = serialize_huffman(&encode_map, encoded_buffer, total_bits);
        println!("{:?}", serialized_huffman);

        std::fs::write("./test_dump", &serialized_huffman);
        let serialized_bytes = std::fs::read("./test_dump").unwrap();

        let actual = deserialze_huffman(&serialized_bytes);
        println!("ACTUAL DECODED: {:?}", actual);

        assert_eq!(actual, target);
    }
}
