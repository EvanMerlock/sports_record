use std::marker::{Send};
use std::convert::{From};
use std::ops::{Drop, Deref, DerefMut};

use std::ptr;

use ffmpeg_sys::*;
use unsafe_code::{AsRawPtr, Frame, UnsafeError, UnsafeErrorKind};

use unsafe_code::PixelFormat;

pub struct SWSContext(*mut SwsContext, SWSImageDefinition, SWSImageDefinition);

pub struct SWSImageDefinition(pub i32, pub i32, pub PixelFormat);

impl SWSImageDefinition {
    pub fn new<T: Into<PixelFormat>>(h: i32, w: i32, fmt: T) -> SWSImageDefinition {
        SWSImageDefinition(h, w, fmt.into())
    }
}

unsafe impl Send for SWSContext {}

impl SWSContext {
    unsafe fn allocate_sws_context(height: i32, width: i32, in_pix_fmt: PixelFormat, out_pix_fmt: PixelFormat) -> Result<*mut SwsContext, UnsafeError> {
        let cached = sws_getCachedContext(ptr::null_mut(), width, height, *in_pix_fmt, width, height, *out_pix_fmt, SWS_BICUBIC, ptr::null_mut(), ptr::null_mut(), ptr::null());

        if cached.is_null() {
            return Err(UnsafeError::new(UnsafeErrorKind::OpenSWSContext));
        }

        Ok(cached)
    }

    pub fn new<T: Into<PixelFormat> + Copy>(height: i32, width: i32, in_pix_fmt: T, out_pix_fmt: T) -> Result<SWSContext, UnsafeError> {
        unsafe {
            match SWSContext::allocate_sws_context(height, width, in_pix_fmt.into(), out_pix_fmt.into()) {
                Ok(sws) => Ok(SWSContext(sws, SWSImageDefinition::new(height, width, in_pix_fmt.into()), SWSImageDefinition::new(height, width, out_pix_fmt.into()))),
                Err(e) => Err(e),
            }
        }
    }

    unsafe fn scale_using_sws(&mut self, old_frame: &mut Frame, align: i32, pts: i64) -> Result<Frame, UnsafeError> {
        let mut scaled_frame = Frame::new();
        scaled_frame.width = old_frame.width;
        scaled_frame.height = old_frame.height;
        scaled_frame.format = 0;
        scaled_frame.pts = pts;

        let scaled_frame_data_ptr: *mut *mut u8 = scaled_frame.data.as_mut_ptr();
        let scaled_frame_const_ptr: *const *const u8 = scaled_frame_data_ptr as *const *const u8;
        let scaled_frame_linesize_ptr: *mut i32 = scaled_frame.linesize.as_mut_ptr();

        av_image_alloc(scaled_frame_data_ptr, scaled_frame_linesize_ptr, old_frame.width, old_frame.height, *(self.2).2, align);

        let raw_frame_data_ptr: *const *const u8 = old_frame.data.as_ptr() as *const *const u8;
        let raw_frame_linesize_ptr: *mut i32 = old_frame.linesize.as_mut_ptr();

        let _ = sws_scale(self.as_mut_ptr(), raw_frame_data_ptr, raw_frame_linesize_ptr, 0, old_frame.height, scaled_frame_const_ptr, scaled_frame_linesize_ptr);

        Ok(Frame::from(scaled_frame))
    }

    pub fn change_pixel_format(&mut self, old_frame: &mut Frame, align: i32, pts: i64) -> Result<Frame, UnsafeError> {
        unsafe {
            match self.scale_using_sws(old_frame, align, pts) {
                Ok(frame) => Ok(frame),
                Err(e) => Err(e),
            }
        }
    }
}

impl AsRawPtr<SwsContext> for SWSContext {
    fn as_ptr(&self) -> *const SwsContext {
        self.0 as *const _
    }

    fn as_mut_ptr(&mut self) -> *mut SwsContext {
        self.0
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

impl Clone for SWSContext {
    fn clone(&self) -> Self {
        SWSContext::new((self.1).0, (self.1).1, (self.1).2, (self.2).2).unwrap()
    }
}