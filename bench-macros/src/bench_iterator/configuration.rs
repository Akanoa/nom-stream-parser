use proc_macro2::{Ident, TokenStream, TokenTree};
use syn_helpers::syn::parse::{Parse, ParseStream, Peek};
use syn_helpers::syn::punctuated::Punctuated;
use syn_helpers::syn::{Expr, LitInt, Token};
use syn_helpers::Argument;

pub struct BenchesConfigurationIterator {
    pub name: Ident,
    pub seed: u64,
    pub parser: Expr,
    pub chunk_size: ChunkSizeList,
    pub buffer: Expr,
    pub config: Expr,
}

impl Parse for BenchesConfigurationIterator {
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
        let Argument(name, chunk_sizes) = input.parse::<Argument<ChunkSizeList>>()?;
        if &name != "chunk_sizes" {
            return Err(syn_helpers::syn::Error::new(
                input.span(),
                format!("Expected chunk_sizes = ; found {} = ", name),
            ));
        }

        Ok(BenchesConfigurationIterator {
            name: bench_name,
            seed: seed.base10_parse()?,
            parser,
            buffer,
            chunk_size: chunk_sizes,
            config: configuration,
        })
    }
}

pub struct ChunkSizeList {
    pub sizes: Vec<u64>,
}

impl Parse for ChunkSizeList {
    fn parse(input: ParseStream) -> syn_helpers::syn::Result<Self> {
        let mut segments: Punctuated<LitInt, Token![,]> = Punctuated::new();

        let first = parse_until(input, Token![,])?;
        segments.push_value(syn_helpers::syn::parse2(first)?);

        while input.peek(Token![,]) {
            segments.push_punct(input.parse()?);

            let next = parse_until(input, Token![,])?;
            segments.push_value(syn_helpers::syn::parse2(next)?);
        }

        let sizes = segments
            .into_iter()
            .map(|segment| segment.base10_parse::<u64>())
            .collect::<syn_helpers::syn::Result<Vec<u64>>>()?;

        Ok(ChunkSizeList { sizes })
    }
}

fn parse_until<E: Peek>(input: ParseStream, end: E) -> syn_helpers::syn::Result<TokenStream> {
    let mut tokens = TokenStream::new();
    while !input.is_empty() && !input.peek(end) {
        let next: TokenTree = input.parse()?;
        tokens.extend(Some(next));
    }
    Ok(tokens)
}
