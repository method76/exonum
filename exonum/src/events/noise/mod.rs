// spell-checker:ignore uint
use failure;

#[cfg(feature = "sodiumoxide-crypto")]
#[doc(inline)]
pub use self::wrappers::sodium_wrapper::{
    handshake::{HandshakeParams, NoiseHandshake},
    wrapper::{
        NoiseWrapper, HANDSHAKE_HEADER_LENGTH, MAX_HANDSHAKE_MESSAGE_LENGTH,
        MIN_HANDSHAKE_MESSAGE_LENGTH,
    },
};

use byteorder::{ByteOrder, LittleEndian};
use futures::future::Future;
use tokio_codec::Framed;
use tokio_io::{
    io::{read_exact, write_all},
    AsyncRead, AsyncWrite,
};

use events::{codec::MessagesCodec, error::into_failure};

pub mod error;
pub mod wrappers;

#[cfg(test)]
mod tests;

pub const MAX_MESSAGE_LENGTH: usize = 65_535;
pub const TAG_LENGTH: usize = 16;
pub const HEADER_LENGTH: usize = 4;

type HandshakeResult<S> =
    Box<dyn Future<Item = (Framed<S, MessagesCodec>, Vec<u8>), Error = failure::Error>>;

pub trait Handshake {
    fn listen<S: AsyncRead + AsyncWrite + 'static>(self, stream: S) -> HandshakeResult<S>;
    fn send<S: AsyncRead + AsyncWrite + 'static>(self, stream: S) -> HandshakeResult<S>;
}

pub struct HandshakeRawMessage(pub Vec<u8>);

impl HandshakeRawMessage {
    pub fn read<S: AsyncRead + 'static>(
        sock: S,
    ) -> impl Future<Item = (S, Self), Error = failure::Error> {
        let buf = vec![0_u8; HANDSHAKE_HEADER_LENGTH];
        // First `HANDSHAKE_HEADER_LENGTH` bytes of handshake message is the payload length
        // in little-endian, remaining bytes is the handshake payload. Therefore, we need to read
        // `HANDSHAKE_HEADER_LENGTH` bytes as a little-endian integer and than we need to read
        // remaining payload.
        read_exact(sock, buf)
            .and_then(|(stream, msg)| {
                let len = LittleEndian::read_uint(&msg, HANDSHAKE_HEADER_LENGTH);
                read_exact(stream, vec![0_u8; len as usize])
            })
            .map_err(into_failure)
            .and_then(|(stream, msg)| Ok((stream, HandshakeRawMessage(msg))))
    }

    pub fn write<S: AsyncWrite + 'static>(
        self,
        sock: S,
    ) -> impl Future<Item = (S, Vec<u8>), Error = failure::Error> {
        let len = self.0.len();
        debug_assert!(len < MAX_HANDSHAKE_MESSAGE_LENGTH);

        // First `HANDSHAKE_HEADER_LENGTH` bytes of handshake message
        // is the payload length in little-endian.
        let mut message = vec![0_u8; HANDSHAKE_HEADER_LENGTH];
        LittleEndian::write_uint(&mut message, len as u64, HANDSHAKE_HEADER_LENGTH);

        write_all(sock, message)
            .and_then(move |(sock, _)| write_all(sock, self.0))
            .map_err(into_failure)
    }
}
