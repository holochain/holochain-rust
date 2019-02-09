#![doc(
    html_logo_url = "https://raw.githubusercontent.com/maidsafe/QA/master/Images/maidsafe_logo.png",
    html_favicon_url = "https://maidsafe.net/img/favicon.ico",
    test(attr(forbid(warnings)))
)]
#![forbid(
    exceeding_bitshifts,
    mutable_transmutes,
    no_mangle_const_items,
    unknown_crate_types,
    warnings
)]
#![deny(
    deprecated,
    improper_ctypes,
    non_shorthand_field_patterns,
    overflowing_literals,
    plugin_as_library,
    stable_features,
    unconditional_recursion,
    unknown_lints,
    unused,
    unused_allocation,
    unused_attributes,
    unused_comparisons,
    unused_features,
    unused_parens,
    while_true
)]
#![warn(
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]
// Allow `trivial_casts` to cast `u8` to `c_char`, which is `u8` or `i8`, depending on the
// architecture.
#![allow(
    bad_style,
    box_pointers,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    non_upper_case_globals,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    variant_size_differences
)]
#![allow(clippy::decimal_literal_representation, clippy::unreadable_literal)]

#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate rand;
#[macro_use]
extern crate unwrap;

// Bindgen generated file.  Generated using the following commands:
// ```
// REGEX="crypto.*|randombytes.*|sodium.*|SODIUM.*"
//
// bindgen sodium.h -o bindgen.rs --ctypes-prefix=libc --distrust-clang-mangling \
// --no-rustfmt-bindings --generate=functions,types,vars --whitelist-function=$REGEX \
// --whitelist-type=$REGEX --whitelist-var=$REGEX
//
// sed -ie 's/&'\''static \[u8; [0-9]*usize\] = \(b".*\\0"\)/*const libc::c_char = (\1 as *const \
// libc::c_uchar) as *const libc::c_char/g' bindgen.rs
// ```
//
// Further manual adjustments are usually needed when upgrading the libsodium version to accommodate
// for e.g.
//   * deprecated libsodium items
//   * https://github.com/rust-lang-nursery/rust-bindgen/issues/511 generating incorrect rust code
//   * applying #[repr(align(...))]
// However, these should show up when running the systest, which should also be reviewed and updated
// when upgrading the libsodium version.
mod bindgen;
mod seeded_rng;

pub use crate::bindgen::*;
pub use crate::seeded_rng::init_with_rng;

#[cfg(test)]
mod tests {
    use super::*;
    use libc::*;

    #[test]
    fn generichash_statebytes() {
        assert!(unsafe { crypto_generichash_statebytes() } > 0);
    }

    #[test]
    fn generichash() {
        let mut out = [0u8; crypto_generichash_BYTES as usize];
        let m = [0u8; 64];
        let key = [0u8; crypto_generichash_KEYBYTES as usize];

        assert_eq!(
            unsafe {
                crypto_generichash(
                    out.as_mut_ptr(),
                    out.len(),
                    m.as_ptr(),
                    m.len() as u64,
                    key.as_ptr(),
                    key.len(),
                )
            },
            0
        );
    }

    #[test]
    fn generichash_multipart() {
        let mut out = [0u8; crypto_generichash_BYTES as usize];
        let m = [0u8; 64];
        let key = [0u8; crypto_generichash_KEYBYTES as usize];
        let mut st = crypto_generichash_state::default();

        assert_eq!(
            unsafe { crypto_generichash_init(&mut st, key.as_ptr(), key.len(), out.len()) },
            0
        );

        assert_eq!(
            unsafe { crypto_generichash_update(&mut st, m.as_ptr(), m.len() as u64) },
            0
        );

        assert_eq!(
            unsafe { crypto_generichash_update(&mut st, m.as_ptr(), m.len() as u64) },
            0
        );

        assert_eq!(
            unsafe { crypto_generichash_final(&mut st, out.as_mut_ptr(), out.len()) },
            0
        );
    }

    #[test]
    fn generichash_blake2b() {
        let mut out = [0u8; crypto_generichash_blake2b_BYTES as usize];
        let m = [0u8; 64];
        let key = [0u8; crypto_generichash_blake2b_KEYBYTES as usize];

        assert_eq!(
            unsafe {
                crypto_generichash_blake2b(
                    out.as_mut_ptr(),
                    out.len(),
                    m.as_ptr(),
                    m.len() as u64,
                    key.as_ptr(),
                    key.len(),
                )
            },
            0
        );
    }

