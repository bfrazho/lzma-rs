use std::io;
use crate::{error, decode::options::UnpackedSize};
use byteorder::{ReadBytesExt, LittleEndian};

use super::options::Options;

///Parameters that describe how the Lzma file is structured
#[derive(Debug)]
pub struct LzmaParams {
    /// most lc significant bits of previous byte are part of the literal context
    pub lc: u32, // 0..8
    /// literal position of lzma file
    pub lp: u32, // 0..4
    /// context for literal/match is plaintext offset modulo 2^pb
    pub pb: u32, // 0..4
    /// dictionary size of Lzma file
    pub dict_size: u32,
    /// the unpacked size of Lzma file
    pub unpacked_size: Option<u64>,
}

impl LzmaParams {
    /// read header of a lzma file
    pub fn read_header<R>(input: &mut R, options: &Options) -> error::Result<LzmaParams>
    where
        R: io::BufRead,
    {
        // Properties
        let props = input.read_u8().map_err(error::Error::HeaderTooShort)?;

        let mut pb = props as u32;
        if pb >= 225 {
            return Err(error::Error::LzmaError(format!(
                "LZMA header invalid properties: {} must be < 225",
                pb
            )));
        }

        let lc: u32 = pb % 9;
        pb /= 9;
        let lp: u32 = pb % 5;
        pb /= 5;

        lzma_info!("Properties {{ lc: {}, lp: {}, pb: {} }}", lc, lp, pb);

        // Dictionary
        let dict_size_provided = input
            .read_u32::<LittleEndian>()
            .map_err(error::Error::HeaderTooShort)?;
        let dict_size = if dict_size_provided < 0x1000 {
            0x1000
        } else {
            dict_size_provided
        };

        lzma_info!("Dict size: {}", dict_size);

        // Unpacked size
        let unpacked_size: Option<u64> = match options.unpacked_size {
            UnpackedSize::ReadFromHeader => {
                let unpacked_size_provided = input
                    .read_u64::<LittleEndian>()
                    .map_err(error::Error::HeaderTooShort)?;
                let marker_mandatory: bool = unpacked_size_provided == 0xFFFF_FFFF_FFFF_FFFF;
                if marker_mandatory {
                    None
                } else {
                    Some(unpacked_size_provided)
                }
            }
            UnpackedSize::ReadHeaderButUseProvided(x) => {
                input
                    .read_u64::<LittleEndian>()
                    .map_err(error::Error::HeaderTooShort)?;
                x
            }
            UnpackedSize::UseProvided(x) => x,
        };

        lzma_info!("Unpacked size: {:?}", unpacked_size);

        let params = LzmaParams {
            lc,
            lp,
            pb,
            dict_size,
            unpacked_size,
        };

        Ok(params)
    }
}
