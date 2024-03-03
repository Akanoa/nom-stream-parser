use quote::{format_ident, quote};
use syn_helpers::syn::parse_macro_input;

use configuration::BenchesConfigurationReader;

pub mod configuration;

pub fn impl_generate_bench(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let bench_configs = parse_macro_input!(input as BenchesConfigurationReader);
    let bench_name = &bench_configs.name;
    let bench_name_string = bench_name.to_string();
    let config = &bench_configs.config;
    let seed = bench_configs.seed;
    let parser = bench_configs.parser;
    let buffer = bench_configs.buffer;

    let fn_name = format_ident!("bench_{}", bench_name_string);
    let tokens = quote! {
        fn #fn_name(c: &mut Criterion) {
    let config = #config;
    raw_data(&config, #seed, |source| {
        let mut work_buffer = #buffer;
        c.bench_function(
            &format!("{} data_size={} seed={} config={}", #bench_name_string, source.len(), #seed, &config),
            |c| {
                c.iter(|| #parser(black_box(source.clone()), &mut work_buffer))
            },
        );
    });
    }};

    let criterion_group = quote! {
        criterion_group!(
            #bench_name,
            #fn_name
        );
    };

    let tokens = quote! {
        #tokens
        #criterion_group
    };

    tokens.into()
}
