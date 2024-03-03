use proc_macro2::Ident;
use syn_helpers::syn::parse::{Parse, ParseStream};
use syn_helpers::syn::{Expr, LitInt, Token};
use syn_helpers::Argument;

pub struct BenchesConfigurationReader {
    pub name: Ident,
    pub seed: u64,
    pub parser: Expr,
    pub buffer: Expr,
    pub config: Expr,
}

impl Parse for BenchesConfigurationReader {
    fn parse(input: ParseStream) -> syn_helpers::syn::Result<Self> {
        let Argument(name, bench_name) = input.parse::<Argument<Ident>>()?;
        if &name != "name" {
            return Err(syn_helpers::syn::Error::new(
                input.span(),
                format!("Expected name = ; found {} = ", name),
            ));
        }
        input.parse::<Token![;]>()?;
        let Argument(name, configuration) = input.parse::<Argument<Expr>>()?;
        if &name != "config" {
            return Err(syn_helpers::syn::Error::new(
                input.span(),
                format!("Expected config = ; found {} = ", name),
            ));
        }
        input.parse::<Token![;]>()?;
        let Argument(name, seed) = input.parse::<Argument<LitInt>>()?;
        if &name != "seed" {
            return Err(syn_helpers::syn::Error::new(
                input.span(),
                format!("Expected seed = ; found {} = ", name),
            ));
        }
        input.parse::<Token![;]>()?;
        let Argument(name, parser) = input.parse::<Argument<Expr>>()?;
        if &name != "parser" {
            return Err(syn_helpers::syn::Error::new(
                input.span(),
                format!("Expected parser = ; found {} = ", name),
            ));
        }
        input.parse::<Token![;]>()?;
        let Argument(name, buffer) = input.parse::<Argument<Expr>>()?;
        if &name != "buffer" {
            return Err(syn_helpers::syn::Error::new(
                input.span(),
                format!("Expected buffer = ; found {} = ", name),
            ));
        }
        input.parse::<Token![;]>()?;

        Ok(BenchesConfigurationReader {
            name: bench_name,
            seed: seed.base10_parse()?,
            parser,
            buffer,
            config: configuration,
        })
    }
}
