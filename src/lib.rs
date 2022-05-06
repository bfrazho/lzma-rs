//! Pure-Rust codecs for LZMA, LZMA2, and XZ.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![forbid(unsafe_code)]

#[macro_use]
mod macros;
mod decode_internal;
/// module for decoding lzma file
pub mod decode;
mod encode;
pub mod error;
mod xz;

use decode::lzma_params::LzmaParams;

use crate::decode_internal::lzbuffer::LzBuffer;
use std::io;

/// Compression helpers.
pub mod compress {
    pub use crate::encode::options::*;
}

/// Decompression helpers.
pub mod decompress {
    pub use crate::decode::options::*;
    #[cfg(feature = "stream")]
    pub use crate::decode::stream::Stream;
}

/// Decompress LZMA data with default [`Options`](decompress/struct.Options.html).
pub fn lzma_decompress<R: io::BufRead, W: io::Write>(
    input: &mut R,
    output: &mut W,
) -> error::Result<()> {
    lzma_decompress_with_options(input, output, &decompress::Options::default())
}

/// Decompress LZMA data with the provided options.
pub fn lzma_decompress_with_options<R: io::BufRead, W: io::Write>(
    input: &mut R,
    output: &mut W,
    options: &decompress::Options,
) -> error::Result<()> {
    let params = decode::lzma_params::LzmaParams::read_header(input, options)?;
    lzma_decompress_with_options_with_lzma_params(input, output, options, params)
}

/// Decompress LZMA data with the provided options and params
pub fn lzma_decompress_with_options_with_lzma_params<R: io::BufRead, W: io::Write>(
    input: &mut R,
    output: &mut W,
    options: &decompress::Options,
    lzma_params: LzmaParams,
) -> error::Result<()> {
    let mut decoder = if let Some(memlimit) = options.memlimit {
        decode_internal::lzma::new_circular_with_memlimit(output, lzma_params, memlimit)?
    } else {
        decode_internal::lzma::new_circular(output, lzma_params)?
    };

    let mut rangecoder = decode_internal::rangecoder::RangeDecoder::new(input)
        .map_err(|e| error::Error::LzmaError(format!("LZMA stream too short: {}", e)))?;
    decoder.process(&mut rangecoder)?;
    decoder.output.finish()?;
    Ok(())
}

/// Compresses data with LZMA and default [`Options`](compress/struct.Options.html).
pub fn lzma_compress<R: io::BufRead, W: io::Write>(
    input: &mut R,
    output: &mut W,
) -> io::Result<()> {
    lzma_compress_with_options(input, output, &compress::Options::default())
}

/// Compress LZMA data with the provided options.
pub fn lzma_compress_with_options<R: io::BufRead, W: io::Write>(
    input: &mut R,
    output: &mut W,
    options: &compress::Options,
) -> io::Result<()> {
    let encoder = encode::dumbencoder::Encoder::from_stream(output, options)?;
    encoder.process(input)
}

/// Decompress LZMA2 data with default [`Options`](decompress/struct.Options.html).
pub fn lzma2_decompress<R: io::BufRead, W: io::Write>(
    input: &mut R,
    output: &mut W,
) -> error::Result<()> {
    decode_internal::lzma2::decode_stream(input, output)
}

/// Compress data with LZMA2 and default [`Options`](compress/struct.Options.html).
pub fn lzma2_compress<R: io::BufRead, W: io::Write>(
    input: &mut R,
    output: &mut W,
) -> io::Result<()> {
    encode::lzma2::encode_stream(input, output)
}

/// Decompress XZ data with default [`Options`](decompress/struct.Options.html).
pub fn xz_decompress<R: io::BufRead, W: io::Write>(
    input: &mut R,
    output: &mut W,
) -> error::Result<()> {
    decode_internal::xz::decode_stream(input, output)
}

/// Compress data with XZ and default [`Options`](compress/struct.Options.html).
pub fn xz_compress<R: io::BufRead, W: io::Write>(input: &mut R, output: &mut W) -> io::Result<()> {
    encode::xz::encode_stream(input, output)
}
