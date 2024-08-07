/// # Power
/// The power is a representation of this primitive data type which when set to the power of 2 gives the size in bytes.
/// The power only has its 2 least significant bits used and the rest are discarded.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    X8,
    X16,
    X32,
    X64
}

impl Size {
    /// Construct this enum from an exponent of the power of 2.
    pub fn from_power(size: u8) -> Self {
        match size & 0b000000_11 {
            0 => Self::X8,
            1 => Self::X16,
            2 => Self::X32,
            3 => Self::X64,
            _ => unreachable!()
        }
    }

    /// Convert this enum representation to a power of 2.
    pub const fn power(self) -> u8 {
        match self {
            Self::X8 => 0,
            Self::X16 => 1,
            Self::X32 => 2,
            Self::X64 => 3
        }
    }

    /// Get the number of bytes it would take to represent a value of this size.
    pub const fn size(self) -> u8 {
        let size = match self {
            Self::X8 => u8::BITS / 8,
            Self::X16 => u16::BITS / 8,
            Self::X32 => u32::BITS / 8,
            Self::X64 => u64::BITS / 8
        };

        size as u8
    }

    /// Attempt to increase the size to the next quantization.
    pub fn upsize(&mut self) {
        *self = match self {
            Self::X8 => Self::X16,
            Self::X16 => Self::X32,
            Self::X32 => Self::X64,
            Self::X64 => Self::X64
        }
    }

    /// Whether if upsizing will return a different value.
    pub const fn can_upsize(self) -> bool {
        !matches!(self, Self::X64)
    }

    /// Attempt to decrease the size to the next quantization. 
    ///
    /// # Result
    /// If this cannot be downsized anymore, then self is returned.
    pub fn downsize(&mut self) {
        *self = match self {
            Self::X64 => Self::X32,
            Self::X32 => Self::X16,
            Self::X16 => Self::X8,
            Self::X8 => Self::X8
        }
    }

    /// Whether if downsizing will return a different value.
    pub const fn can_downsize(self) -> bool {
        !matches!(self, Self::X8)
    }

    pub const fn mask(self) -> u64 {
        match self {
            Self::X8 => u8::MAX as u64,
            Self::X16 => u16::MAX as u64,
            Self::X32 => u32::MAX as u64,
            Self::X64 => u64::MAX
        }
    }

    pub const fn minimum(value: u64) -> Self {
        if value > u32::MAX as u64 { Self::X64 }
        else if value > u16::MAX as u64 { Self::X32 }
        else if value > u8::MAX as u64 { Self::X16 }
        else { Self::X8 }
    }

    pub const fn minimum_signed(value: i64) -> Self {
        if value > i32::MAX as i64 || value < i32::MIN as i64 { Self::X64 }
        else if value > i16::MAX as i64 || value < i16::MIN as i64 { Self::X32 }
        else if value > i8::MAX as i64 || value < i8::MIN as i64 { Self::X16 }
        else { Self::X8 }
    }
}

macro_rules! impl_mask {
    ($ty: ty, $name: tt) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $name<const MASK: $ty>($ty);
        
        impl<const MASK: $ty> $name<MASK> {
            pub const fn new(value: $ty) -> Self { Self(value & MASK) }
            pub const fn get(self) -> $ty { self.0 }
            pub fn set(&mut self, value: $ty) { self.0 = value & MASK }
        }
    };
}

impl_mask!(u8, MaskedU8);
impl_mask!(u16, MaskedU16);
impl_mask!(u32, MaskedU32);
impl_mask!(u64, MaskedU64);
impl_mask!(i8, MaskedI8);
impl_mask!(i16, MaskedI16);
impl_mask!(i32, MaskedI32);
impl_mask!(i64, MaskedI64);