mod bench_iterator;
mod bench_reader;

#[proc_macro]
pub fn generate_bench_iterator(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    bench_iterator::impl_generate_bench(input)
}

#[proc_macro]
pub fn generate_bench_reader(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    bench_reader::impl_generate_bench(input)
}
