// ensure a constant
pub(in crate::base) struct UnMut<T>(T);

impl<T> UnMut<T> {
    #[inline(always)]
    pub fn new(x: T) -> UnMut<T> {
        UnMut(x)
    }

    // #[inline(always)]
    // pub fn as_ref(&self) -> &T {
    //     &self.0
    // }

    #[inline(always)]
    pub fn as_const_ref(&self) -> *const T {
        &self.0 as *const _
    }

    // #[inline(always)]
    // pub fn into_inner(self) -> T {
    //     self.0
    // }
}
