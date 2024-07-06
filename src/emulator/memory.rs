//! Memory and address utilities.
//! 
//! # Addresses
//! There are 2 types of addresses and there are 2 parts of an address.
//! - Physical. A real address that points to the same address every time in ram.
//! - Virtual. An illusion address which appears to point to one address but in reality could point to an address that 
//!   is completely different.
//! 
//! The 2 parts of an address are the
//! - Page, which is the block of memory. The page is of the left most bits.
//! - Item, which points to an actual byte that is part of a page. This is of the right most bits and near the least 
//!   significant bit.
//! 
//! A virtual page and a physical page both refer to the same thing, but the difference is the type of address. For
//! example, saying a virtual page would be the same as saying the page of the virtual address.
//! This whole page and item system exists to create something called virtual memory. 
//! 
//! # Purpose
//! The purpose of this is so programs can run in virtual memory address space and have a known upper and lower limit of
//! memory and be able to address areas of memory in between those two extremes as the program runs. These areas are 
//! filled in through declaring pages.  
//! 
//! This also allows for programs to use variable amounts of memory by growing or shrinking their usage. More programs 
//! can also use up memory that one program returned back to the system. 
//! 
//! Finally, it can protect memory owned by other processes and the real mode program such as an operating system from 
//! a program running in virtual memory. 
//! 
//! # Translation
//! Virtual addresses are meant to be translated before they can be used by the processor. Translation involves 
//! injecting a different page into the address and then using that new address. The item remains the same.

use std::collections::HashMap;
use std::io;
use std::io::{ErrorKind, Read, Seek, SeekFrom};
use utility::{LastError, ReadAll, write_buffer_into_bytes};
use crate::number;
use crate::number::{BYTE_SIZE, DUAL_SIZE, QUAD_SIZE, Size, WORD_SIZE};
use crate::utility::read_bytes_into_buffer;

// region: Constants
pub const WORD_ALIGNED_MASK   : u64 = 0b1;
pub const DUAL_ALIGNED_MASK   : u64 = 0b11;
pub const QUAD_ALIGNED_MASK   : u64 = 0b111;

pub const PAGE_ITEM_BITS      : u64 = 13;
pub const PAGE_IDENTIFIER_MASK: u64 = u64::MAX << PAGE_ITEM_BITS;
pub const PAGE_ITEM_MASK      : u64 = u64::MAX >> (64 - PAGE_ITEM_BITS);
pub const MAX_PAGES_COUNT     : u64 = u64::MAX & PAGE_IDENTIFIER_MASK;
pub const PAGE_BYTES_COUNT    : u64 = (u64::MAX & PAGE_ITEM_MASK) + 1;
// pub const PAGE_BYTES_COUNT    : u64 = 2u64.pow(PAGE_ITEM_BITS as u32); TODO: Whats the issue? This generates the 
//                                                                        TODO: maximum index, not the count. 
// endregion

/// An address frame which includes a memory address and the frame size.
#[derive(Debug, Clone)]
pub struct Frame {
    pub address: u64,
    pub size: number::Size
}

impl Frame {
    /// Check to see if the current address frame is aligned to memory. Only aligned frames can be used to interact
    /// with memory.
    /// ```
    /// use atln_processor::memory::Frame;
    /// use atln_processor::number::Size;
    ///
    /// // Aligned
    /// assert!(Frame { address: 0, size: Size::Byte }.is_aligned());
    /// assert!(Frame { address: 0, size: Size::Quad }.is_aligned());
    /// assert!(Frame { address: 7, size: Size::Byte }.is_aligned());
    ///
    /// assert!(Frame { address: 8, size: Size::Word }.is_aligned());
    /// assert!(Frame { address: 8, size: Size::Quad }.is_aligned());
    ///
    /// // Not aligned
    /// assert!(!Frame { address: 7, size: Size::Word }.is_aligned());
    /// assert!(!Frame { address: 1, size: Size::Quad }.is_aligned());
    /// ```
    pub fn is_aligned(&self) -> bool {
        let masked = match self.size {
            number::Size::Byte => 0,
            number::Size::Word => self.address & WORD_ALIGNED_MASK,
            number::Size::Dual => self.address & DUAL_ALIGNED_MASK,
            number::Size::Quad => self.address & QUAD_ALIGNED_MASK
        };

        masked == 0
    }

