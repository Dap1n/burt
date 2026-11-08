use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    Expr, ExprLit, FnArg, ItemFn, Lit, Meta, Pat, PathArguments, Token, Type, parse::Parser,
    parse_macro_input, punctuated::Punctuated,
};

type Required = bool;

fn parse_param(ty: &Type) -> Option<(Required, &Ident)> {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        if segment.ident.to_string() == "Option" {
            if let PathArguments::AngleBracketed(args) = &segment.arguments
                && let Some(syn::GenericArgument::Type(inner_type)) = args.args.first()
                && let Type::Path(inner_path) = inner_type
                && let Some(inner_seg) = inner_path.path.segments.last()
            {
                Some((false, &inner_seg.ident))
            } else {
                None
            }
        } else {
            Some((true, &segment.ident))
        }
    } else {
        None
    }
}

fn parse_doc_to_clog_segments(doc: &str) -> TokenStream2 {
    let mut segments = Vec::new();
    let mut in_code = false;
    let mut current_chunk = String::new();

    for ch in doc.chars() {
        if ch == '`' {
            if !current_chunk.is_empty() {
                let color = if in_code {
                    quote! { crate::structs::Color::YELLOW }
                } else {
                    quote! { crate::structs::Color::WHITE }
                };
                segments.push(quote! {
                    [#color => "{}", #current_chunk]
                });
                current_chunk.clear();
            }
            in_code = !in_code;
        } else {
            current_chunk.push(ch);
        }
    }

    if !current_chunk.is_empty() {
        let color = if in_code {
            quote! { crate::structs::Color::YELLOW }
        } else {
            quote! { crate::structs::Color::WHITE }
        };
        segments.push(quote! {
            [#color => "{}", #current_chunk]
        });
    }

    let mut out = TokenStream2::new();
    for (i, seg) in segments.iter().enumerate() {
        out.extend(seg.clone());
        if i < segments.len() - 1 {
            out.extend(quote! { , });
        }
    }
    out
}

#[proc_macro_attribute]
pub fn burt_command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let attrs = &input_fn.attrs;
    let vis = &input_fn.vis;
    let fn_name = &input_fn.sig.ident;
    let stmts = &input_fn.block.stmts;

    let mut is_cheat = false;
    let mut var_path: Option<syn::Path> = None;

    if !attr.is_empty() {
        let ts: TokenStream2 = attr.into();
        let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
        let metas = match parser.parse2(ts.clone()) {
            Ok(m) => m,
            Err(e) => return e.to_compile_error().into(),
        };
        for meta in metas {
            match &meta {
                Meta::Path(path) if path.is_ident("cheat") => {
                    is_cheat = true;
                }
                Meta::NameValue(nv) if nv.path.is_ident("var") => {
                    if let Expr::Path(expr_path) = &nv.value {
                        var_path = Some(expr_path.path.clone());
                        continue;
                    }
                    return syn::Error::new_spanned(&meta, "Expected `var = PATH`")
                        .to_compile_error()
                        .into();
                }
                _ => {
                    return syn::Error::new_spanned(&meta, "Unknown meta")
                        .to_compile_error()
                        .into();
                }
            }
        }
    }

    let vis_str = if matches!(vis, syn::Visibility::Inherited) {
        String::new()
    } else {
        format!("{} ", quote!(#vis))
    };

    let params_str = input_fn
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                let pat = &pat_type.pat;
                let ty = &pat_type.ty;
                Some(format!("{}: {}", quote!(#pat), quote!(#ty)))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(", ");

    let clean_params = params_str
        .replace(" : ", ": ")
        .replace(" < ", "<")
        .replace(" >", ">");

    let prefix = {
        let mut segments = TokenStream2::new();
        let fn_name_str = fn_name.to_string();

        segments.extend(if is_cheat {
            quote! { [crate::structs::Color::RED => "Cheat: "], }
        } else {
            quote! { [crate::structs::Color::LIME => "Signature: "], }
        });

        segments.extend(quote! {
            [crate::structs::Color::GRAY => "{}fn ", #vis_str],
            [crate::structs::Color::LIME => "{}", #fn_name_str],
            [crate::structs::Color::GRAY => "("],
        });

        let args: Vec<_> = clean_params.split(',').collect();
        for (i, arg) in args.iter().enumerate() {
            let arg = arg.trim();
            if let Some((name, ty)) = arg.split_once(':') {
                let name = name.trim();
                let ty = ty.trim();
                segments.extend(quote! {
                    [crate::structs::Color::WHITE => "{}: ", #name],
                    [crate::structs::Color::LIME => "{}", #ty],
                });
            } else {
                segments.extend(quote! {
                    [crate::structs::Color::WHITE => "{}", #arg],
                });
            }
            if i < args.len() - 1 {
                segments.extend(quote! {
                    [crate::structs::Color::GRAY => ", "],
                });
            }
        }

        segments.extend(quote! {
            [crate::structs::Color::GRAY => ")"],
            [crate::structs::Color::WHITE => "\n"]
        });

        if let Some(v) = &var_path {
            segments.extend(quote! {
                ,
                [crate::structs::Color::YELLOW => "Current value: "],
                [crate::structs::Color::WHITE => "{}\n\n", crate::vars::BurtVar::display(&#v)]
            });
        }

        segments
    };

    let mut doc_lines = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("doc")
            && let Meta::NameValue(meta_name_value) = &attr.meta
            && let Expr::Lit(ExprLit {
                lit: Lit::Str(lit_str),
                ..
            }) = &meta_name_value.value
        {
            doc_lines.push(lit_str.value().trim().to_string());
        }
    }

    if doc_lines.is_empty() {
        return syn::Error::new_spanned(&input_fn, "Missing command documentation")
            .to_compile_error()
            .into();
    }

    let mut param_bindings = Vec::new();
    let mut arg_index: usize = 1;
    let mut required_params_exist = false;
    let mut opt_param_found = false;
    let mut args_param_found = false;

    for arg in &input_fn.sig.inputs {
        if let FnArg::Typed(pat_type) = arg
            && let Pat::Ident(pat_ident) = &*pat_type.pat
        {
            if args_param_found {
                return syn::Error::new_spanned(&arg, "Parameters after `args` not allowed")
                    .to_compile_error()
                    .into();
            }

            let param_name = &pat_ident.ident;
            let param_str = param_name.to_string();
            let Some((param_required, param_ident)) = parse_param(&pat_type.ty) else {
                return syn::Error::new_spanned(&arg, "Can't parse parameter")
                    .to_compile_error()
                    .into();
            };

            required_params_exist = required_params_exist || param_required;

            if opt_param_found && param_required {
                return syn::Error::new_spanned(
                    &arg,
                    "Required parameters after optional ones not allowed",
                )
                .to_compile_error()
                .into();
            }
            opt_param_found = opt_param_found || !param_required;

            let binding = match (param_str.as_str(), param_required) {
                ("args", true) => {
                    args_param_found = true;
                    quote! {
                        let #param_name = if let Some(remainder) = command.get_args_remainder(#arg_index) {
                            match <#param_ident as crate::params::BurtParam>::from_input(remainder) {
                                Ok(parsed) => parsed,
                                Err(err_str) => {
                                    crate::clog!(
                                        #prefix,
                                        [crate::structs::Color::WHITE => "Special parameter "],
                                        [crate::structs::Color::RED => "args"],
                                        [crate::structs::Color::WHITE => " "],
                                        [crate::structs::Color::WHITE => "{}", err_str],
                                    );
                                    return;
                                }
                            }
                        } else {
                            crate::clog!(
                                #prefix,
                                [crate::structs::Color::WHITE => "Special parameter "],
                                [crate::structs::Color::RED => "args"],
                                [crate::structs::Color::WHITE => " is required but not provided"],
                            );
                            return;
                        };
                    }
                }
                ("args", false) => {
                    args_param_found = true;
                    quote! {
                        let #param_name = if let Some(remainder) = command.get_args_remainder(#arg_index) {
                            match <#param_ident as crate::params::BurtParam>::from_input(remainder) {
                                Ok(parsed) => Some(parsed),
                                Err(err_str) => {
                                    crate::clog!(
                                        #prefix,
                                        [crate::structs::Color::WHITE => "Special parameter "],
                                        [crate::structs::Color::RED => "args"],
                                        [crate::structs::Color::WHITE => " "],
                                        [crate::structs::Color::WHITE => "{}", err_str],
                                    );
                                    return;
                                }
                            }
                        } else {
                            None
                        };
                    }
                }
                (_, true) => {
                    quote! {
                        let #param_name = if let Some(input) = command.get_arg(#arg_index) {
                            match <#param_ident as crate::params::BurtParam>::from_input(input) {
                                Ok(parsed) => parsed,
                                Err(err_str) => {
                                    crate::clog!(
                                        #prefix,
                                        [crate::structs::Color::WHITE => "Parameter "],
                                        [crate::structs::Color::RED => "{}", #param_str],
                                        [crate::structs::Color::WHITE => " "],
                                        [crate::structs::Color::WHITE => "{}", err_str],
                                    );
                                    return;
                                }
                            }
                        } else {
                            crate::clog!(
                                #prefix,
                                [crate::structs::Color::WHITE => "Parameter "],
                                [crate::structs::Color::RED => "{}", #param_str],
                                [crate::structs::Color::WHITE => " is required but not provided"],
                            );
                            return;
                        };
                    }
                }
                (_, false) => {
                    quote! {
                        let #param_name = if let Some(input) = command.get_arg(#arg_index) {
                            match <#param_ident as crate::params::BurtParam>::from_input(input) {
                                Ok(parsed) => Some(parsed),
                                Err(err_str) => {
                                    crate::clog!(
                                        #prefix,
                                        [crate::structs::Color::WHITE => "Parameter "],
                                        [crate::structs::Color::RED => "{}", #param_str],
                                        [crate::structs::Color::WHITE => " "],
                                        [crate::structs::Color::WHITE => "{}", err_str],
                                    );
                                    return;
                                }
                            }
                        } else {
                            None
                        };
                    }
                }
            };

            param_bindings.push(binding);
            arg_index += 1;
        }
    }

    let clog_stmt = {
        let doc_segments = parse_doc_to_clog_segments(&doc_lines.join("\n"));
        quote! {
            if #required_params_exist && command.is_empty()
                || command.get_args_remainder(1).is_some_and(|s| s.trim_end() == "?")
            {
                crate::clog!(
                    #prefix,
                    #doc_segments,
                );
                return;
            }
            if !#args_param_found && command.get_arg(#arg_index).is_some() {
                crate::clog!(
                    #prefix,
                    [crate::structs::Color::RED => "Too many arguments"],
                );
                return;
            }
        }
    };

    let cheat_doc_prefix = if is_cheat {
        quote! {
            /// Flagged as a cheat.
            ///
        }
    } else {
        quote! {}
    };

    let cheat_guard = if is_cheat {
        quote! {
            if !crate::game::cvars::cheats_enabled() {
                crate::clog!(err_prefix;
                    [crate::structs::Color::WHITE => "This command is cheat protected. Consider using "],
                    [crate::structs::Color::RED => "sv_cheats 1"],
                );
                return;
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #cheat_doc_prefix
        #(#attrs)*
        #vis extern "C" fn #fn_name(_: *const ::std::ffi::c_void, command: *const crate::bridge::commands::CCommand) {
            if command.is_null() {
                return;
            }
            let command = unsafe { &*command };

            #clog_stmt
            #cheat_guard

            #(#param_bindings)*
            #(#stmts)*
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn burt_autocomplete(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let vis = &input_fn.vis;
    let fn_name = &input_fn.sig.ident;
    let stmts = &input_fn.block.stmts;

    let mut args = input_fn.sig.inputs.iter();

    let Some(syn::FnArg::Typed(partial_arg)) = args.next() else {
        return syn::Error::new_spanned(&input_fn, "Expected `partial: String` as first argument")
            .to_compile_error()
            .into();
    };
    let Some(syn::FnArg::Typed(writer_arg)) = args.next() else {
        return syn::Error::new_spanned(
            &input_fn,
            "Expected `writer: &mut AutocompleteWriter` as second argument",
        )
        .to_compile_error()
        .into();
    };

    let impl_fn_name = syn::Ident::new(&format!("{}_impl", fn_name), fn_name.span());
    let partial_name = &partial_arg.pat;
    let writer_name = &writer_arg.pat;

    let expanded = quote! {
        #[inline]
        fn #impl_fn_name(#partial_name: String, #writer_name: &mut AutocompleteWriter) {
            #(#stmts)*
        }

        #vis unsafe extern "C" fn #fn_name(partial: *const std::ffi::c_char, commands: *mut std::ffi::c_char) -> std::ffi::c_int {
            let partial_str = if partial.is_null() {
                String::new()
            } else {
                unsafe { std::ffi::CStr::from_ptr(partial).to_string_lossy().into_owned() }
            };

            let mut writer = unsafe { AutocompleteWriter::new(commands) };

            #impl_fn_name(partial_str, &mut writer);

            writer.len() as std::ffi::c_int
        }
    };

    TokenStream::from(expanded)
}
