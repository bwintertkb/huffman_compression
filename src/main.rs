use huffman_compression::{
    build_huffman_tree, generate_encode_map, huff_encode_bitvec, tally_frequency,
};

fn main() {
    let bytes = [
        1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
    ];
    let freq_buff = tally_frequency(&bytes);
    let huffnode = build_huffman_tree(freq_buff);
    let encoded_map = generate_encode_map(huffnode.unwrap());
    let encoded_map = huff_encode_bitvec(&bytes, &encoded_map);
}
