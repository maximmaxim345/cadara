#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cognitive_complexity)]

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input, token, Error, FnArg, Ident, ItemFn, Pat, PatType, Receiver, Result,
    ReturnType, Token, Type, TypeReference, TypeTuple,
};

/// Parsed arguments passed in the `node` macro.
#[derive(Debug)]
struct NodeArgs {
    /// Name of the struct of the node
    ///
    /// The run method will be implemented on this struct
    node_name: Ident,
    /// Names for how the type returned by `run` should be named
    output_names: OutputNames,
}

impl Parse for NodeArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let node_name: Ident = input.parse()?;
        let output_names = input.parse::<OutputNames>()?;

        Ok(Self {
            node_name,
            output_names,
        })
    }
}

/// Parsed list of output names
#[derive(Debug)]
enum OutputNames {
    /// Output names should be chosen automatically
    NotSpecified,
    /// The whole return type is bound to this name
    Single(Ident, Token![->]),
    /// Each element of the returned tuple are bound each of the names
    Tuple(Vec<Ident>, Token![->]),
}

impl Parse for OutputNames {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![->]) {
            let arrow_token: Token![->] = input.parse()?;

            let lookahead = input.lookahead1();
            if lookahead.peek(Ident) {
                Ok(Self::Single(input.parse()?, arrow_token))
            } else if lookahead.peek(token::Paren) {
                let content;
                parenthesized!(content in input);
                let names = content
                    .parse_terminated(Ident::parse, Token![,])?
                    .into_iter()
                    .collect();
                Ok(Self::Tuple(names, arrow_token))
            } else {
                Err(lookahead.error())
            }
        } else {
            Ok(Self::NotSpecified)
        }
    }
}

#[derive(Debug)]
struct InputArg {
    ident: Ident,
    base_type: Type,
}

#[derive(Debug)]
struct OutputArg {
    ident: Ident,
    base_type: Type,
}

#[proc_macro_attribute]
pub fn node(args: TokenStream, input: TokenStream) -> TokenStream {
    node_impl(args, input)
}

