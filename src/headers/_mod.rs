//! C headers generation.
//!
//! This module is only enabled when the `"headers"` feature of `::safer_ffi` is
//! enabled, which is expected to be done through a cargo feature within the
//! (downstream) crate defining the `#[ffi_export]`ed
//! functions.
//!
//! ```toml
//! [dependencies]
//! safer-ffi = { version = "...", features = ["proc_macros"] }
//!
//! [features]
//! generate-headers = ["safer-ffi/headers"]
//! ```
//!
//! Then, to generate the bindings, just define a
//! `#[safer_ffi::cfg_headers]`-gated `#[test]` function,
//! which can then call the [`builder`] to do the work:
//!
//! ```rust
//! use ::std::{io, fs};
//! use ::safer_ffi::prelude::*;
//!
//! /// Concatenate two strings.
//! ///
//! /// The returned value must be freed with `rust_free`
//! #[ffi_export]
//! fn rust_concat (fst: char_p::Ref<'_>, snd: char_p::Ref<'_>)
//!   -> char_p::Box
//! {
//!     let s: String = format!("{}{}\0", fst, snd);
//!     s   .try_into() // Try to convert to a boxed `char *` pointer
//!         .unwrap()   // Only fails if there is an inner nul byte.
//! }
//!
//! /// Frees a pointer obtained by calling `rust_concat`.
//! #[ffi_export]
//! fn rust_free (it: char_p::Box)
//! {
//!     drop(it);
//! }
//!
//! # macro_rules! ignore { ($($t:tt)*) => () } ignore! {
//! #[::safer_ffi::cfg_headers]
//! #[test]
//! # }
//! fn generate_c_header ()
//!   -> io::Result<()>
//! {
//!     ::safer_ffi::headers::builder()
//!         .with_guard("__ASGARD__")
//!         .to_file("filename.h")?
//!         .generate()
//! }
// //! # generate_c_header().unwrap();
//! ```
//!
//! so that
//!
//! ```shell
//! cargo test --features generate-headers -- \
//!     --exact generate_c_header \
//!     --nocapture
//! ```
//!
//! generates a `"filename.h"` file (⚠️ overwriting it if it exists ⚠️) with
//! the following contents:
//!
//! <pre style="color:#000020;background:#f6f8ff;"><span style="color:#3f7f8f; ">/*! \file */</span>
//! <span style="color:#3f7f8f; ">/*******************************************</span>
//! <span style="color:#3f7f8f; ">&nbsp;*                                         *</span>
//! <span style="color:#3f7f8f; ">&nbsp;*  File auto-generated by `::safer_ffi`.  *</span>
//! <span style="color:#3f7f8f; ">&nbsp;*                                         *</span>
//! <span style="color:#3f7f8f; ">&nbsp;*  Do not manually edit this file.        *</span>
//! <span style="color:#3f7f8f; ">&nbsp;*                                         *</span>
//! <span style="color:#3f7f8f; ">&nbsp;*******************************************/</span>
//!
//! <span style="color:#004a43; ">#</span><span style="color:#004a43; ">ifndef</span><span style="color:#004a43; "> __ASGARD__</span>
//! <span style="color:#004a43; ">#</span><span style="color:#004a43; ">define</span><span style="color:#004a43; "> __ASGARD__</span>
//!
//!
//! <span style="color:#3f7f8f; ">/** \brief</span>
//! <span style="color:#3f7f8f; ">&nbsp;*  Concatenate two strings.</span>
//! <span style="color:#3f7f8f; ">&nbsp;* </span>
//! <span style="color:#3f7f8f; ">&nbsp;*  The returned value must be freed with `rust_free_string`</span>
//! <span style="color:#3f7f8f; ">&nbsp;*/</span>
//! <span style="color:#200080; font-weight:bold; ">char</span> <span style="color:#308080; ">*</span> rust_concat <span style="color:#308080; ">(</span>
//!     <span style="color:#200080; font-weight:bold; ">char</span> <span style="color:#200080; font-weight:bold; ">const</span> <span style="color:#308080; ">*</span> fst<span style="color:#308080; ">,</span>
//!     <span style="color:#200080; font-weight:bold; ">char</span> <span style="color:#200080; font-weight:bold; ">const</span> <span style="color:#308080; ">*</span> snd<span style="color:#308080; ">)</span><span style="color:#406080; ">;</span>
//!
//! <span style="color:#3f7f8f; ">/** \brief</span>
//! <span style="color:#3f7f8f; ">&nbsp;*  Frees a pointer obtained by calling `rust_concat`.</span>
//! <span style="color:#3f7f8f; ">&nbsp;*/</span>
//! <span style="color:#200080; font-weight:bold; ">void</span> rust_free_string <span style="color:#308080; ">(</span>
//!     <span style="color:#200080; font-weight:bold; ">char</span> <span style="color:#308080; ">*</span> it<span style="color:#308080; ">)</span><span style="color:#406080; ">;</span>
//!
//!
//! <span style="color:#004a43; ">#</span><span style="color:#004a43; ">endif</span><span style="color:#004a43; "> </span><span style="color:#595979; ">/* __ASGARD__ */</span>
//! </pre>

