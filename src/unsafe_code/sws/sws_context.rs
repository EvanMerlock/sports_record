use std::marker::{Send};
use std::convert::{From};
use std::ops::{Drop, Deref, DerefMut};

use ffmpeg_sys::*;

pub struct SWSContext(*mut SwsContext);

unsafe impl Send for SWSContext {}

impl SWSContext {
    pub unsafe fn as_ptr(&self) -> *const SwsContext {
        self.0 as *const _
    }

    pub unsafe fn as_mut_ptr(&mut self) -> *mut SwsContext {
        self.0
    }
}

impl From<*mut SwsContext> for SWSContext {
    fn from(ctx: *mut SwsContext) -> SWSContext {
        SWSContext(ctx)
    }
}

impl Deref for SWSContext {
    type Target = SwsContext;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &*self.0
        }
    }
}

impl DerefMut for SWSContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.0
        }
    } 
}

impl Drop for SWSContext {
	fn drop(&mut self) {
		unsafe {
			sws_freeContext(self.as_mut_ptr());
		}
	}
}

