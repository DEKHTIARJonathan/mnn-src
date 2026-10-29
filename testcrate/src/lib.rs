use mnn_src as _;

#[cfg(test)]
mod tests {
    unsafe extern "C" {
        fn smoke(expected: *const std::ffi::c_char) -> i32;
    }
    #[test]
    fn model_inference_and_lifecycle() {
        let version = std::ffi::CString::new(env!("MNN_EXPECTED_VERSION")).unwrap();
        assert_eq!(unsafe { smoke(version.as_ptr()) }, 0);
    }
}
