// Copyright 2018 Syn Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A trait that can provide the `Span` of the complete contents of a syntax
//! tree node.
//!
//! *This module is available if Syn is built with both the `"parsing"` and
//! `"printing"` features.*
//!
//! # Example
//!
//! Suppose in a procedural macro we have a [`Type`] that we want to assert
//! implements the [`Sync`] trait. Maybe this is the type of one of the fields
//! of a struct for which we are deriving a trait implementation, and we need to
//! be able to pass a reference to one of those fields across threads.
//!
//! [`Type`]: ../enum.Type.html
//! [`Sync`]: https://doc.rust-lang.org/std/marker/trait.Sync.html
//!
//! If the field type does *not* implement `Sync` as required, we want the
//! compiler to report an error pointing out exactly which type it was.
//!
//! The following macro code takes a variable `ty` of type `Type` and produces a
//! static assertion that `Sync` is implemented for that type.
//!
//! ```
//! #[macro_use]
//! extern crate quote;
//!
//! extern crate syn;
//! extern crate proc_macro;
//! extern crate proc_macro2;
//!
//! use syn::Type;
//! use syn::spanned::Spanned;
//! use proc_macro::TokenStream;
//! use proc_macro2::Span;
//!
//! # const IGNORE_TOKENS: &str = stringify! {
//! #[proc_macro_derive(MyMacro)]
//! # };
//! pub fn my_macro(input: TokenStream) -> TokenStream {
//!     # let ty = get_a_type();
//!     /* ... */
//!
//!     let def_site = Span::def_site();
//!     let ty_span = ty.span().resolved_at(def_site);
//!     let assert_sync = quote_spanned! {ty_span=>
//!         struct _AssertSync where #ty: Sync;
//!     };
//!
//!     /* ... */
//!     # input
//! }
//! #
//! # fn get_a_type() -> Type {
//! #     unimplemented!()
//! # }
//! #
//! # fn main() {}
//! ```
//!
//! By inserting this `assert_sync` fragment into the output code generated by
//! our macro, the user's code will fail to compile if `ty` does not implement
//! `Sync`. The errors they would see look like the following.
//!
//! ```text
//! error[E0277]: the trait bound `*const i32: std::marker::Sync` is not satisfied
//!   --> src/main.rs:10:21
//!    |
//! 10 |     bad_field: *const i32,
//!    |                ^^^^^^^^^^ `*const i32` cannot be shared between threads safely
//! ```
//!
//! In this technique, using the `Type`'s span for the error message makes the
//! error appear in the correct place underlining the right type. But it is
//! **incredibly important** that the span for the assertion is **resolved** at
//! the procedural macro definition site rather than at the `Type`'s span. This
//! way we guarantee that it refers to the `Sync` trait that we expect. If the
//! assertion were **resolved** at the same place that `ty` is resolved, the
//! user could circumvent the check by defining their own `Sync` trait that is
//! implemented for their type.

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, Tokens};

/// A trait that can provide the `Span` of the complete contents of a syntax
/// tree node.
///
/// This trait is automatically implemented for all types that implement
/// [`ToTokens`] from the `quote` crate.
///
/// [`ToTokens`]: https://docs.rs/quote/0.4/quote/trait.ToTokens.html
///
/// See the [module documentation] for an example.
///
/// [module documentation]: index.html
///
/// *This trait is available if Syn is built with both the `"parsing"` and
/// `"printing"` features.*
pub trait Spanned {
    /// Returns a `Span` covering the complete contents of this syntax tree
    /// node, or [`Span::call_site()`] if this node is empty.
    ///
    /// [`Span::call_site()`]: https://docs.rs/proc-macro2/0.1/proc_macro2/struct.Span.html#method.call_site
    fn span(&self) -> Span;
}

impl<T> Spanned for T
where
    T: ToTokens,
{
    #[cfg(procmacro2_semver_exempt)]
    fn span(&self) -> Span {
        let mut tokens = Tokens::new();
        self.to_tokens(&mut tokens);
        let token_stream = TokenStream::from(tokens);
        let mut iter = token_stream.into_iter();
        let mut span = match iter.next() {
            Some(tt) => tt.span,
            None => {
                return Span::call_site();
            }
        };
        for tt in iter {
            if let Some(joined) = span.join(tt.span) {
                span = joined;
            }
        }
        span
    }

    #[cfg(not(procmacro2_semver_exempt))]
    fn span(&self) -> Span {
        let mut tokens = Tokens::new();
        self.to_tokens(&mut tokens);
        let token_stream = TokenStream::from(tokens);
        let mut iter = token_stream.into_iter();

        // We can't join spans without procmacro2_semver_exempt so just grab the
        // first one.
        match iter.next() {
            Some(tt) => tt.span,
            None => Span::call_site(),
        }
    }
}
