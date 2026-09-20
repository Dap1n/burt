use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, ExprLit, FnArg, ItemFn, Lit, Meta, Pat, PathArguments, Type, parse_macro_input};

enum ParamType {
    String,
    OptionString,
    I32,
    OptionI32,
    F32,
    OptionF32,
    Bool,
    OptionBool,
    Unsupported,
}

impl ParamType {
    fn is_required(&self) -> bool {
        matches!(
            self,
            Self::String | Self::I32 | Self::F32 | Self::Bool | Self::Unsupported
        )
    }
}

fn parse_param_type(ty: &Type) -> ParamType {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        let ident = segment.ident.to_string();
        if ident == "String" {
            return ParamType::String;
        } else if ident == "i32" {
            return ParamType::I32;
        } else if ident == "f32" {
            return ParamType::F32;
        } else if ident == "bool" {
            return ParamType::Bool;
        } else if ident == "Option"
            && let PathArguments::AngleBracketed(args) = &segment.arguments
            && let Some(syn::GenericArgument::Type(inner_type)) = args.args.first()
            && let Type::Path(inner_path) = inner_type
            && let Some(inner_seg) = inner_path.path.segments.last()
        {
            let inner_ident = inner_seg.ident.to_string();
            if inner_ident == "String" {
                return ParamType::OptionString;
            } else if inner_ident == "i32" {
                return ParamType::OptionI32;
            } else if inner_ident == "f32" {
                return ParamType::OptionF32;
            } else if inner_ident == "bool" {
                return ParamType::OptionBool;
            }
        }
    }

    ParamType::Unsupported
}

#[proc_macro_attribute]
pub fn burt_command(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let attrs = &input_fn.attrs;
    let vis = &input_fn.vis;
    let fn_name = &input_fn.sig.ident;
    let stmts = &input_fn.block.stmts;

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

    let sig_str = format!("{vis_str}fn {fn_name}({clean_params})");

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

    for arg in &input_fn.sig.inputs {
        if let FnArg::Typed(pat_type) = arg
            && let Pat::Ident(pat_ident) = &*pat_type.pat
        {
            let param_name = &pat_ident.ident;
            let param_str = param_name.to_string();
            let param_type = parse_param_type(&pat_type.ty);

            required_params_exist = required_params_exist || param_type.is_required();

            let binding = match (param_str.as_str(), param_type) {
                ("args", ParamType::String) => {
                    quote! {
                        let #param_name: String = command.get_args_remainder(#arg_index).unwrap_or_default().to_string();
                        if #param_name.is_empty() {
                            clog!(
                                [crate::bridge::console::Color::RED => "{}", #param_str],
                                [crate::bridge::console::Color::WHITE => " parameter expects a non-empty string"],
                            );
                            return;
                        }
                    }
                }
                ("args", ParamType::OptionString) => {
                    quote! {
                        let #param_name: Option<String> = command.get_args_remainder(#arg_index).map(String::from);
                    }
                }
                (_, ParamType::String) => {
                    quote! {
                        let #param_name: String = command.get_arg(#arg_index).unwrap_or_default().to_string();
                        if #param_name.is_empty() {
                            clog!(
                                [crate::bridge::console::Color::RED => "{}", #param_str],
                                [crate::bridge::console::Color::WHITE => " parameter expects a non-empty string"],
                            );
                            return;
                        }
                    }
                }
                (_, ParamType::OptionString) => {
                    quote! {
                        let #param_name: Option<String> = command.get_arg(#arg_index).map(String::from);
                    }
                }
                (_, ParamType::I32) => {
                    quote! {
                        let #param_name: i32 = if let Some(arg) = command.get_arg(#arg_index).and_then(|s| s.parse().ok()) {
                            arg
                        } else {
                            clog!(
                                [crate::bridge::console::Color::RED => "{}", #param_str],
                                [crate::bridge::console::Color::WHITE => " parameter expects a parsable i32 integer"],
                            );
                            return;
                        };
                    }
                }
                (_, ParamType::OptionI32) => {
                    quote! {
                        let #param_name: Option<i32> = command.get_arg(#arg_index)
                            .and_then(|s| s.parse::<i32>().ok());
                    }
                }
                (_, ParamType::F32) => {
                    quote! {
                        let #param_name: f32 = if let Some(arg) = command.get_arg(#arg_index).and_then(|s| s.parse().ok()) {
                            arg
                        } else {
                            clog!(
                                [crate::bridge::console::Color::RED => "{}", #param_str],
                                [crate::bridge::console::Color::WHITE => " parameter expects a parsable f32 number"],
                            );
                            return;
                        };
                    }
                }
                (_, ParamType::OptionF32) => {
                    quote! {
                        let #param_name: Option<f32> = command.get_arg(#arg_index)
                            .and_then(|s| s.parse::<f32>().ok());
                    }
                }
                (_, ParamType::Bool) => {
                    quote! {
                        let #param_name: bool = match command
                            .get_arg(#arg_index)
                            .unwrap_or_default()
                            .to_lowercase()
                            .as_str()
                        {
                            "1" | "true" | "t" | "yes" | "y" | "enabled" => true,
                            "0" | "false" | "f" | "no" | "n" | "disabled" => false,
                            _ => {
                                clog!(
                                    [crate::bridge::console::Color::RED => "{}", #param_str],
                                    [crate::bridge::console::Color::WHITE => " parameter expects a parsable boolean"],
                                );
                                return;
                            }
                        };
                    }
                }
                (_, ParamType::OptionBool) => {
                    quote! {
                        let #param_name: Option<bool> = match command
                            .get_arg(#arg_index)
                            .unwrap_or_default()
                            .to_lowercase()
                            .as_str()
                        {
                            "1" | "true" | "t" | "yes" | "y" | "enabled" => Some(true),
                            "0" | "false" | "f" | "no" | "n" | "disabled" => Some(false),
                            _ => None,
                        };
                    }
                }
                _ => {
                    return syn::Error::new_spanned(
                        &pat_type.ty,
                        "Unsupported parameter type in #[burt_command]",
                    )
                    .to_compile_error()
                    .into();
                }
            };

            param_bindings.push(binding);
            arg_index += 1;
        }
    }

    let clog_stmt = {
        let doc_msg = doc_lines.join("\n");
        quote! {
            if #required_params_exist && command.is_empty()
                || #clean_params.is_empty()
                && command.get_arg(1).is_some()
                || !#required_params_exist
                && !#clean_params.is_empty()
                && command.get_args_remainder(1).is_some_and(|s| s.chars().all(|c| ['?', ' '].contains(&c)))
            {
                clog!(
                    [crate::bridge::console::Color::PURPLE => "Signature: "],
                    [crate::bridge::console::Color::WHITE => "{}\n{}", #sig_str, #doc_msg]
                );
                return;
            }
        }
    };

    let expanded = quote! {
        #(#attrs)*
        #vis extern "C" fn #fn_name(_this: *const ::std::ffi::c_void, command: *const CCommand) {
            if command.is_null() {
                return;
            }
            let command = unsafe { &*command };

            #clog_stmt

            #(#param_bindings)*

            #(#stmts)*
        }
    };

    TokenStream::from(expanded)
}
