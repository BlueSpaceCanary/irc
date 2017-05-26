//! Implementation of line-delimiting codec for Tokio.

use std::io;
use bytes::{BufMut, BytesMut};
use encoding::{DecoderTrap, EncoderTrap, EncodingRef};
use encoding::label::encoding_from_whatwg_label;
use tokio_io::codec::{Decoder, Encoder};

/// A line-based codec parameterized by an encoding.
pub struct LineCodec {
    encoding: EncodingRef,
}

impl LineCodec {
    /// Creates a new instance of LineCodec from the specified encoding.
    pub fn new(label: &str) -> io::Result<LineCodec> {
        encoding_from_whatwg_label(label).map(|enc| LineCodec { encoding: enc }).ok_or(
            io::Error::new(
                io::ErrorKind::InvalidInput,
                &format!("Attempted to use unknown codec {}.", label)[..]
            )
        )
    }
}

impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<String>> {
        if let Some(n) = src.as_ref().iter().position(|b| *b == b'\n') {
            // Remove the next frame from the buffer.
            let line = src.split_to(n + 1);

            // Remove the new-line from the buffer.
            src.split_to(1);

            // Decode the line using the codec's encoding.
            match self.encoding.decode(line.as_ref(), DecoderTrap::Replace) {
                Ok(data) => Ok(Some(data)),
                Err(data) => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    &format!("Failed to decode {} as {}.", data, self.encoding.name())[..]
                ))
            }
        } else {
            Ok(None)
        }
    }
}

impl Encoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn encode(&mut self, msg: String, dst: &mut BytesMut) -> io::Result<()> {
        // Encode the message using the codec's encoding.
        let data = try!(self.encoding.encode(&msg, EncoderTrap::Replace).map_err(|data| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                &format!("Failed to encode {} as {}.", data, self.encoding.name())[..]
            )
        }));

        // Write the encoded message to the output buffer.
        dst.put(&data);

        Ok(())
    }
}
