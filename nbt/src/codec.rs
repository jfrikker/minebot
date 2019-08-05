use bytes::{BufMut, BytesMut};
use crate::{NbtDecode, NbtEncode, NbtEncoder, VarNum};
use std::io::{ErrorKind, Read, Result, Write};
use std::net::TcpStream;
use std::time::Instant;

#[derive(Clone)]
pub struct NbtCodec {
    incoming: BytesMut,
    outgoing: BytesMut
}

impl NbtCodec {
    pub fn new() -> Self {
        NbtCodec {
            incoming: BytesMut::with_capacity(1024 * 1024),
            outgoing: BytesMut::with_capacity(1024 * 1024)
        }
    }

    pub fn send<W, P>(&mut self, mut out: W, packet: P) -> Result<()>
        where W: Write, P: NbtEncode {
        let item_len = packet.encoded_size();
        self.outgoing.reserve(item_len + (VarNum.encoded_size(&(item_len as i32))));
        VarNum.encode(&(item_len as i32), &mut self.outgoing);
        packet.encode(&mut self.outgoing);
        out.write_all(&self.outgoing.take())
    }

    pub fn receive<P>(&mut self, mut input: impl Read) -> Result<P>
        where P: NbtDecode {
        let len = decode_length(&mut input)?;
        self.incoming.reserve(len);
        self.incoming.resize(len, 0);
        input.read_exact(&mut self.incoming)?;
        let message = P::decode(&mut self.incoming.split_to(len).freeze());
        Ok(message)
    }

    pub fn receive_timeout<P>(&mut self, input: &mut TcpStream, until: Instant) -> Result<Option<P>>
        where P: NbtDecode {
        loop {
            match try_decode_length(&self.incoming) {
                (Some(len), used) => {
                    self.incoming.reserve(len + used);
                    if self.incoming.len() >= len + used {
                        self.incoming.split_to(used);
                        let message = P::decode(&mut self.incoming.split_to(len).freeze());
                        input.set_read_timeout(None)?;
                        return Ok(Some(message))
                    }
                }
                (None, used) => {
                    self.incoming.reserve(used + 1);
                }
            }

            let now = Instant::now();
            if now >= until {
                input.set_read_timeout(None)?;
                return Ok(None)
            }
            input.set_read_timeout(Some(until - now))?;
            unsafe {
                let read_result = input.read(self.incoming.bytes_mut());

                match read_result {
                    Err(e) => {
                        if e.kind() != ErrorKind::WouldBlock {
                            return Err(e)
                        }
                    }
                    Ok(s) => self.incoming.advance_mut(s)
                }
            };
        }
    }
}

fn decode_length(mut input: impl Read) -> Result<usize> {
    let mut result = 0;
    let mut buf: [u8; 1] = [0; 1];
    let mut read = 0;
    loop {
        input.read_exact(&mut buf)?;
        let byte = buf[0];
        result = result | ((byte as usize & 0x7F) << (read * 7));

        if byte & 0x80 == 0 {
            return Ok(result);
        }
        read += 1;
    }
}

fn try_decode_length(buf: &[u8]) -> (Option<usize>, usize) {
    let mut result = 0;
    let mut read = 0;
    for byte in buf {
        result = result | ((*byte as usize & 0x7F) << (read * 7));

        if byte & 0x80 == 0 {
            return (Some(result), read + 1);
        }
        read += 1;
    }

    (None, read)
}