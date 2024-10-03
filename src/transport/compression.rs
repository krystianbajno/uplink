use flate2::write::GzEncoder;
use flate2::Compression;
use flate2::read::GzDecoder;
use std::io::prelude::*;

pub fn compress(data: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).expect("Compression failed");
    encoder.finish().expect("Compression failed")
}

pub fn decompress(data: &[u8]) -> Vec<u8> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data).expect("Decompression failed");
    decompressed_data
}