    #[test]
    fn generichash_blake2b_salt_personal() {
        let mut out = [0u8; crypto_generichash_blake2b_BYTES as usize];
        let m = [0u8; 64];
        let key = [0u8; crypto_generichash_blake2b_KEYBYTES as usize];
        let salt = [0u8; crypto_generichash_blake2b_SALTBYTES as usize];
        let personal = [0u8; crypto_generichash_blake2b_PERSONALBYTES as usize];

        assert_eq!(
            unsafe {
                crypto_generichash_blake2b_salt_personal(
                    out.as_mut_ptr(),
                    out.len(),
                    m.as_ptr(),
                    m.len() as u64,
                    key.as_ptr(),
                    key.len(),
                    salt.as_ptr(),
                    personal.as_ptr(),
                )
            },
            0
        );
    }

    #[test]
    fn pwhash_scryptsalsa208sha256_str() {
        let password = "Correct Horse Battery Staple";
        let mut hashed_password = [0; crypto_pwhash_scryptsalsa208sha256_STRBYTES as usize];
        let ret_hash = unsafe {
            crypto_pwhash_scryptsalsa208sha256_str(
                hashed_password.as_mut_ptr(),
                password.as_ptr() as *const c_char,
                password.len() as c_ulonglong,
                c_ulonglong::from(crypto_pwhash_scryptsalsa208sha256_OPSLIMIT_INTERACTIVE),
                crypto_pwhash_scryptsalsa208sha256_MEMLIMIT_INTERACTIVE as size_t,
            )
        };
        assert!(ret_hash == 0);
        let ret_verify = unsafe {
            crypto_pwhash_scryptsalsa208sha256_str_verify(
                hashed_password.as_ptr(),
                password.as_ptr() as *const c_char,
                password.len() as c_ulonglong,
            )
        };
        assert!(ret_verify == 0);
    }

    #[test]
    #[rustfmt::skip]
    fn pwhash_scryptsalsa208sha256_ll_1() {
        // See https://www.tarsnap.com/scrypt/scrypt.pdf Page 16
        let password = "";
        let salt = "";
        let n = 16;
        let r = 1;
        let p = 1;
        let mut buf = [0u8; 64];
        let expected = [0x77, 0xd6, 0x57, 0x62, 0x38, 0x65, 0x7b, 0x20, 0x3b, 0x19, 0xca, 0x42,
            0xc1, 0x8a, 0x04, 0x97, 0xf1, 0x6b, 0x48, 0x44, 0xe3, 0x07, 0x4a, 0xe8, 0xdf, 0xdf,
            0xfa, 0x3f, 0xed, 0xe2, 0x14, 0x42, 0xfc, 0xd0, 0x06, 0x9d, 0xed, 0x09, 0x48, 0xf8,
            0x32, 0x6a, 0x75, 0x3a, 0x0f, 0xc8, 0x1f, 0x17, 0xe8, 0xd3, 0xe0, 0xfb, 0x2e, 0x0d,
            0x36, 0x28, 0xcf, 0x35, 0xe2, 0x0c, 0x38, 0xd1, 0x89, 0x06, ];
        let ret = unsafe {
            crypto_pwhash_scryptsalsa208sha256_ll(
                password.as_ptr(),
                password.len() as size_t,
                salt.as_ptr(),
                salt.len() as size_t,
                n,
                r,
                p,
                buf.as_mut_ptr(),
                buf.len() as size_t,
            )
        };
        assert!(ret == 0);
        assert!(buf[0..] == expected[0..]);
    }