    /// Gets the largest targeted address.
    pub fn max_address(&self) -> u64 {
        self.address + self.size.size() as u64
    }
}

// region: Address utilities
/// A utility containing methods for manipulating and reading addresses and their partitioned segments. The terminology
/// here may be confusing, so refer to this module's documentation.
pub trait Address {
    /// Extract the virtual address item. This would be the right most bits. The number of bits is specified by the
    /// [PAGE_ITEM_BITS] constant. The item is the actual byte address in the page. This points to real data in memory.
    /// This expects that this address is formatted as an address.
    fn extract_item(&self) -> u64;

    /// Translate a virtual address into a physical address. This allows you to add a virtual address item address to a
    /// page identifier code. This simply layers a physical page onto a virtual item address. In other words, this is
    /// setting the item bits of an address.
    fn set_item(&self, r#virtual: u64) -> u64;

    /// Extract the virtual address page identifier code. These are the left most bits and correspond to the page of
    /// memory the byte address lies in. This function is to be used on full addresses with the page encoded in the
    /// correct section.
    fn extract_page(&self) -> u64;

    /// Set the page bits of this address. These are the left most bits.
    fn set_page(&self, page: u64) -> u64;

    /// Offset the page code to have the page identifier bits in the correct segment of the address. This is for numbers
    /// that contain the page code without being partitioned. This function will simply move the bits to the left, so
    /// it can be operated on with or and a virtual address suffix.
    ///
    /// The result is used as a layer, needs to be shifted over to allow for it to layer on an item suffix. This also
    /// behaves as removing the items bits.
    fn offset_page(&self) -> u64;
}

impl Address for u64 {
    /// ```
    /// assert!(false); // TODO: Test
    /// ```
    fn extract_item(&self) -> u64 {
        PAGE_ITEM_MASK & self
    }

    /// ```
    /// use atln_processor::memory::Address;
    ///
    /// // TODO: Exhaustive testing potentially required.
    /// assert_eq!(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000.set_item(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_11111111), 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_11111111);
    /// assert_eq!(0b00000000_00000000_00000000_00000000_00000000_00000000_00100000_00000000.set_item(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000011), 0b00000000_00000000_00000000_00000000_00000000_00000000_00100000_00000011);
    /// assert_eq!(0b11111111_00000000_00000000_00000000_00000000_00000000_00000000_00000000.set_item(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00001010), 0b11111111_00000000_00000000_00000000_00000000_00000000_00000000_00001010);
    /// ```
    fn set_item(&self, r#virtual: u64) -> u64 {
        let page_item = r#virtual & PAGE_ITEM_MASK;
        (self & PAGE_IDENTIFIER_MASK) | page_item
    }

    /// ```
    /// assert!(false); // TODO: Test
    /// ```
    fn extract_page(&self) -> u64 {
        (PAGE_IDENTIFIER_MASK & self) >> PAGE_ITEM_BITS
    }

    /// ```
    /// assert!(false); // TODO: Test
    /// ```
    fn set_page(&self, _page: u64) -> u64 {
        todo!()
    }

