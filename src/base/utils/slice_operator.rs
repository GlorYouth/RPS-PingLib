#![cfg_attr(debug_assertions, allow(dead_code))]

use std::slice::Iter;

#[derive(Debug)]
pub struct SliceOperator<'a> {
    slice: &'a mut [u8],
    pos: usize,
}

impl<'a> From<&'a mut [u8]> for SliceOperator<'a> {
    #[inline]
    fn from(slice: &'a mut [u8]) -> SliceOperator<'a> {
        SliceOperator { slice, pos: 0 }
    }
}

#[allow(unused)]
impl<'a> SliceOperator<'a> {
    #[inline]
    pub fn pos(&self) -> usize {
        self.pos
    }

    #[inline]
    pub fn peek_u8(&self) -> u8 {
        self.slice[self.pos]
    }

    #[inline]
    pub fn peek_u16(&self) -> u16 {
        u16::from_be_bytes([self.slice[self.pos], self.slice[self.pos + 1]])
    }

    #[inline]
    pub fn peek_u32(&self) -> u32 {
        u32::from_be_bytes(self.slice[self.pos..self.pos + 4].try_into().unwrap())
    }

    #[inline]
    pub fn peek_u64(&self) -> u64 {
        u64::from_be_bytes(self.slice[self.pos..self.pos + 8].try_into().unwrap())
    }

    #[inline]
    pub fn read_u8(&mut self) -> u8 {
        self.pos += 1;
        self.slice[self.pos - 1]
    }

    #[inline]
    pub fn read_u16(&mut self) -> u16 {
        self.pos += 2;
        u16::from_be_bytes(self.slice[self.pos - 2..self.pos].try_into().unwrap())
    }

    #[inline]
    pub fn read_u32(&mut self) -> u32 {
        self.pos += 4;
        u32::from_be_bytes(self.slice[self.pos - 4..self.pos].try_into().unwrap())
    }

    #[inline]
    pub fn read_u64(&mut self) -> u64 {
        self.pos += 8;
        u64::from_be_bytes(self.slice[self.pos - 8..self.pos].try_into().unwrap())
    }

    #[inline]
    pub fn write_u8(&mut self, val: u8) {
        self.slice[self.pos] = val;
        self.pos += 1;
    }

    #[inline]
    pub fn write_u16(&mut self, val: u16) {
        self.slice[self.pos..self.pos + 2].copy_from_slice(&val.to_be_bytes());
        self.pos += 2;
    }

    #[inline]
    pub fn write_u32(&mut self, val: u32) {
        self.slice[self.pos..self.pos + 4].copy_from_slice(&val.to_be_bytes());
        self.pos += 4;
    }

    #[inline]
    pub fn write_u64(&mut self, val: u64) {
        self.slice[self.pos..self.pos + 8].copy_from_slice(&val.to_be_bytes());
        self.pos += 8;
    }

    #[inline]
    pub fn write_slice(&mut self, slice: &[u8]) {
        self.slice[self.pos..self.pos + slice.len()].copy_from_slice(slice);
        self.pos += slice.len();
    }

    #[inline]
    pub fn iter_from_current_pos(&self) -> Iter<u8> {
        self.slice[self.pos..].iter()
    }

    #[inline]
    pub fn skip(&mut self, n: usize) {
        self.pos += n;
    }

    #[inline]
    pub fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
    }

    #[inline]
    pub fn as_ref(&self) -> &[u8] {
        self.slice
    }

    #[inline]
    pub fn as_mut(&mut self) -> &mut [u8] {
        self.slice
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.slice.len()
    }

    #[inline]
    pub fn read_slice(&mut self, len: usize) -> &[u8] {
        self.pos += len;
        &self.slice[self.pos - len..self.pos]
    }

    #[inline]
    pub fn from_slice(slice: &'a mut [u8]) -> Self {
        SliceOperator { slice, pos: 0 }
    }
}