#![allow(missing_copy_implementations, missing_debug_implementations)]

use ::std::{
    collections::HashSet,
    env,
    fs,
    io,
    path::Path,
};

use_prelude!();
use rust::{String, Vec};

pub use definer::{Definer, HashSetDefiner};
mod definer;

macro_rules! with_optional_fields {(
    $(
        $(#[$field_meta:meta])*
        $field:ident : $field_ty:ty
    ),* $(,)?
) => (
    #[derive(Default)]
    pub
    struct Builder<'__, W> {
        target: W,
        $(
            $field : Option<$field_ty>,
        )*
    }

    pub
    fn builder<'__> ()
      -> Builder<'__, WhereTo>
    {
        Builder::default()
    }

    use __::WhereTo;
    mod __ {
        #[derive(Default)]
        pub
        struct WhereTo;
    }

    ::paste::item! {
        impl<'__, W> Builder<'__, W> {
            $(
                $(#[$field_meta])*
                pub
                fn [<with_$field>] (self, $field : $field_ty)
                  -> Self
                {
                    let $field = Some($field);
                    Self {
                        $field,
                        .. self
                    }
                }
            )*
        }
    }

    impl<'__> Builder<'__, WhereTo> {
        /// Specify the path to the file to be generated.
        ///
        /// **⚠️ If it already exists, its contents will be overwritten ⚠️**
        ///
        /// There is no default value here, either `.to_file()` or [`.to_writer()`]
        /// need to be called to be able to [`.generate()`] the headers.
        ///
        /// For more fine-grained control over the "output stream" where the
        /// headers will be written to, use [`.to_writer()`].
        ///
        /// # Example
        ///
        /// ```rust,no_run
        /// # fn main () -> ::std::io::Result<()> { Ok({
        /// ::safer_ffi::headers::builder()
        ///     .to_file("my_header.h")?
        ///     .generate()?
        /// # })}
        /// ```
        ///
        /// [`.to_writer()`]: `Builder::to_writer`
        /// [`.generate()`]: `Builder::generate`
        pub
        fn to_file (
            self: Self,
            filename: impl AsRef<Path>,
        ) -> io::Result<Builder<'__, fs::File>>
        {
            Ok(self.to_writer(
                fs::OpenOptions::new()
                    .create(true)/*or*/.truncate(true)
                    .write(true)
                    .open(filename)?
            ))
        }

        /// Specify the [`Write`][`io::Write`] "stream" where the headers will
        /// be written to.
        ///
        /// # Example
        ///
        /// ```rust,no_run
        /// // Display the headers to the standard output
        /// // (may need the `--nocapture` flag when running the tests)
        /// # fn main () -> ::std::io::Result<()> { Ok({
        /// ::safer_ffi::headers::builder()
        ///     .to_writer(::std::io::stdout())
        ///     .generate()?
        /// # })}
        /// ```
        pub
        fn to_writer<W> (
            self: Self,
            out: W,
        ) -> Builder<'__, W>
        where
            W : io::Write
        {
            let Self {
                target: WhereTo, $(
                $field, )*
                ..
            } = self;
            Builder {
                target: out,
                $($field ,)*
            }
        }
    }

    impl<'__, W : io::Write> Builder<'__, W> {
        /// Generate the C header file.
        pub
        fn generate (self)
          -> io::Result<()>
        {
            let Self { mut target, $($field ,)* } = self;
            Builder {
                target: WhereTo, $(
                $field, )*
            }.generate_with_definer(HashSetDefiner {
                out: &mut target,
                defines_set: Default::default(),
            })
        }

        // pub
        // fn as_mut_dyn (self: &'__ mut Self)
        //   -> Builder<'__, &'__ mut dyn io::Write>
        // where
        //     W : '__,
        // {
        //     let Self { ref mut target, $($field ,)* } = *self;
        //     Builder {
        //         target, $(
        //         $field, )*
        //     }
        // }
    }
)}

with_optional_fields! {
    /// Sets up the name of the `ifndef` guard of the header file.
    ///
    /// It defaults to:
    ///
    /// ```rust,ignore
    /// format!("__RUST_{}__", env::var("CARGO_PKG_NAME")?.to_ascii_uppercase())
    /// ```
    guard: &'__ str,

    /// Sets up the banner of the generated C header file.
    ///
    /// It defaults to:
    ///
    /// ```rust,ignore
    /// concat!(
    ///     "/*! \\file */\n",
    ///     "/*******************************************\n",
    ///     " *                                         *\n",
    ///     " *  File auto-generated by `::safer_ffi`.  *\n",
    ///     " *                                         *\n",
    ///     " *  Do not manually edit this file.        *\n",
    ///     " *                                         *\n",
    ///     " *******************************************/\n",
    /// )
    /// ```
    ///
    /// <pre style="color:#000020;background:#f6f8ff;"><span style="color:#3f7f8f; ">/*! \file */</span>
    /// <span style="color:#3f7f8f; ">/*******************************************</span>
    /// <span style="color:#3f7f8f; ">&nbsp;*                                         *</span>
    /// <span style="color:#3f7f8f; ">&nbsp;*  File auto-generated by `::safer_ffi`.  *</span>
    /// <span style="color:#3f7f8f; ">&nbsp;*                                         *</span>
    /// <span style="color:#3f7f8f; ">&nbsp;*  Do not manually edit this file.        *</span>
    /// <span style="color:#3f7f8f; ">&nbsp;*                                         *</span>
    /// <span style="color:#3f7f8f; ">&nbsp;*******************************************/</span>
    /// </pre>
    banner: &'__ str,
}

impl Builder<'_, WhereTo> {
    /// More customizable version of [`.generate()`][`Builder::generate].
    ///
    /// With this call, one can provide a custom implementation of a [`Definer`],
    /// which can be useful for mock tests, mainly.
    pub
    fn generate_with_definer (self, mut definer: impl Definer)
      -> io::Result<()>
    {
        let s;
        let config = self;
        let guard: &'_ str =
            if let Some(it) = config.guard { it } else {
                s = format!("__RUST_{}__",
                    env::var("CARGO_PKG_NAME")
                        .unwrap()
                        .to_ascii_uppercase()
                );
                &*s
            }
        ;
        let banner: &'_ str = config.banner.unwrap_or(concat!(
            "/*! \\file */\n",
            "/*******************************************\n",
            " *                                         *\n",
            " *  File auto-generated by `::safer_ffi`.  *\n",
            " *                                         *\n",
            " *  Do not manually edit this file.        *\n",
            " *                                         *\n",
            " *******************************************/",
        ));

        write!(definer.out(),
            concat!(
                "{banner}\n\n",
                "#ifndef {guard}\n",
                "#define {guard}\n",
                "\n",
                "#ifdef __cplusplus\n",
                "extern \"C\" {{\n",
                "#endif\n\n",
            ),
            guard = guard,
            banner = banner,
        )?;
        crate::inventory::iter
            .into_iter()
            // Iterate in reverse fashion to more closely match
            // the Rust definition order.
            .collect::<Vec<_>>().into_iter().rev()
            .try_for_each(|crate::FfiExport(define)| define(&mut definer))
            ?
        ;
        write!(definer.out(),
            concat!(
                "\n",
                "#ifdef __cplusplus\n",
                "}} /* extern \"C\" */\n",
                "#endif\n",
                "\n",
                "#endif /* {} */\n",
            ),
            guard,
        )?;
        Ok(())
    }
}