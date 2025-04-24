use std::io::Read;
use anchor_syn::*;
use syn::*;
use quote::*;


#[proc_macro_attribute]
pub fn modularized_program(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let ProgramMacro { programs } = syn::parse_macro_input!(args as ProgramMacro);

    let fns = programs.into_iter()
        .map(|(path, p)| {
            p.ixs.into_iter().map(|ix| build_relay(&path, ix)).collect::<Vec<_>>()
        })
        .flatten().collect();

    let input = insert_fns_into_first_module(input, fns);
    let program = syn::parse_macro_input!(input as anchor_syn::Program);
    program.to_token_stream().into()
}

fn build_relay(path: &Path, ix: Ix) -> ItemFn {
    let item_fn = &ix.raw_method;
    let Signature { ident: fn_name, inputs, generics, output, .. } = &item_fn.sig;

    let first = path.segments[0].clone().ident;
    let new_name = Ident::new(format!("{}_{}", first, fn_name).as_str(), first.span());

    // Extract argument names for the function call
    let arg_names: Vec<Ident> = inputs
        .iter()
        .filter_map(|arg| match arg { FnArg::Typed(pt) => Some(&*pt.pat), _ => None })
        .filter_map(|pt| match pt { syn::Pat::Ident(id) => Some(id.ident.clone()), _ => None })
        .collect();

    parse_quote! {
        pub fn #new_name #generics(#inputs) #output {
            #path::#fn_name(#(#arg_names),*)
        }
    }
}


#[derive(Debug)]
struct ProgramMacro { programs: Vec<(Path, Program)>, }

// Implement parsing for the ProgramMacro struct
impl parse::Parse for ProgramMacro {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {

        // Parse `modules`
        let modules_ident: Ident = input.parse()?;
        if modules_ident != "modules" {
            return Err(syn::Error::new(modules_ident.span(), "expected `modules`"));
        }

        input.parse::<Token![=]>()?;

        // Parse the bracketed list `[cell, placement]`
        let content;
        syn::bracketed!(content in input);
        let idents: syn::punctuated::Punctuated<Path, Token![,]> =
            content.parse_terminated(|p| p.parse::<Path>())?;

        // Convert Punctuated<Ident, _> to Vec<Ident>
        let programs = idents.into_iter()
            .map(|path| (path.clone(), resolve_module(path)))
            .collect();

        Ok(ProgramMacro { programs })
    }
}

fn resolve_module(path: Path) -> Program {
    let mod_path = format!(
        "{}/src/{}.rs",
        std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default(),
        path.segments.into_pairs().fold(String::new(), |s, p| format!("{}/{}", s, p.value().ident))
    );
    let mut code_str = String::new();
    std::fs::File::open(mod_path).unwrap().read_to_string(&mut code_str).unwrap();
    let parsed = syn::parse_file(&code_str).unwrap();

    let program_mod = ItemMod {
        vis: Visibility::Public(VisPublic { pub_token: Default::default() }),
        attrs: vec![],
        mod_token: syn::token::Mod::default(),
        ident: Ident::new("abc", proc_macro2::Span::call_site()),
        content: Some((
            syn::token::Brace::default(),
            parsed.items,
        )),
        semi: None,
    };
    let program = anchor_syn::parser::program::parse(program_mod).unwrap();
    assert!(program.fallback_fn.is_none(), "additional program module cant have fallback");
    program
}


fn insert_fns_into_first_module(input: proc_macro::TokenStream, fns: Vec<ItemFn>) -> proc_macro::TokenStream {
    // Convert proc_macro::TokenStream to proc_macro2::TokenStream for parsing
    let input_stream: proc_macro2::TokenStream = input.into();
    let fn_items = fns.into_iter().map(Item::Fn).collect::<Vec<_>>();

    // Parse the input into a syn::File
    let mut item_mod: ItemMod = parse2(input_stream).unwrap();
    if let Some((_, content)) = &mut item_mod.content {
        // Module has a body; append functions to its content
        content.extend(fn_items);
    } else {
        panic!("ono!");
    }

    // Convert the modified File back to a proc_macro::TokenStream
    let output_stream: proc_macro2::TokenStream = quote! { #item_mod };
    output_stream.into()
}
