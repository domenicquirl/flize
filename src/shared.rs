use crate::tag::{read_tag, set_tag, strip, Tag};
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Shared<'shield, V, T>
where
    V: 'shield,
    T: Tag,
{
    pub(crate) data: usize,
    _m0: PhantomData<&'shield ()>,
    _m1: PhantomData<V>,
    _m2: PhantomData<T>,
}

impl<'shield, V, T> Shared<'shield, V, T>
where
    V: 'shield,
    T: Tag,
{
    pub fn null() -> Self {
        unsafe { Self::from_raw(0) }
    }

    /// # Safety
    /// The alignment of `V` must free up sufficient low bits so that `T` fits.
    pub unsafe fn from_ptr(ptr: *mut V) -> Self {
        Self::from_raw(ptr as usize)
    }

    pub unsafe fn from_raw(data: usize) -> Self {
        Self {
            data,
            _m0: PhantomData,
            _m1: PhantomData,
            _m2: PhantomData,
        }
    }

    pub fn into_raw(self) -> usize {
        self.data
    }

    pub fn as_ptr(&self) -> *mut V {
        strip::<T>(self.data) as *mut V
    }

    /// # Safety
    /// - The pointer must either be null or point to a valid instance of `V`.
    /// - You must ensure the instance of `V` is not borrowed mutably.
    pub unsafe fn as_ref(&self) -> Option<&'shield V> {
        self.as_ptr().as_ref()
    }

    /// # Safety
    /// - The pointer must either be null or point to a valid instance of `V`.
    /// - You must ensure the instance of `V` is not borrowed.
    pub unsafe fn as_mut_ref(&mut self) -> Option<&'shield mut V> {
        let ptr = self.as_ptr();

        if !ptr.is_null() {
            Some(&mut *ptr)
        } else {
            None
        }
    }

    /// # Safety
    /// - The pointer must point to a valid instance of `V`.
    /// - You must ensure the instance of `V` is not borrowed mutably.
    pub unsafe fn as_ref_unchecked(&self) -> &'shield V {
        &*self.as_ptr()
    }

    /// # Safety
    /// - The pointer must point to a valid instance of `V`.
    /// - You must ensure the instance of `V` is not borrowed.
    pub unsafe fn as_mut_ref_unchecked(&mut self) -> &'shield mut V {
        &mut *self.as_ptr()
    }

    pub fn is_null(&self) -> bool {
        self.as_ptr().is_null()
    }

    pub fn tag(&self) -> T {
        let bits = read_tag::<T>(self.data);
        Tag::deserialize(bits)
    }

    pub fn with_tag(&self, tag: T) -> Self {
        let bits = tag.serialize();
        let data = set_tag::<T>(self.data, bits);
        unsafe { Self::from_raw(data) }
    }
}
