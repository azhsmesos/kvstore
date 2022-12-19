use std::marker::PhantomData;
use std::mem;

// 通过atomic 实现cas

const ADDRESS_BITS: usize = 48;

pub struct Owned<T> {
    data: u64,
    _marker: PhantomData<Box<T>>,
}

pub trait Pointer<T> {
    fn into_in_u64(self) -> u64;

    unsafe fn from_in_u64(data: u64) -> Self;
}

impl<T> Pointer<T> for Owned<T> {
    #[inline]
    fn into_in_u64(self) -> u64 {
        let data = self.data;
        mem::forget(self);
        data
    }

    #[inline]
    unsafe fn from_in_u64(data: u64) -> Self {
        debug_assert!(data != 0, "converting zero into `Owned`");
        Owned {
            data,
            _marker: PhantomData,
        }
    }
}

impl <T> Owned<T> {

    pub fn new(data: T) -> Owned<T> {
        unsafe {
            Self::from_in_u64(into_raw_ptr(Box::new(data)) as u64)
        }
    }
}


unsafe fn into_raw_ptr<T>(b: Box<T>) -> usize {
    // 获取key并将其转换成指针
    let key_ptr = Box::into_raw(b);
    // 前 16 位作为标记、后 48 位作为地址的原子指针
    debug_assert!((key_ptr as u64) < (1 << ADDRESS_BITS));

    key_ptr as usize
}