#[allow(clippy::too_many_lines)]
fn node_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let NodeArgs {
        node_name,
        output_names,
    } = parse_macro_input!(args as NodeArgs);

    let function = parse_macro_input!(input as ItemFn);
    let signature = function.sig.clone();

    if signature.ident != "run" {
        return Error::new_spanned(signature.ident, "node function must be named `run`")
            .to_compile_error()
            .into();
    }

    let mut input_args: Vec<InputArg> = vec![];

    // Check if the input parameters are correct
    let mut rec_found = false;
    for input in &signature.inputs {
        match input {
            FnArg::Receiver(rec) => {
                let Receiver {
                    reference,
                    mutability,
                    ..
                } = rec;
                match reference {
                    Some((_, Some(lifetime))) => {
                        return Error::new_spanned(
                            lifetime.ident.clone(),
                            "The first parameter of `run` must be `&self` without lifetimes specified",
                        )
                        .to_compile_error()
                        .into();
                    }
                    None => {
                        return Error::new_spanned(
                            rec.self_token,
                            "The first parameter of `run` must be `&self`",
                        )
                        .to_compile_error()
                        .into();
                    }
                    _ => {}
                }
                if let Some(m) = mutability {
                    return Error::new_spanned(
                        m,
                        "The first parameter of `run` must be `&self` without mutability",
                    )
                    .to_compile_error()
                    .into();
                }
                rec_found = true;
            }
            FnArg::Typed(pat_type) => {
                let PatType { pat, ty, .. } = pat_type;
                let base_type = match **ty {
                    Type::Reference(ref r) => {
                        let TypeReference {
                            and_token: _,
                            lifetime,
                            mutability,
                            elem,
                        } = r;
                        if lifetime.is_some() {
                            return Error::new_spanned(
                                r,
                                "All types must be a `&` without lifetime annotations",
                            )
                            .to_compile_error()
                            .into();
                        }
                        if mutability.is_some() {
                            return Error::new_spanned(
                                r,
                                "All input parameters mut be immutable references",
                            )
                            .to_compile_error()
                            .into();
                        }
                        *elem.clone()
                    }
                    _ => {
                        return Error::new_spanned(ty, "All input types must be behind a `&`")
                            .to_compile_error()
                            .into();
                    }
                };
                if let Pat::Ident(ident) = &**pat {
                    let mut arg_ident = ident.ident.clone();

                    // Remove leading underscore if present
                    if arg_ident.to_string().starts_with('_') {
                        arg_ident = format_ident!("{}", arg_ident.to_string()[1..]);
                    }

                    if input_args.iter().any(|arg| arg.ident == arg_ident) {
                        return Error::new_spanned(
                            ident,
                            "All input arguments must have a unique identifier",
                        )
                        .to_compile_error()
                        .into();
                    }
                    input_args.push(InputArg {
                        ident: arg_ident,
                        base_type,
                    });
                } else {
                    return Error::new_spanned(pat, "expected identifier")
                        .to_compile_error()
                        .into();
                }
            }
        }
    }

    if !rec_found {
        return Error::new_spanned(
            signature.ident,
            "The first parameter of `run` must be a `&self`",
        )
        .to_compile_error()
        .into();
    }

    let mut output_args: Vec<OutputArg> = vec![];

    // Check if the output types and names are correct
    match signature.output {
        ReturnType::Default => match output_names {
            OutputNames::NotSpecified => {}
            OutputNames::Single(_, token) | OutputNames::Tuple(_, token) => {
                return Error::new_spanned(
                    token,
                    "function does not return, but output names are specified",
                )
                .to_compile_error()
                .into();
            }
        },
        ReturnType::Type(_, output_type) => {
            if let Type::Tuple(tuple) = *output_type.clone() {
                let TypeTuple {
                    paren_token: _,
                    elems,
                } = tuple;
                match output_names {
                    OutputNames::NotSpecified => {
                        match elems.len() {
                            0 => {
                                // return type of '-> ()'
                            }
                            1 => {
                                return Error::new_spanned(node_name,
                                "ambiguous return type, use #[node({node_name} -> output)] (to return the whole tuple) or #[node({node_name} -> (output))] (to return the first element of the tuple)"
                                ).to_compile_error().into();
                            }
                            n => {
                                return Error::new_spanned(node_name.clone(),
                                    format!("specify return names for {n} elements: #[node({node_name} -> (output1, output2, ...))]")
                                ).to_compile_error().into();
                            }
                        }
                    }
                    OutputNames::Single(name, _) => {
                        output_args.push(OutputArg {
                            ident: name,
                            base_type: *output_type,
                        });
                    }
                    OutputNames::Tuple(names, token) => {
                        if names.len() == elems.len() {
                            for (name, ty) in names.iter().zip(elems.iter()) {
                                if output_args.iter().any(|arg| arg.ident == *name) {
                                    return Error::new_spanned(
                                        name,
                                        "All output arguments must have a unique identifier",
                                    )
                                    .to_compile_error()
                                    .into();
                                }
                                output_args.push(OutputArg {
                                    ident: name.clone(),
                                    base_type: ty.clone(),
                                });
                            }
                        } else {
                            return Error::new_spanned(
                                token,
                                format!(
                                    "function has {} return values, but {} names were specified",
                                    elems.len(),
                                    names.len()
                                ),
                            )
                            .to_compile_error()
                            .into();
                        }
                    }
                }
            } else {
                // The return type is any other type
                match output_names {
                    OutputNames::NotSpecified => {
                        // Use default name 'output'
                        output_args.push(OutputArg {
                            base_type: *output_type,
                            ident: format_ident!("output"),
                        });
                    }
                    OutputNames::Single(name, _) => {
                        output_args.push(OutputArg {
                            base_type: *output_type,
                            ident: name,
                        });
                    }
                    OutputNames::Tuple(_names, token) => {
                        return Error::new_spanned(
                        token,
                            "function has a single return type, use #[node(#node_name -> ...)] instead of #[node(#node_name -> (...))]",
                        )
                        .to_compile_error()
                        .into();
                    }
                }
            }
        }
    }

    let inputs_type_definitions: Vec<_> = input_args
        .iter()
        .map(|a| {
            let ident = a.ident.to_string();
            let in_type = a.base_type.clone();
            quote! {
                (#ident, ::core::any::TypeId::of::<#in_type>())
            }
        })
        .collect();
    let outputs_type_definitions: Vec<_> = output_args
        .iter()
        .map(|a| {
            let OutputArg {
                ident,
                base_type: ty,
            } = a;
            let ident = ident.to_string();
            quote! {
                (#ident, ::core::any::TypeId::of::<#ty>())
            }
        })
        .collect();

    let run_call_parameters = 0..input_args.len();

    let handle_name = format_ident!("{}Handle", node_name);
    let handle_input_ports = input_args.iter().map(|a| {
        let InputArg { ident, base_type } = a;
        let fn_ident = if *ident == "input" {
            ident.clone()
        } else {
            format_ident!("input_{}", ident)
        };
        let input_name = ident.to_string();
        quote! {
            pub fn #fn_ident(&self) -> ::computegraph::InputPort<#base_type> {
                ::computegraph::InputPort {
                    port_type: ::std::marker::PhantomData,
                    port: ::computegraph::InputPortUntyped {
                        node: self.handle.clone(),
                        input_name: #input_name,
                    },
                }
            }
        }
    });
    let handle_output_ports = output_args.iter().map(|o| {
        let OutputArg { ident, base_type } = o;
        let fn_ident = if *ident == "output" {
            ident.clone()
        } else {
            format_ident!("output_{}", ident)
        };
        let output_name = ident.to_string();
        quote! {
            pub fn #fn_ident(&self) -> ::computegraph::OutputPort<#base_type> {
                ::computegraph::OutputPort {
                    port_type: ::std::marker::PhantomData,
                    port: ::computegraph::OutputPortUntyped {
                        node: self.handle.clone(),
                        output_name: #output_name,
                    },
                }
            }
        }
    });
    let run_result_to_boxed = match handle_output_ports.len() {
        0 => quote!(),
        1 => quote!(::std::boxed::Box::new(res)),
        n => {
            let i = (0..n).map(syn::Index::from);
            quote! {
                #(::std::boxed::Box::new(res.#i)),*
            }
        }
    };

    quote! {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct #handle_name {
            pub handle: ::computegraph::NodeHandle
        }

        impl #handle_name {
            #(#handle_input_ports)*
            #(#handle_output_ports)*
        }

        impl Into<::computegraph::NodeHandle> for #handle_name {
            fn into(self) -> ::computegraph::NodeHandle {
                self.handle
            }
        }

        impl ::computegraph::NodeFactory for #node_name {
            type Handle = #handle_name;

            fn inputs() -> ::std::vec::Vec<(&'static str, ::core::any::TypeId)> {
                ::std::vec![
                    #(#inputs_type_definitions,)*
                ]
            }

            fn outputs() -> ::std::vec::Vec<(&'static str, ::core::any::TypeId)> {
                ::std::vec![
                    #(#outputs_type_definitions,)*
                ]
            }

            fn create_handle(gnode: &::computegraph::GraphNode) -> Self::Handle {
                Self::Handle {
                    handle: gnode.handle().clone(),
                }
            }
        }

        impl ::computegraph::ExecutableNode for #node_name {
            fn run(&self, input: &[&dyn ::std::any::Any]) -> Vec<::std::boxed::Box<dyn ::computegraph::SendSyncAny>> {
                let res = self.run(
                    #( input[#run_call_parameters].downcast_ref().unwrap() ),*
                );
                ::std::vec![
                    #run_result_to_boxed
                ]
            }
        }

        impl #node_name {
            #[allow(clippy::missing_const_for_fn)]
            #[allow(clippy::unused_self)]
            #[allow(clippy::trivially_copy_pass_by_ref)]
            #[allow(clippy::ptr_arg)]
            #function
        }
    }
    .into()
}
