use std::io::Read;
use anchor_syn::*;
use syn::*;
use quote::*;


/*
 * Replacement program macro to include additional instruction modules, i.e.:
 *
 * #[modularized_program(
 *     modules=[
 *         foo::instructions,
 *         bar::instructions
 *     ]
 * )]
 * pub mod my_program {
 *     use super::*;
 * }
 */

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


/*
 * This builds relay instructions, i.e, for `foo::instructions::do_thing`:
 *
 * pub fn foo_do_thing(ctx: Context<YourInstructionContext>, ...) -> Result<()> {
 *     foo::instructions::do_thing(ctx, ...)
 * }
 */

fn build_relay(path: &Path, ix: Ix) -> ItemFn {
    //ix.cfgs
    let item_fn = &ix.raw_method;
    let ItemFn { attrs, .. } = &item_fn;
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
        #(#attrs)*
        pub fn #new_name #generics(#inputs) #output {
            #path::#fn_name(#(#arg_names),*)
        }
    }
}


/*
 * Get an anchor Program from the given path, by parsing the file, i.e.
 * foo::instructions is converted to "$PROGRAM_DIR/src/foo/instructions.rs"
 */

fn resolve_module(path: Path) -> Program {

    let mod_path = format!(
        "{}/src{}.rs",
        std::env::var("CARGO_MANIFEST_DIR").unwrap(),
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



/*
 * Parse the macro arguments
 */

#[derive(Debug)]
struct ProgramMacro { programs: Vec<(Path, Program)>, }

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


/*
 * Append instruction functions to main program module
 */

fn insert_fns_into_first_module(input: proc_macro::TokenStream, fns: Vec<ItemFn>) -> proc_macro::TokenStream {

    let mut item_mod: ItemMod = parse2(input.into()).expect("Failed to parse main program module");

    item_mod.content
        .as_mut()
        .expect("Program module has no body?")
        .1.extend(fns.into_iter().map(Into::into));

    quote! { #item_mod }.into()
}


