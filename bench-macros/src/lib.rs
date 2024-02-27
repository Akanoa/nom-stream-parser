use crate::bench_configuration::BenchesConfiguration;
use syn_helpers::format_ident;
use syn_helpers::quote;
use syn_helpers::syn::parse_macro_input;

mod bench_configuration;

#[proc_macro]
pub fn generate_bench(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    impl_generate_bench(input)
}

fn impl_generate_bench(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let bench_configs = parse_macro_input!(input as BenchesConfiguration);
    let bench_name = &bench_configs.name;
    let bench_name_string = bench_name.to_string();
    let config = &bench_configs.config;
    let seed = bench_configs.seed;
    let parser = bench_configs.parser;
    let buffer = bench_configs.buffer;

    let mut tokens = quote!();

    for size in &bench_configs.chunk_size.sizes {
        let fn_name = format_ident!("{}_{}_{}", bench_name_string, "chunk", size);
        let token_stream = quote! {
            fn #fn_name(c: &mut Criterion) {
        let config = #config;
        source_data(&config, #seed, #size as usize, |source| {
            let mut save_buffer = #buffer;
            let mut work_buffer = #buffer;
            c.bench_function(
                &format!("{}_chunk_{} data_size={} seed={} config={}", #bench_name_string,  #size, source.get_len(), #seed, &config),
                |c| {
                    c.iter(|| #parser(black_box(source.clone()), &mut save_buffer, &mut work_buffer))
                },
            );
        });
        }};
        tokens = quote! {
            #tokens
            #token_stream
        }
    }

    let mut benches = quote!();
    for size in bench_configs.chunk_size.sizes {
        let fn_name = format_ident!("{}_{}_{}", bench_name_string, "chunk", size);
        benches = quote! {
            #benches
            #fn_name,
        }
    }

    let criterion_group = quote! {
        criterion_group!(
            #bench_name,
            #benches
        );
    };

    tokens = quote! {
        #tokens
        #criterion_group
    };

    tokens.into()
}
