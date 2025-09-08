/*
 * This file is part of Steam-Art-Manager which is licensed under GNU Lesser General Public License v2.1
 * See file LICENSE or go to https://www.gnu.org/licenses/old-licenses/lgpl-2.1.en.html for full license details
 */

trait HasByteConvert {
    fn from_le_bytes(bytes: &[u8], offset: usize) -> Self;
    fn from_be_bytes(bytes: &[u8], offset: usize) -> Self;
}

impl HasByteConvert for u8 {
    fn from_le_bytes(bytes: &[u8], offset: usize) -> u8 {
        return bytes[offset];
    }
    fn from_be_bytes(bytes: &[u8], offset: usize) -> u8 {
        return bytes[offset];
    }
}

impl HasByteConvert for u16 {
    fn from_le_bytes(bytes: &[u8], offset: usize) -> u16 {
        return u16::from_le_bytes(
            bytes[offset..offset + 2]
                .try_into()
                .expect("incorrect length"),
        );
    }
    fn from_be_bytes(bytes: &[u8], offset: usize) -> u16 {
        return u16::from_be_bytes(
            bytes[offset..offset + 2]
                .try_into()
                .expect("incorrect length"),
        );
    }
}

impl HasByteConvert for u32 {
    fn from_le_bytes(bytes: &[u8], offset: usize) -> u32 {
        return u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("incorrect length"),
        );
    }
    fn from_be_bytes(bytes: &[u8], offset: usize) -> u32 {
        return u32::from_be_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("incorrect length"),
        );
    }
}

impl HasByteConvert for u64 {
    fn from_le_bytes(bytes: &[u8], offset: usize) -> u64 {
        return u64::from_le_bytes(
            bytes[offset..offset + 8]
                .try_into()
                .expect("incorrect length"),
        );
    }
    fn from_be_bytes(bytes: &[u8], offset: usize) -> u64 {
        return u64::from_be_bytes(
            bytes[offset..offset + 8]
                .try_into()
                .expect("incorrect length"),
        );
    }
}

impl HasByteConvert for i8 {
    fn from_le_bytes(bytes: &[u8], offset: usize) -> i8 {
        return i8::from_le_bytes(
            bytes[offset..offset + 1]
                .try_into()
                .expect("incorrect length"),
        );
    }
    fn from_be_bytes(bytes: &[u8], offset: usize) -> i8 {
        return i8::from_be_bytes(
            bytes[offset..offset + 1]
                .try_into()
                .expect("incorrect length"),
        );
    }
}

impl HasByteConvert for i16 {
    fn from_le_bytes(bytes: &[u8], offset: usize) -> i16 {
        return i16::from_le_bytes(
            bytes[offset..offset + 2]
                .try_into()
                .expect("incorrect length"),
        );
    }
    fn from_be_bytes(bytes: &[u8], offset: usize) -> i16 {
        return i16::from_be_bytes(
            bytes[offset..offset + 2]
                .try_into()
                .expect("incorrect length"),
        );
    }
}

impl HasByteConvert for i32 {
    fn from_le_bytes(bytes: &[u8], offset: usize) -> i32 {
        return i32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("incorrect length"),
        );
    }
    fn from_be_bytes(bytes: &[u8], offset: usize) -> i32 {
        return i32::from_be_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("incorrect length"),
        );
    }
}

impl HasByteConvert for i64 {
    fn from_le_bytes(bytes: &[u8], offset: usize) -> i64 {
        return i64::from_le_bytes(
            bytes[offset..offset + 8]
                .try_into()
                .expect("incorrect length"),
        );
    }
    fn from_be_bytes(bytes: &[u8], offset: usize) -> i64 {
        return i64::from_be_bytes(
            bytes[offset..offset + 8]
                .try_into()
                .expect("incorrect length"),
        );
    }
}

impl HasByteConvert for f32 {
    fn from_le_bytes(bytes: &[u8], offset: usize) -> f32 {
        return f32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("incorrect length"),
        );
    }
    fn from_be_bytes(bytes: &[u8], offset: usize) -> f32 {
        return f32::from_be_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("incorrect length"),
        );
    }
}

impl HasByteConvert for f64 {
    fn from_le_bytes(bytes: &[u8], offset: usize) -> f64 {
        return f64::from_le_bytes(
            bytes[offset..offset + 8]
                .try_into()
                .expect("incorrect length"),
        );
    }
    fn from_be_bytes(bytes: &[u8], offset: usize) -> f64 {
        return f64::from_be_bytes(
            bytes[offset..offset + 8]
                .try_into()
                .expect("incorrect length"),
        );
    }
}

pub struct Reader<'a> {
    data: &'a [u8],
    offset: usize,
    length: u64,
}

