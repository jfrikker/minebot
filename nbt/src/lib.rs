pub mod codec;

pub use codec::NbtCodec;

use bytes::{Bytes, IntoBuf};
use bytes::buf::{Buf, BufMut};
use json::{self, JsonValue};
use uuid::Uuid;
use std::convert::AsRef;
use std::fmt::{self, Debug};
use std::rc::Rc;
use std::str::from_utf8;
use std::fmt::Display;
use std::fmt::Formatter;

pub trait NbtDecode {
    fn decode(buf: &mut Bytes) -> Self;
}

pub trait NbtEncode {
    fn encoded_size(&self) -> usize;
    fn encode<B: BufMut>(&self, buf: &mut B);
}

pub trait NbtDecoder<T> {
    fn decode(&self, buf: &mut Bytes) -> T;
}

pub trait NbtEncoder<T> {
    fn encoded_size(&self, val: &T) -> usize;
    fn encode<B: BufMut>(&self, val: &T, buf: &mut B);
}

#[derive(Debug)]
pub struct VarNum;

impl NbtDecoder<i32> for VarNum {
    fn decode(&self, buf: &mut Bytes) -> i32 {
        let mut result = 0;
        let mut read = 0;
        loop {
            let byte = buf[0];
            buf.advance(1);
            result = result | ((byte as i32 & 0x7F) << (read * 7));

            if byte & 0x80 == 0 {
                return result;
            }
            read += 1;
        }
    }
}

impl NbtEncoder<i32> for VarNum {
    fn encoded_size(&self, val: &i32) -> usize {
        if *val < 0 {
            5
        } else if *val <= 0x7f {
            1
        } else if *val <= 0x3fff {
            2
        } else if *val <= 0x1fffff {
            3
        } else if *val <= 0xfffffff {
            4
        } else {
            5
        }
    }

    fn encode<B: BufMut>(&self, val: &i32, buf: &mut B) {
        let mut local_val = *val;
        if local_val == 0 {
            buf.put_u8(0);
        } else {
            while local_val != 0 {
                let mut byte = (local_val & 0x7f) as u8;
                local_val >>= 7;
                if local_val != 0 {
                    byte |= 0x80;
                }
                buf.put_u8(byte);
            }
        }
    }
}

#[derive(Clone)]
pub struct NbtString {
    bytes: Bytes
}

impl AsRef<str> for NbtString {
    fn as_ref(&self) -> &str {
        from_utf8(&self.bytes[..]).unwrap()
    }
}

impl Debug for NbtString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let as_str: &str = self.as_ref();
        write!(f, "NbtString({:?})", as_str)
    }
}

impl Display for NbtString {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.as_ref())
    }
}

impl NbtDecode for NbtString {
    fn decode(buf: &mut Bytes) -> Self {
        let len = VarNum.decode(buf);
        NbtString {
            bytes: buf.split_to(len as usize)
        }
    }
}

impl Into<String> for NbtString {
    fn into(self) -> String {
        String::from(self.as_ref())
    }
}

impl Into<String> for &NbtString {
    fn into(self) -> String {
        String::from(self.as_ref())
    }
}

impl NbtEncode for str {
    fn encoded_size(&self) -> usize {
        let len = self.len();
        VarNum.encoded_size(&(len as i32)) + len
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        let len = self.len();
        VarNum.encode(&(len as i32), buf);
        buf.put_slice(self.as_ref());
    }
}

impl NbtDecode for bool {
    fn decode(buf: &mut Bytes) -> Self {
        buf.split_to(1).into_buf().get_u8() > 0
    }
}

impl NbtEncode for bool {
    fn encoded_size(&self) -> usize {
        1
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_u8(if *self {1} else {0});
    }
}

impl NbtDecode for u8 {
    fn decode(buf: &mut Bytes) -> Self {
        buf.split_to(1).into_buf().get_u8()
    }
}

impl NbtEncode for u8 {
    fn encoded_size(&self) -> usize {
        1
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_u8(*self);
    }
}

