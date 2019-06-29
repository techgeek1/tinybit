//! # Tinybit
//! A library inspired by `byteorder` focused on parsing primitive types from binary streams
//! with robust error handling options

#![deny(missing_docs)]

use std::io::{self, Read, Write};
use std::mem;
use std::ptr;
use std::slice;

/// An error incurred when converting between binary representations
pub enum EndianError {
    /// Reached the end of the stream while reading/writing, contains the number of bytes written/read
    EndOfStream(usize)
}

/// A trait providing various generic implementations for serializing primitive copy only types
pub trait Endian: Copy + Default {
    /// Convert self into little endian representation and write the results to `out`
    /// 
    /// # Remarks
    /// Assumes that `out` is large enough to hold `mem::size_of::<Self>()` bytes, truncates otherwise
    /// 
    /// # Returns
    /// Ok - The number of bytes written to `out`
    /// Err - The number of bytes written before reaching the end of `out`
    fn to_le_bytes<W: Write>(&self, buf: &mut W) -> Result<usize, io::Error> {
        // Type is a ZST, can't copy anything but it still "succeeds"
        // (why anyone would ever try this I have no idea)
        let size = mem::size_of::<Self>();
        if size == 0 {
            return Ok(0);
        }

        unsafe {
            // Construct the data
            let mut tmp = *self;
            let src = &mut tmp as *mut _ as *mut u8;
            let slice = slice::from_raw_parts(src, size);

            // Ensure correct encoding
            transform_le_bytes(src, size);

            // Write to the buffer
            Ok(buf.write(slice)?)
        }
    }

    /// Convert self into little endian representation and write the results to `out`
    /// 
    /// # Safety
    /// Assumes that `out` is valid, well aligned, and large enough to hold `mem::size_of::<Self>()` bytes
    /// 
    /// # Returns
    /// Ok - The number of bytes written to `out`
    /// Err - The number of bytes written before reaching the end of `out`
    unsafe fn to_le_bytes_unchecked(&self, out: *mut u8) -> usize {
        // Type is a ZST, can't copy anything but it still "succeeds"
        // (why anyone would ever try this I have no idea)
        let size = mem::size_of::<Self>();
        if size == 0 {
            return 0;
        }

        let src = self as *const _ as *const u8;
        ptr::copy_nonoverlapping(src, out, size);
        transform_le_bytes(out, size);
        
        size
    }

    /// Convert self into big endian representation and write the results to `out`
    /// 
    /// # Remarks
    /// Assumes that `out` is large enough to hold `mem::size_of::<Self>()` bytes, truncates otherwise
    /// 
    /// # Returns
    /// Ok - The number of bytes written to `out`
    /// Err - The number of bytes written before reaching the end of `out`
    fn to_be_bytes<W: Write>(&self, buf: &mut W) -> Result<usize, io::Error> {
        // Type is a ZST, can't copy anything but it still "succeeds"
        // (why anyone would ever try this I have no idea)
        let size = mem::size_of::<Self>();
        if size == 0 {
            return Ok(0);
        }

        unsafe {
            // Construct the data
            let mut tmp = *self;
            let src = &mut tmp as *mut _ as *mut u8;
            let slice = slice::from_raw_parts(src, size);

            // Ensure correct encoding
            transform_be_bytes(src, size);

            // Write to the buffer
            Ok(buf.write(slice)?)
        }
    }

    /// Convert self into big endian representation and write the results to `out`
    /// 
    /// # Safety
    /// Assumes that `out` is valid, well aligned, and large enough to hold `mem::size_of::<Self>()` bytes
    /// 
    /// # Returns
    /// Ok - The number of bytes written to `out`
    /// Err - The number of bytes written before reaching the end of `out`
    unsafe fn to_be_bytes_unchecked(&self, out: *mut u8) -> usize {
        // Type is a ZST, can't copy anything but it still "succeeds"
        // (why anyone would ever try this I have no idea)
        let size = mem::size_of::<Self>();
        if size == 0 {
            return 0;
        }

        let src = self as *const _ as *const u8;
        ptr::copy_nonoverlapping(src, out, size);
        transform_be_bytes(out, size);

        size
    }