    /// ```
    /// // TODO
    /// ```
    fn offset_page(&self) -> u64 {
        self << PAGE_ITEM_BITS
    }
}
// endregion

/// Memory addressing must be aligned, and rules must be followed for frame oriented operations on memory.
/// - If the memory is size constrained, then ensure the frame is not reaching past the memory size limit.
/// - Frames must be aligned to simulate hardware limitations of an implemented memory module.
#[derive(Debug, Clone)]
pub struct Memory<T: AsRef<[u8]> + AsMut<[u8]>> {
    pub bytes: T,
    pub max_address: Option<u64>,
    /// Mappings of virtual page addresses to physical page addresses.
    pub pages: HashMap<u64, u64>
}

/// Error caused from setting data in memory.
pub enum SetError {
    /// Error from using an unaligned address frame.
    UnalignedFrame
}

/// Caused by invalid parameters to initialize an address frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GetError {
    /// The memory address requested data that is sized outside the memory aligned divisions.
    UnalignedFrame,
    /// The address frame crosses the positive memory boundaries.
    OutOfBounds,
    /// Virtual memory context was in use but the remapping did not exist in the page list.
    PageFault
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> Memory<T> {
    /// Translate the virtual address into a physical address based on the current situation. This returns a unit if the
    /// page mapping does not exist. This is a page fault.
    /// If the page does not exist then that case is a page fault. This function would return [None] to imply a page
    /// fault.
    /// ```
    /// use std::collections::HashMap;
    /// use atln_processor::memory::{Memory};
    ///
    /// let mut memory = Memory::from(Vec::new());
    /// memory.pages = HashMap::from([
    ///     // Pages that are next to each other.
    ///     (10, 200),
    ///     (9, 199),
    ///     (8, 198)
    /// ]);
    ///
    /// // Test multiple mappings.
    /// assert_eq!(memory.translate_virtual(0b000_00000000_00000000_00000000_00000000_00000000_00001010__00000_00001010).unwrap(), 0b000_00000000_00000000_00000000_00000000_00000000_11001000__00000_00001010);
    /// assert_eq!(memory.translate_virtual(0b000_00000000_00000000_00000000_00000000_00000000_00001001__00000_00001010).unwrap(), 0b000_00000000_00000000_00000000_00000000_00000000_11000111__00000_00001010);
    /// assert_eq!(memory.translate_virtual(0b000_00000000_00000000_00000000_00000000_00000000_00001000__00000_00001010).unwrap(), 0b000_00000000_00000000_00000000_00000000_00000000_11000110__00000_00001010);
    ///
    /// // Unmapped page. This is a page fault situation.
    /// assert!(matches!(memory.translate_virtual(0b000_00000000_00000000_00000000_00000000_00000000_00000000__00000_00001010), None));
    /// ```
    pub fn translate_virtual(&self, r#virtual: u64) -> Option<u64> {
        let virtual_page = r#virtual.extract_page();
        // Find the mapping based on the virtual page.
        let physical_page = self.pages.get(&virtual_page)?.offset_page();
        let virtual_item = r#virtual.extract_item();

        Some(physical_page.set_item(virtual_item))
    }
    
    pub fn new(bytes: T) -> Self {
        Self {
            bytes,
            pages: Default::default(),
            max_address: None
        }
    }

    /// Utility function to check for errors in an address frame when performing operations on memory and to handle
    /// translating frame addresses.
    ///
    /// If the frame is marked as virtual through the [r#virtual] parameter, then the frame will have its address
    /// translated. This also tests for the following errors:
    /// - If the address is unaligned, then [Err(GetError::UnalignedFrame)] is returned.
    /// - Otherwise, if a page fault occurred, then [Err(GetError::PageFault)] is returned.
    /// - Finally, if the address is out of bounds, then [Err(GetError::OutOfBounds)] is returned.
    /// ```
    /// assert!(false); // TODO: Test
    /// ```
    fn process_test_frame(&self, frame: &mut Frame, translate: bool) -> Result<(), GetError> {
        // Ensure the frame is aligned to emulate hardware limitations.
        if !frame.is_aligned() { return Err(GetError::UnalignedFrame) }

        if translate {
            frame.address = match self.translate_virtual(frame.address) {
                Some(value) => value,
                None => return Err(GetError::PageFault)
            };
        }

        // Make sure the frame bounds lies in the memory size range.
        if let Some(max_address) = self.max_address { if frame.max_address() > max_address { return Err(GetError::OutOfBounds) }}

        Ok(())
    }

    /// Read and return the data targeted by the frame with safeguards and emulated hardware limitations. If the page
    /// is not cached in this list, then a [GetError::PageFault] is caused.
    /// ```
    /// use std::collections::HashMap;
    /// use atln_processor::memory::{Frame, Memory, PAGE_BYTES_COUNT, PAGE_ITEM_BITS};
    /// use atln_processor::number::{Data, Size};
    ///
    /// // region: Basic non virtual addressing.
    /// let mut memory = Memory::from(Vec::from([ 0, 0, 0, 0 ]));
    /// assert_eq!(memory.get(Frame { address: 0, size: Size::Dual }, false).unwrap(), Data::Dual(0));
    ///
    /// let mut memory = Memory::from(Vec::from([ 255, 255, 255, 255, 0, 0, 0, 0 ]));
    /// assert_eq!(memory.get(Frame { address: 0, size: Size::Quad }, false).unwrap(), Data::Quad(u32::MAX as u64));
    ///
    /// let mut memory = Memory::from(Vec::from(1001u64.to_le_bytes()));
    /// assert_eq!(memory.get(Frame { address: 0, size: Size::Quad }, false).unwrap(), Data::Quad(1001));
    /// assert_eq!(memory.get(Frame { address: 1, size: Size::Byte }, false).unwrap(), Data::Byte(3));
    /// // endregion
    /// 
    /// // region: Test virtual memory. This is very address specific and everything must work perfectly.
    /// let mut memory = Memory::from({
    ///     let mut store = vec![0u8; (PAGE_BYTES_COUNT * 2) as usize];
    ///   
    ///     // Memory addresses are zero indexed.
    ///     let second_page_index = PAGE_BYTES_COUNT as usize;
    ///
    ///     store[second_page_index] = 255;
    ///     store[second_page_index + 1] = 1;
    ///
    ///     // To account for memory alignment.
    ///     store[second_page_index + 5] = 1;
    ///     store[second_page_index + 6] = 255;
    ///     
    ///     store
    /// });
    ///
    /// // Map addresses from first virtual page boundary to the second hardware page. Hardware and virtual pages align 
    /// // parallel.
    /// memory.pages.insert(0, 1);
    ///
    /// // Test.
    /// assert_eq!(memory.get(Frame { address: 0, size: Size::Byte }, true).unwrap(), Data::Byte(255));
    /// assert_eq!(memory.get(Frame { address: 0, size: Size::Word }, true).unwrap(), Data::Word(511));
    /// assert_eq!(memory.get(Frame { address: 4, size: Size::Word }, true).unwrap(), Data::Word(256));
    /// // endregion
    /// ```
    pub fn get(&self, mut frame: Frame, r#virtual: bool) -> Result<number::Number, GetError> {
        self.process_test_frame(&mut frame, r#virtual)?;
        let mut max_buffer = [0u8; QUAD_SIZE];

        Ok(match frame.size {
            Size::Byte => {
                let buffer = &mut max_buffer[0..BYTE_SIZE];
                if read_bytes_into_buffer(&self.bytes, frame.address as usize, buffer) != buffer.len() { return Err(GetError::OutOfBounds) }
                number::Number::Byte(buffer[0])
            },
            Size::Word => {
                let buffer = &mut max_buffer[0..WORD_SIZE];
                if read_bytes_into_buffer(&self.bytes, frame.address as usize, buffer) != buffer.len() { return Err(GetError::OutOfBounds) }
                number::Number::Word(u16::from_le_bytes([ buffer[0], buffer[1] ]))
            },
            Size::Dual => {
                let buffer = &mut max_buffer[0..DUAL_SIZE];
                if read_bytes_into_buffer(&self.bytes, frame.address as usize, buffer) != buffer.len() { return Err(GetError::OutOfBounds) }
                number::Number::Dual(u32::from_le_bytes([ buffer[0], buffer[1], buffer[2], buffer[3] ]))
            },
            Size::Quad => {
                let buffer = &mut max_buffer[0..QUAD_SIZE];
                if read_bytes_into_buffer(&self.bytes, frame.address as usize, buffer) != buffer.len() { return Err(GetError::OutOfBounds) }
                number::Number::Quad(u64::from_le_bytes([ buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7] ]))
            }
        })
    }
    
    pub fn set(&mut self, mut frame: Frame, r#virtual: bool, value: number::Number) -> Result<(), GetError> {
        self.process_test_frame(&mut frame, r#virtual)?;
        let max_buffer = value.quad_buffer();
        let buffer = frame.size.buffer(&max_buffer);
        
        match frame.size {
            Size::Byte => *self.bytes.as_mut().get_mut(frame.address as usize).ok_or(GetError::OutOfBounds)? = u8::from(&value),
            _ => if write_buffer_into_bytes(&mut self.bytes, frame.address as usize, buffer) != buffer.len() { return Err(GetError::OutOfBounds) }
        }
        
        Ok(())
    }
}