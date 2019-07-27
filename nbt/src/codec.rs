use bytes::BytesMut;
use crate::{NbtDecode, NbtEncode, NbtEncoder, VarNum};
use std::io::{Read, Result, Write};

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

    pub fn receive<R, P>(&mut self, mut input: R) -> Result<P>
        where R: Read, P: NbtDecode {
        let len = decode_length(&mut input)? as usize;
        self.incoming.reserve(len);
        self.incoming.resize(len, 0);
        input.read_exact(&mut self.incoming)?;
        let message = P::decode(&mut self.incoming.split_to(len).freeze());
        Ok(message)
    }
}

fn decode_length<R: Read>(mut input: R) -> Result<i32> {
    let mut result = 0;
    let mut buf: [u8; 1] = [0; 1];
    let mut read = 0;
    loop {
        input.read_exact(&mut buf)?;
        let byte = buf[0];
        result = result | ((byte as i32 & 0x7F) << (read * 7));

        if byte & 0x80 == 0 {
            return Ok(result);
        }
        read += 1;
    }
}