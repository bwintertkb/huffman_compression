use huffman_compression::*;

const FILE_PATH: &str = "./tests/resources/dna_seq_test.txt";

/// Serialize to deserialize test
#[test]
fn serialize_and_deserialize_test() {
    let original: String = std::fs::read_to_string(FILE_PATH).unwrap();

    let bytes = original.as_bytes();
    let freq_buff = tally_frequency(bytes);
    let huffnode = build_huffman_array(freq_buff);
    let encoded_map = encode_huffman_array(&huffnode);
    let (bit_buffer, total_bits) = huff_encode_bitvec(bytes, &encoded_map);
    let serialized_buffer = serialize_huffman(&encoded_map, bit_buffer, total_bits);
    let deserialized_bytes = deserialze_huffman(&serialized_buffer);

    let actual = String::from_utf8_lossy(&deserialized_bytes);

    assert_eq!(actual, original);
}
