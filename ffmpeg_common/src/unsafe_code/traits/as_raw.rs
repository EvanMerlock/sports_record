use libc;

pub trait AsRawPtr<T> {
    fn as_ptr(&self) -> *const T;
    fn as_mut_ptr(&mut self) -> *mut T;

    fn as_void_ptr(&self) -> *const libc::c_void {
        self.as_ptr() as *const _
    }

    fn as_mut_void_ptr(&mut self) -> *mut libc::c_void {
        self.as_mut_ptr() as *mut _
    }
}