    #[test]
    #[rustfmt::skip]
    fn pwhash_scryptsalsa208sha256_ll_2() {
        // See https://www.tarsnap.com/scrypt/scrypt.pdf Page 16
        let password = "password";
        let salt = "NaCl";
        let n = 1024;
        let r = 8;
        let p = 16;
        let mut buf = [0u8; 64];
        let expected = [0xfd, 0xba, 0xbe, 0x1c, 0x9d, 0x34, 0x72, 0x00, 0x78, 0x56, 0xe7, 0x19,
            0x0d, 0x01, 0xe9, 0xfe, 0x7c, 0x6a, 0xd7, 0xcb, 0xc8, 0x23, 0x78, 0x30, 0xe7, 0x73,
            0x76, 0x63, 0x4b, 0x37, 0x31, 0x62, 0x2e, 0xaf, 0x30, 0xd9, 0x2e, 0x22, 0xa3, 0x88,
            0x6f, 0xf1, 0x09, 0x27, 0x9d, 0x98, 0x30, 0xda, 0xc7, 0x27, 0xaf, 0xb9, 0x4a, 0x83,
            0xee, 0x6d, 0x83, 0x60, 0xcb, 0xdf, 0xa2, 0xcc, 0x06, 0x40, ];
        let ret = unsafe {
            crypto_pwhash_scryptsalsa208sha256_ll(
                password.as_ptr(),
                password.len() as size_t,
                salt.as_ptr(),
                salt.len() as size_t,
                n,
                r,
                p,
                buf.as_mut_ptr(),
                buf.len() as size_t,
            )
        };
        assert!(ret == 0);
        assert!(buf[0..] == expected[0..]);
    }

    #[test]
    #[rustfmt::skip]
    fn pwhash_scryptsalsa208sha256_ll_3() {
        // See https://www.tarsnap.com/scrypt/scrypt.pdf Page 16
        let password = "pleaseletmein";
        let salt = "SodiumChloride";
        let n = 16_384;
        let r = 8;
        let p = 1;
        let mut buf = [0u8; 64];
        let expected = [0x70, 0x23, 0xbd, 0xcb, 0x3a, 0xfd, 0x73, 0x48, 0x46, 0x1c, 0x06, 0xcd,
            0x81, 0xfd, 0x38, 0xeb, 0xfd, 0xa8, 0xfb, 0xba, 0x90, 0x4f, 0x8e, 0x3e, 0xa9, 0xb5,
            0x43, 0xf6, 0x54, 0x5d, 0xa1, 0xf2, 0xd5, 0x43, 0x29, 0x55, 0x61, 0x3f, 0x0f, 0xcf,
            0x62, 0xd4, 0x97, 0x05, 0x24, 0x2a, 0x9a, 0xf9, 0xe6, 0x1e, 0x85, 0xdc, 0x0d, 0x65,
            0x1e, 0x40, 0xdf, 0xcf, 0x01, 0x7b, 0x45, 0x57, 0x58, 0x87, ];
        let ret = unsafe {
            crypto_pwhash_scryptsalsa208sha256_ll(
                password.as_ptr(),
                password.len() as size_t,
                salt.as_ptr(),
                salt.len() as size_t,
                n,
                r,
                p,
                buf.as_mut_ptr(),
                buf.len() as size_t,
            )
        };
        assert!(ret == 0);
        assert!(buf[0..] == expected[0..]);
    }

    #[test]
    #[rustfmt::skip]
    fn pwhash_scryptsalsa208sha256_ll_4() {
        // See https://www.tarsnap.com/scrypt/scrypt.pdf Page 16
        let password = "pleaseletmein";
        let salt = "SodiumChloride";
        let n = 1_048_576;
        let r = 8;
        let p = 1;
        let mut buf = [0u8; 64];
        let expected = [0x21, 0x01, 0xcb, 0x9b, 0x6a, 0x51, 0x1a, 0xae, 0xad, 0xdb, 0xbe, 0x09,
            0xcf, 0x70, 0xf8, 0x81, 0xec, 0x56, 0x8d, 0x57, 0x4a, 0x2f, 0xfd, 0x4d, 0xab, 0xe5,
            0xee, 0x98, 0x20, 0xad, 0xaa, 0x47, 0x8e, 0x56, 0xfd, 0x8f, 0x4b, 0xa5, 0xd0, 0x9f,
            0xfa, 0x1c, 0x6d, 0x92, 0x7c, 0x40, 0xf4, 0xc3, 0x37, 0x30, 0x40, 0x49, 0xe8, 0xa9,
            0x52, 0xfb, 0xcb, 0xf4, 0x5c, 0x6f, 0xa7, 0x7a, 0x41, 0xa4, ];
        let ret = unsafe {
            crypto_pwhash_scryptsalsa208sha256_ll(
                password.as_ptr(),
                password.len() as size_t,
                salt.as_ptr(),
                salt.len() as size_t,
                n,
                r,
                p,
                buf.as_mut_ptr(),
                buf.len() as size_t,
            )
        };
        assert!(ret == 0);
        assert!(buf[0..] == expected[0..]);
    }
}