impl NbtDecode for u16 {
    fn decode(buf: &mut Bytes) -> Self {
        buf.split_to(2).into_buf().get_u16_be()
    }
}

impl NbtEncode for u16 {
    fn encoded_size(&self) -> usize {
        2
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_u16_be(*self);
    }
}

impl NbtDecode for i16 {
    fn decode(buf: &mut Bytes) -> Self {
        buf.split_to(2).into_buf().get_i16_be()
    }
}

impl NbtEncode for i16 {
    fn encoded_size(&self) -> usize {
        2
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_i16_be(*self);
    }
}

impl NbtDecode for i32 {
    fn decode(buf: &mut Bytes) -> Self {
        buf.split_to(4).into_buf().get_i32_be()
    }
}

impl NbtEncode for i32 {
    fn encoded_size(&self) -> usize {
        4
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_i32_be(*self);
    }
}

impl NbtDecode for i64 {
    fn decode(buf: &mut Bytes) -> Self {
        buf.split_to(8).into_buf().get_i64_be()
    }
}

impl NbtEncode for i64 {
    fn encoded_size(&self) -> usize {
        8
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_i64_be(*self);
    }
}

impl NbtDecode for u64 {
    fn decode(buf: &mut Bytes) -> Self {
        buf.split_to(8).into_buf().get_u64_be()
    }
}

impl NbtEncode for u64 {
    fn encoded_size(&self) -> usize {
        8
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_u64_be(*self);
    }
}

impl NbtDecode for f32 {
    fn decode(buf: &mut Bytes) -> Self {
        buf.split_to(4).into_buf().get_f32_be()
    }
}

impl NbtEncode for f32 {
    fn encoded_size(&self) -> usize {
        4
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_f32_be(*self);
    }
}

impl NbtDecode for f64 {
    fn decode(buf: &mut Bytes) -> Self {
        buf.split_to(8).into_buf().get_f64_be()
    }
}

impl NbtEncode for f64 {
    fn encoded_size(&self) -> usize {
        8
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_f64_be(*self);
    }
}

impl <T: NbtEncode> NbtEncode for Rc<T> {
    fn encoded_size(&self) -> usize {
        self.as_ref().encoded_size()
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        self.as_ref().encode(buf);
    }
}

impl <T: NbtDecode> NbtDecode for Vec<T> {
    fn decode(buf: &mut Bytes) -> Self {
        let len = VarNum.decode(buf);
        let mut res = Vec::with_capacity(len as usize);
        for _ in 0..len {
            res.push(T::decode(buf));
        }
        res
    }
}

impl <T: NbtEncode> NbtEncode for Vec<T> {
    fn encoded_size(&self) -> usize {
        let len_size: usize = VarNum.encoded_size(&(self.len() as i32));
        let item_size: usize = self.iter().map(|e| e.encoded_size()).sum();
        len_size + item_size
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        VarNum.encode(&(self.len() as i32), buf);
        for item in self.iter() {
            item.encode(buf);
        }
    }
}

impl NbtDecode for Bytes {
    fn decode(buf: &mut Bytes) -> Self {
        let len = VarNum.decode(buf);
        buf.split_to(len as usize)
    }
}

impl NbtDecode for JsonValue {
    fn decode(buf: &mut Bytes) -> Self {
        let s = NbtString::decode(buf);
        json::parse(s.as_ref()).unwrap()
    }
}

impl <T: NbtDecode> NbtDecode for Option<T> {
    fn decode(buf: &mut Bytes) -> Self {
        let exists = bool::decode(buf);
        if exists {
            Some(T::decode(buf))
        } else {
            None
        }
    }
}

impl NbtDecode for Uuid {
    fn decode(buf: &mut Bytes) -> Self {
        let mut bytes = [0u8; 16];
        buf.split_to(16).into_buf().copy_to_slice(&mut bytes);
        Uuid::from_bytes(bytes)
    }
}

impl <T: NbtEncode> NbtEncode for &T {
    fn encoded_size(&self) -> usize {
        (*self).encoded_size()
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        (*self).encode(buf)
    }
}