use std::marker::{Send};
use std::convert::{From};
use std::ops::{Drop, Deref, DerefMut};

use ffmpeg_sys::*;
use unsafe_code::AsRawPtr;

pub struct SWSContext(*mut SwsContext);

unsafe impl Send for SWSContext {}

impl AsRawPtr<SwsContext> for SWSContext {
    fn as_ptr(&self) -> *const SwsContext {
        self.0 as *const _
    }

    fn as_mut_ptr(&mut self) -> *mut SwsContext {
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
        self.as_ref()
    }
}

impl DerefMut for SWSContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    } 
}

impl AsRef<SwsContext> for SWSContext {
    fn as_ref(&self) -> &SwsContext {
        unsafe {
            &*self.0
        }
    }
}

impl AsMut<SwsContext> for SWSContext {
    fn as_mut(&mut self) -> &mut SwsContext {
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

