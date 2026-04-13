//! A macro to enforce strict typing on the fields in Rust.
//!
//! Please refer to the documentation of the macro for more details:
//! [`macro@strict_types`].

use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Fields, Ident, Item, Path, ReturnType, Token, Type,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
};

#[derive(Default, Clone)]
enum Mode {
    #[default]
    Default,
    Allow(Vec<Path>),
    Disallow(Vec<Path>),
}

#[derive(Default)]
struct StrictTypesArgs {
    disallow: Vec<Path>,
    mode: Mode,
}

impl Parse for StrictTypesArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::default());
        }

        let key: Ident = input.parse()?;
        let content;
        let _ = syn::parenthesized!(content in input);

        let paths: Punctuated<Path, Token![,]> =
            content.parse_terminated(Path::parse, Token![,])?;
        let paths_vec: Vec<Path> = paths.into_iter().collect();

        let mode;
        let mut disallow = default_disallowed_types();
        let disallow = match key.to_string().as_str() {
            "disallow" => {
                // let new_paths: Vec<Path> = paths_vec
                //     .iter()
                //     .filter(|path| !disallow.contains(path))
                //     .cloned()
                //     .collect();
                // mode = Mode::Disallow(new_paths);
                mode = Mode::Disallow(paths_vec.clone());
                disallow.extend(paths_vec);
                disallow
            }
            "allow" => {
                mode = Mode::Allow(paths_vec.clone());
                disallow.retain(|path| !paths_vec.contains(path));
                disallow
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    key,
                    "expected `disallow(...)` or `allow(...)`",
                ));
            }
        };

        Ok(Self { disallow, mode })
    }
}

fn default_disallowed_types() -> Vec<Path> {
    vec![
        parse_quote!(u8),
        parse_quote!(u16),
        parse_quote!(u32),
        parse_quote!(u64),
        parse_quote!(u128),
        parse_quote!(usize),
        parse_quote!(i8),
        parse_quote!(i16),
        parse_quote!(i32),
        parse_quote!(i64),
        parse_quote!(i128),
        parse_quote!(isize),
        parse_quote!(f32),
        parse_quote!(f64),
        parse_quote!(bool),
        parse_quote!(char),
    ]
}

fn contains_forbidden_type(ty: &Type, disallowed: &[Path]) -> bool {
    match ty {
        Type::Path(type_path) => {
            if disallowed.contains(&type_path.path) {
                return true;
            }

            for segment in &type_path.path.segments {
                if let syn::PathArguments::AngleBracketed(generic_args) = &segment.arguments {
                    for arg in &generic_args.args {
                        if let syn::GenericArgument::Type(inner_ty) = arg {
                            if contains_forbidden_type(inner_ty, disallowed) {
                                return true;
                            }
                        }
                    }
                }
            }

            false
        }

        Type::Tuple(tuple) => tuple
            .elems
            .iter()
            .any(|elem| contains_forbidden_type(elem, disallowed)),

        Type::Group(group) => contains_forbidden_type(&group.elem, disallowed),
        Type::Paren(paren) => contains_forbidden_type(&paren.elem, disallowed),

        _ => false, // you can expand this for more complex cases like references, impl traits, etc.
    }
}

fn doc_lines(attrs: &[syn::Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                if let Ok(nv) = attr.meta.clone().require_name_value() {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = &nv.value
                    {
                        return Some(s.value().trim().to_string());
                    }
                }
            }
            None
        })
        .collect()
}