#[allow(dead_code)]
impl Reader<'_> {
    /// Gets the underlying data of the reader.
    pub fn get_data(&self) -> &[u8] {
        return self.data;
    }
    /// Gets the offset of the reader.
    pub fn get_offset(&self) -> usize {
        return self.offset;
    }
    /// Gets the length of the reader.
    pub fn get_length(&self) -> u64 {
        return self.length;
    }

    /// Creates a new Reader from the provided buffer.
    pub fn new(buf: &[u8]) -> Reader<'_> {
        return Reader {
            data: buf,
            offset: 0,
            length: buf.len() as u64,
        };
    }

    /// Slices the Reader's buffer and returns a new Reader for the slice.
    pub fn slice(&self, offset: usize, length: usize) -> Reader<'_> {
        let sliced = &self.data[offset..(offset + length)];
        return Reader {
            data: sliced,
            offset: 0,
            length: length as u64,
        };
    }

    /// Seek to a new offset, from 0 (start), 1 (current), or 2 (end) of the buffer.
    pub fn seek(&mut self, offset: usize, position: u8) {
        if position == 0 {
            self.offset = offset;
        } else if position == 1 {
            self.offset += offset;
        } else {
            self.offset = (self.length as usize) - offset;
        }
    }

    /// Gets the remaining length of the buffer.
    pub fn remaining(&mut self) -> u64 {
        return self.length - (self.offset as u64);
    }

    /// Data reading interface.
    fn read_i<T: HasByteConvert>(&mut self, endianness: bool) -> T {
        if endianness {
            return T::from_le_bytes(self.data, self.offset);
        } else {
            return T::from_be_bytes(self.data, self.offset);
        }
    }

    /// Reads the next char from the buffer.
    pub fn read_char(&mut self, endianness: bool) -> char {
        return self.read_uint8(endianness) as char;
    }

    /// Reads the next 8 bit unsigned int from the buffer.
    pub fn read_uint8(&mut self, endianness: bool) -> u8 {
        let res = self.read_i::<u8>(endianness);
        self.offset += 1;
        return res;
    }
    /// Reads the next 16 bit unsigned int from the buffer.
    pub fn read_uint16(&mut self, endianness: bool) -> u16 {
        let res = self.read_i::<u16>(endianness);
        self.offset += 2;
        return res;
    }
    /// Reads the next 32 bit unsigned int from the buffer.
    pub fn read_uint32(&mut self, endianness: bool) -> u32 {
        let res = self.read_i::<u32>(endianness);
        self.offset += 4;
        return res;
    }
    /// Reads the next 64 bit unsigned int from the buffer.
    pub fn read_uint64(&mut self, endianness: bool) -> u64 {
        let res = self.read_i::<u64>(endianness);
        self.offset += 8;
        return res;
    }

    /// Reads the next 8 bit signed int from the buffer.
    pub fn read_int8(&mut self, endianness: bool) -> i8 {
        let res = self.read_i::<i8>(endianness);
        self.offset += 1;
        return res;
    }
    /// Reads the next 16 bit signed int from the buffer.
    pub fn read_int16(&mut self, endianness: bool) -> i16 {
        let res = self.read_i::<i16>(endianness);
        self.offset += 2;
        return res;
    }
    /// Reads the next 32 bit signed int from the buffer.
    pub fn read_int32(&mut self, endianness: bool) -> i32 {
        let res = self.read_i::<i32>(endianness);
        self.offset += 4;
        return res;
    }
    /// Reads the next 64 bit signed int from the buffer.
    pub fn read_int64(&mut self, endianness: bool) -> i64 {
        let res = self.read_i::<i64>(endianness);
        self.offset += 8;
        return res;
    }

    /// Reads the next 32 bit float from the buffer.
    pub fn read_float32(&mut self, endianness: bool) -> f32 {
        let res = self.read_i::<f32>(endianness);
        self.offset += 4;
        return res;
    }
    /// Reads the next 64 bit float from the buffer.
    pub fn read_float64(&mut self, endianness: bool) -> f64 {
        let res = self.read_i::<f64>(endianness);
        self.offset += 8;
        return res;
    }

    /// Reads the next string from the buffer, using the provided length or reading till next 00 byte.
    pub fn read_string(&mut self, length: Option<u32>) -> String {
        let mut len: usize = 0;

        if length.is_some() {
            len = length.unwrap() as usize;
        } else {
            loop {
                if self.data[self.offset + len] == 0 {
                    break;
                } else {
                    len += 1;
                }
            }
        }

        let u8_vec = self.data[self.offset..self.offset + len].to_vec();

        let utf8_res = String::from_utf8(u8_vec.clone());
        self.offset += len + 1;

        if utf8_res.is_ok() {
            return utf8_res.unwrap();
        } else {
            let u16_vec: Vec<u16> = u8_vec
                .iter()
                .map(|char_code| {
                    return char_code.to_owned() as u16;
                })
                .collect();
            let char_codes = &u16_vec[..];

            return String::from_utf16(char_codes).unwrap();
        }
    }
}
