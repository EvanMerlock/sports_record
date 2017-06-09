
use std::ptr;

use unsafe_code::{UnsafeError, UnsafeErrorKind};
use unsafe_code::sws::SWSContext;

use ffmpeg_sys::*;

unsafe fn allocate_sws_context(height: i32, width: i32, in_pix_fmt: AVPixelFormat, out_pix_fmt: AVPixelFormat) -> Result<*mut SwsContext, UnsafeError> {
    let cached = sws_getCachedContext(ptr::null_mut(), width, height, in_pix_fmt, width, height, out_pix_fmt, SWS_BICUBIC, ptr::null_mut(), ptr::null_mut(), ptr::null());

    if cached.is_null() {
        return Err(UnsafeError::new(UnsafeErrorKind::OpenSWSContext));
    }

    Ok(cached)
}

pub fn create_sws_context(height: i32, width: i32, in_pix_fmt: AVPixelFormat, out_pix_fmt: AVPixelFormat) -> Result<SWSContext, UnsafeError> {
    unsafe {
        match allocate_sws_context(height, width, in_pix_fmt, out_pix_fmt) {
            Ok(sws) => Ok(SWSContext::from(sws)),
            Err(e) => Err(e),
        }
    }
}

unsafe fn scale_using_sws(old_frame: &mut AVFrame, sws_context: &mut SWSContext, align: i32, pts: i64) -> Result<*mut AVFrame, UnsafeError> {
    let scaled_frame = av_frame_alloc();
    (*scaled_frame).width = old_frame.width;
    (*scaled_frame).height = old_frame.height;
    (*scaled_frame).format = 0;
    (*scaled_frame).pts = pts;

    let scaled_frame_data_ptr: *mut *mut u8 = (*scaled_frame).data.as_mut_ptr();
    let scaled_frame_linesize_ptr: *mut i32 = (*scaled_frame).linesize.as_mut_ptr();

    av_image_alloc(scaled_frame_data_ptr, scaled_frame_linesize_ptr, old_frame.width, old_frame.height, AV_PIX_FMT_YUV420P, align);

    let raw_frame_data_ptr: *const *const u8 = old_frame.data.as_ptr() as *const *const u8;
    let raw_frame_linesize_ptr: *mut i32 = old_frame.linesize.as_mut_ptr();

    let _ = sws_scale(sws_context.as_mut_ptr(), raw_frame_data_ptr, raw_frame_linesize_ptr, 0, old_frame.height, scaled_frame_data_ptr, scaled_frame_linesize_ptr);

    Ok(scaled_frame)
}

pub fn change_pixel_format<'a>(old_frame: &mut AVFrame, sws_context: &mut SWSContext, align: i32, pts: i64) -> Result<&'a mut AVFrame, UnsafeError> {
    unsafe {
        match scale_using_sws(old_frame, sws_context, align, pts) {
            Ok(frame) => Ok(&mut *frame),
            Err(e) => Err(e),
        }
    }
}