fn verify_docs(mode: Mode, docs: &[String], input: &Item) -> Vec<syn::Error> {
    let mut errors = Vec::new();

    if let Mode::Allow(paths) | Mode::Disallow(paths) = &mode {
        let mut strict_section_found = false;
        let mut documented_types = Vec::new();

        for line in docs {
            if line.trim() == "# Strictness" {
                strict_section_found = true;
                continue;
            }

            if strict_section_found {
                if let Some(rest) = line.trim().strip_prefix("- [") {
                    if let Some(end_idx) = rest.find(']') {
                        let type_str = &rest[..end_idx];
                        documented_types.push(type_str.to_string());
                    }
                }
            }
        }

        for path in paths {
            let ty_str = quote!(#path).to_string();
            if !documented_types.iter().any(|doc| doc == &ty_str) {
                errors.push(syn::Error::new_spanned(
                    path,
                    format!(
                        "Missing `/// - [{ty_str}] justification` in `/// # Strictness` section"
                    ),
                ));
            }
        }

        if errors.is_empty() && !strict_section_found {
            errors.push(syn::Error::new_spanned(
                input,
                "Missing `/// # Strictness` section for `allow(...)` or `disallow(...)` override",
            ));
        }
    }

    errors
}

/// A macro to enforce strict typing on struct and enum fields.
/// It checks if any field uses a primitive type and generates a
/// compile-time error if it does. The idea is to encourage the use of
/// newtype wrappers for primitive types to ensure type safety and
/// clarity in the codebase.
///
/// The motivation behind this macro is to prevent the use of primitive
/// types directly in structs, which can lead to confusion and bugs.
/// The primitive types are often too generic and have a too wide range
/// of values, can be misused in different contexts, and do not
/// convey the intent of the data being represented, especially meaning
/// having useful names for the types and intentions behind them.
///
/// Also, often, the primitive types are not only checked for the width
/// of the allowed range of values, but must also contain some values
/// that are not allowed from within the allowed range. For example,
/// a `u8` type can be used to represent a percentage, but it can also
/// be used to represent a count of items, which is a different
/// concept. In this case, the `u8` type does not convey the intent of
/// the data being represented, and it is better to use a newtype wrapper
/// to make the intent clear. There might be at least two "Percentage"
/// types in the codebase, one is limited to the range of `0-100`, and
/// another type which can go beyond 100 (but still not less than zero),
/// to express the surpassing of the 100% mark. Not to mention that
/// sometimes, in certain contexts, the percentage can be negative
/// (e.g. when calculating the difference between two values).
/// This macro is a way to enforce the use of newtype wrappers for
/// primitive types in structs, which can help to avoid confusion and
/// bugs in the codebase. It is a compile-time check that will generate
/// an error if any field in a struct uses a primitive type directly.
///
/// # Example usage:
///
/// ```rust
/// use strict_typing::strict_types;
///
/// #[repr(transparent)]
/// struct MyNewTypeWrapper<T>(T);
///
/// #[strict_types]
/// struct MyStruct {
///     // This will generate a compile-time error
///     // because `u8` is a primitive type.
///     // my_field: u8,
///     // But this not:
///     my_field: MyNewTypeWrapper<u8>,
/// }
/// ```
///
/// Yes, this is a very simple macro, but it is intended to be used
/// as a way to enforce strict typing in the codebase, and to encourage
/// the use of newtype wrappers for primitive types in structs.
///
/// /// # Example with `disallow` which **adds** types to the disallowed
/// list:
///
/// ```rust,ignore,no_run
/// use strict_typing::strict_types;
///
/// #[strict_types(disallow(String))]
/// struct MyStruct {
///    // This will generate a compile-time error
///    // because `String` is now also a forbidden type.
///    my_field: String,
/// }
/// ```
///
/// When a type is added to the disallowed list or removed from it,
/// the macro requires the user to document the reason for
/// the change in the `/// # Strictness` section of the documentation.
/// The documentation should be in the form of a list of items,
/// where each item is a type that is allowed or disallowed, example:
///
/// ```rust,ignore,no_run
/// use strict_typing::strict_types;
///
/// /// # Strictness
/// ///
/// /// - [String] - this is a disallowed type, because it is too bad.
/// #[strict_types(disallow(String))]
/// struct MyStruct {
///     my_field: String,
/// }
/// ```
///
/// To remove from the default disallow list, you can use the
/// `allow` directive:
/// ```rust,ignore,no_run
/// use strict_typing::strict_types;
/// /// # Strictness
/// ///
/// /// - [u8] - this is an allowed type, because it is used for
/// ///   representing a small number of items.
/// #[strict_types(allow(u8))]
/// struct MyStruct {
///     my_field: u8,
/// }
/// ```
///
/// The macro also supports working directly on the whole `impl` and
/// `trait` items, analysing the function signatures and their
/// return types; however, annotating a trait method or an impl method
/// is yet impossible due to Rust limitations.
#[proc_macro_attribute]
pub fn strict_types(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as StrictTypesArgs);
    let item_clone = item.clone();
    let input = parse_macro_input!(item as Item);

    let disallowed: Vec<Path> = if args.disallow.is_empty() {
        default_disallowed_types()
    } else {
        args.disallow
    };

    let mut errors = Vec::new();

    let attrs = match &input {
        Item::Struct(struct_item) => {
            for field in &struct_item.fields {
                if let Type::Path(tp) = &field.ty {
                    if contains_forbidden_type(&field.ty, &disallowed) {
                        let fname = field
                            .ident
                            .as_ref()
                            .map(|i| i.to_string())
                            .unwrap_or("<unnamed>".into());
                        errors.push(syn::Error::new_spanned(
                            &field.ty,
                            format!("field `{}` uses disallowed type `{}`", fname, quote!(#tp)),
                        ));
                    }
                }
            }
            &struct_item.attrs
        }

        Item::Enum(enum_item) => {
            for variant in &enum_item.variants {
                match &variant.fields {
                    Fields::Unit => {}
                    Fields::Named(fields) => {
                        for field in &fields.named {
                            if let Type::Path(tp) = &field.ty {
                                if contains_forbidden_type(&field.ty, &disallowed) {
                                    errors.push(syn::Error::new_spanned(
                                        &field.ty,
                                        format!(
                                            "variant `{}` has field with disallowed type `{}`",
                                            variant.ident,
                                            quote!(#tp)
                                        ),
                                    ));
                                }
                            }
                        }
                    }
                    Fields::Unnamed(fields) => {
                        for field in &fields.unnamed {
                            if let Type::Path(tp) = &field.ty {
                                if contains_forbidden_type(&field.ty, &disallowed) {
                                    errors.push(syn::Error::new_spanned(
                                        &field.ty,
                                        format!(
                                            "variant `{}` has field with disallowed type `{}`",
                                            variant.ident,
                                            quote!(#tp)
                                        ),
                                    ));
                                }
                            }
                        }
                    }
                }
            }

            &enum_item.attrs
        }

        Item::Fn(fn_item) => {
            let sig = &fn_item.sig;

            for arg in &sig.inputs {
                if let syn::FnArg::Typed(pat_type) = arg {
                    if let Type::Path(tp) = &*pat_type.ty {
                        if contains_forbidden_type(&pat_type.ty, &disallowed) {
                            let path = &tp.path;
                            let arg_str = quote!(#path).to_string();
                            errors.push(syn::Error::new_spanned(
                                &pat_type.ty,
                                format!("function parameter uses disallowed type `{arg_str}`"),
                            ));
                        }
                    }
                }
            }

            if let ReturnType::Type(_, ty) = &fn_item.sig.output {
                if let Type::Path(tp) = ty.as_ref() {
                    if contains_forbidden_type(ty, &disallowed) {
                        errors.push(syn::Error::new_spanned(
                            tp,
                            format!(
                                "function return type is disallowed: `{}`",
                                tp.path.to_token_stream()
                            ),
                        ));
                    }
                }
            }

            errors.extend(verify_docs(args.mode, &doc_lines(&fn_item.attrs), &input));

            let diagnostics = errors.into_iter().map(|e| e.to_compile_error());
            let output = quote! {
                #fn_item
                #(#diagnostics)*
            };

            return output.into();
        }

        Item::Trait(item_trait) => {
            for item in &item_trait.items {
                if let syn::TraitItem::Fn(method) = item {
                    if let ReturnType::Type(_, ty) = &method.sig.output {
                        if let Type::Path(tp) = ty.as_ref() {
                            if contains_forbidden_type(ty, &disallowed) {
                                errors.push(syn::Error::new_spanned(
                                    tp,
                                    format!(
                                        "trait method return type is disallowed: `{}`",
                                        tp.path.to_token_stream()
                                    ),
                                ));
                            }
                        }
                    }

                    for arg in &method.sig.inputs {
                        if let syn::FnArg::Typed(pat_type) = arg {
                            if let Type::Path(tp) = &*pat_type.ty {
                                if contains_forbidden_type(&pat_type.ty, &disallowed) {
                                    let path = &tp.path;
                                    let arg_str = quote!(#path).to_string();
                                    errors.push(syn::Error::new_spanned(
                                        &pat_type.ty,
                                        format!("trait method parameter uses disallowed type `{arg_str}`"),
                                    ));
                                }
                            }
                        }
                    }
                }
            }

            &item_trait.attrs
        }

        Item::Impl(item_impl) => {
            for item in &item_impl.items {
                if let syn::ImplItem::Fn(method) = item {
                    if let ReturnType::Type(_, ty) = &method.sig.output {
                        if let Type::Path(tp) = ty.as_ref() {
                            if contains_forbidden_type(ty, &disallowed) {
                                errors.push(syn::Error::new_spanned(
                                    tp,
                                    format!(
                                        "impl method return type is disallowed: `{}`",
                                        tp.path.to_token_stream()
                                    ),
                                ));
                            }
                        }
                    }

                    for arg in &method.sig.inputs {
                        if let syn::FnArg::Typed(pat_type) = arg {
                            if let Type::Path(tp) = &*pat_type.ty {
                                if contains_forbidden_type(&pat_type.ty, &disallowed) {
                                    let path = &tp.path;
                                    let arg_str = quote!(#path).to_string();
                                    errors.push(syn::Error::new_spanned(
                                        &pat_type.ty,
                                        format!(
                                            "impl method parameter uses disallowed type `{arg_str}`"
                                        ),
                                    ));
                                }
                            }
                        }
                    }
                }
            }

            &item_impl.attrs
        }

        _ => {
            errors.push(syn::Error::new_spanned(
                &input,
                "#[strict_types] only works on structs, enums, functions, impls and traits",
            ));

            let original = proc_macro2::TokenStream::from(item_clone);
            let diagnostics = errors.into_iter().map(|e| e.to_compile_error());

            return quote! {
                #original
                #(#diagnostics)*
            }
            .into();
        }
    };

    errors.extend(verify_docs(args.mode, &doc_lines(attrs), &input));

    let original = proc_macro2::TokenStream::from(item_clone);
    let diagnostics = errors.into_iter().map(|e| e.to_compile_error());

    quote! {
        #original
        #(#diagnostics)*
    }
    .into()
}

// #[proc_macro_attribute]
// pub fn strict_types(attr: TokenStream, item: TokenStream) -> TokenStream {
//     let forbidden = {
//         let parsed = parse_macro_input!(attr as StrictTypesArgs);

//         if parsed.disallow.is_empty() {
//             default_forbidden_types()
//         } else {
//             parsed.disallow
//         }
//     };

//     let input = parse_macro_input!(item as DeriveInput);
//     let ident = &input.ident;

//     let error_tokens = if let Data::Struct(data_struct) = &input.data {
//         let mut errors = Vec::new();

//         for field in data_struct.fields.iter() {
//             if let Type::Path(type_path) = &field.ty {
//                 if let Some(ident) = type_path.path.get_ident() {
//                     if forbidden.contains(&type_path.path) {
//                         let field_name = field
//                             .ident
//                             .as_ref()
//                             .map(|i| i.to_string())
//                             .unwrap_or("<unnamed>".into());

//                         errors.push(syn::Error::new_spanned(
//                             &field.ty,
//                             format!(
//                                 "field `{field_name}` uses forbidden primitive type `{ty_str}` — use a newtype wrapper"
//                             ),
//                         ));
//                     }
//                 }
//             }
//         }

//         if errors.is_empty() {
//             quote! {}
//         } else {
//             let combined = errors.iter().map(syn::Error::to_compile_error);
//             quote! { #(#combined)* }
//         }
//     } else {
//         syn::Error::new_spanned(ident, "#[enforce_strict_types] only works on structs")
//             .to_compile_error()
//     };

//     let output = quote! {
//         #input
//         #error_tokens
//     };

//     output.into()
// }
