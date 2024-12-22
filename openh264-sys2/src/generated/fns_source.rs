/* automatically generated by rust-bindgen 0.71.1 */

use super::types::*;

unsafe extern "C" {
    #[doc = " @brief   Create encoder\n  @param   ppEncoder encoder\n  @return  0 - success; otherwise - failed;"]
    pub fn WelsCreateSVCEncoder(ppEncoder: *mut *mut ISVCEncoder) -> ::std::os::raw::c_int;
    #[doc = " @brief   Destroy encoder\n   @param   pEncoder encoder\n  @return  void"]
    pub fn WelsDestroySVCEncoder(pEncoder: *mut ISVCEncoder);
    #[doc = " @brief   Get the capability of decoder\n  @param   pDecCapability  decoder capability\n  @return  0 - success; otherwise - failed;"]
    pub fn WelsGetDecoderCapability(pDecCapability: *mut SDecoderCapability) -> ::std::os::raw::c_int;
    #[doc = " @brief   Create decoder\n  @param   ppDecoder decoder\n  @return  0 - success; otherwise - failed;"]
    pub fn WelsCreateDecoder(ppDecoder: *mut *mut ISVCDecoder) -> ::std::os::raw::c_long;
    #[doc = " @brief   Destroy decoder\n  @param   pDecoder  decoder\n  @return  void"]
    pub fn WelsDestroyDecoder(pDecoder: *mut ISVCDecoder);
    #[doc = " @brief   Get codec version\n           Note, old versions of Mingw (GCC < 4.7) are buggy and use an\n           incorrect/different ABI for calling this function, making it\n           incompatible with MSVC builds.\n  @return  The linked codec version"]
    pub fn WelsGetCodecVersion() -> OpenH264Version;
    #[doc = " @brief   Get codec version\n  @param   pVersion  struct to fill in with the version"]
    pub fn WelsGetCodecVersionEx(pVersion: *mut OpenH264Version);
}
