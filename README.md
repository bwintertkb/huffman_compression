# huffc - Huffman Compression Tool

`huffc` is a CLI tool for compressing and decompressing files using Huffman encoding. It supports both file-based and standard input compression, allowing users to efficiently encode and decode data.

## Installation

You can install `huffc` using Cargo:

```sh
cargo install huffc
```

## Usage

`huffc` provides options for both compression and decompression. You must specify either `--compress` or `--decompress`, but not both.

### Compressing a File

To compress a file and save the output:

```sh
huffc --compress -i input.txt -o compressed.huff
```

### Decompressing a File

To decompress a `.huff` file back to its original form:

```sh
huffc --decompress -i compressed.huff -o output.txt
```

### Using Standard Input

You can also use standard input for compression or decompression:

#### Compressing Standard Input

```sh
echo "Hello, Huffman!" | huffc --compress -o output.huff
```

#### Decompressing Standard Input

```sh
cat output.huff | huffc --decompress -o output.txt
```

## Arguments

| Flag | Description |
|------|-------------|
| `-c, --compress` | Compress a file or standard input |
| `-d, --decompress` | Decompress a file or standard input |
| `-i, --input <FILE>` | Specify the input file (optional for stdin) |
| `-o, --out-file <FILE>` | Specify the output file (required for stdin) |

## Error Handling

`huffc` provides meaningful error messages when incorrect arguments are used:

- **`Error: You must specify either --compress or --decompress, but not both.`**
- **`No file path provided. Use --help for more information.`**
- **`File does not exist in path provided.`**
- **`File does not have the file extension '.huff'`** (for decompression)
- **`No outfile path provided.`** (when using standard input)

## License

This project is licensed under the MIT License.