    /// Creates self from the little endian bytes in `buf`
    /// 
    /// # Returns
    /// Ok with self
    /// Err when `buf` does not contain enough bytes
    fn from_le_bytes<R: Read>(buf: &mut R) -> Result<Self, io::Error> {
        let size = mem::size_of::<Self>();
        if size == 0 {
            return Ok(Default::default());
        }

        unsafe {
            let mut result = mem::MaybeUninit::zeroed();
            let dst = result.as_mut_ptr() as *mut u8;
            let slice = slice::from_raw_parts_mut(dst, size);

            if buf.read(slice)? < size {
                return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
            }

            transform_le_bytes(dst, size);

            Ok(result.assume_init())
        }
    }

    /// Creates self from the little endian bytes at `src`
    /// 
    /// # Safety
    /// Assumes the following
    /// - `src` is valid and well aligned
    /// - `src` is valid for `mem::size_of::<Self>()` bytes
    unsafe fn from_le_bytes_unchecked(src: *const u8) -> Self {
        let size = mem::size_of::<Self>();
        if size == 0 {
            return mem::zeroed();
        }

        let mut result = mem::MaybeUninit::zeroed();
        let dst = result.as_mut_ptr() as *mut u8;

        ptr::copy_nonoverlapping(src, dst, size);

        transform_le_bytes(dst, size);

        result.assume_init()
    }

    /// Creates self from the big endian bytes in `buf`
    /// 
    /// # Returns
    /// Ok with self
    /// Err when `buf` does not contain enough bytes
    fn from_be_bytes<R: Read>(buf: &mut R) -> Result<Self, io::Error> {
        let size = mem::size_of::<Self>();
        if size == 0 {
            return Ok(unsafe { mem::zeroed() });// ZST optimizes out to a noop
        }

        unsafe {
            let mut result = mem::MaybeUninit::zeroed();
            let dst = result.as_mut_ptr() as *mut u8;
            let slice = slice::from_raw_parts_mut(dst, size);

            if buf.read(slice)? < size {
                return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
            }

            transform_be_bytes(dst, size);

            Ok(result.assume_init())
        }
    }

    /// Creates self from the big endian bytes in `buf`
    /// 
    /// # Safety
    /// Assumes the following
    /// - `src` is valid and well aligned
    /// - `src` is valid for `mem::size_of::<Self>()` bytes
    unsafe fn from_be_bytes_unchecked(src: *const u8) -> Self {
        let size = mem::size_of::<Self>();
        if size == 0 {
            return mem::zeroed();// ZST optimizes out to a noop
        }

        let mut result = mem::MaybeUninit::zeroed();
        let dst = result.as_mut_ptr() as *mut u8;

        ptr::copy_nonoverlapping(src, dst, size);

        transform_be_bytes(dst, size);

        result.assume_init()
    }
}

// Blanket impl to cover all trivial types
impl<T> Endian for T
    where T: Copy + Default
{ }

/// Transform a binary representation into little endian format
/// 
/// Is a noop when the target endianess matches the native endianess
/// 
/// # Safety
/// Assumes the following
/// - `ptr` is valid and well aligned
/// - `ptr` is valid for `len` bytes
#[allow(unused_variables)]
unsafe fn transform_le_bytes(ptr: *mut u8, len: usize) {
    // Little endian systems are a noop so do nothing

    // Big endian systems need to flip the bytes in place to get the little endian representation
    #[cfg(target_endian = "big")] {
        let half_len = len / 2;
        if half_len > 0 {
            for i in 0..half_len {
                ptr::swap(ptr.add(i), ptr.add(len - i - 1));
            }
        }
    }
}

/// Transform a binary representation into big endian format
/// 
/// # Safety
/// Assumes the following
/// - `ptr` is valid and well aligned
/// - `ptr` is valid for `len` bytes
unsafe fn transform_be_bytes(ptr: *mut u8, len: usize) {
    // Big endian systems can just return here as it's already correct
    #[cfg(target_endian = "big")] { 
        return;
    }

    // Little endian systems need to flip the bytes in place to get the big endian representation
    #[cfg(target_endian = "little")] {
        let half_len = len / 2;
        if half_len > 0 {
            for i in 0..half_len {
                ptr::swap(ptr.add(i), ptr.add(len - i - 1));
            }
        }
    }
}