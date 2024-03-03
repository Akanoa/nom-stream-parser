# Nom Stream Parser

## Usage

## Streaming mode

The [nom](https://docs.rs/nom/latest/nom/) crate is a parser combinator which allow to parse
various kind of data.

One of its ability apart its zero-copy promise, is to be able
to parse a stream of data.

Let's take an example.

You want to parse something like this

`
(1,2,3,45)
`

A comma separated list of numbers delimited by parentheses.

With `nom`, you'll write a parse like this.

```ignore
use nom;
fn parser(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    delimited(
        tag("("),
        separated_list1(
            bytes::complete::tag(","),
            map_parser(character::complete::digit1, character::complete::u8),
        ),
        tag(")"),
    )(input)
}
```

As you can see I use the `complete` version of `tag` and `digit1` parsers.

This means, that this parser has two return value possible, either success or fail.

But if the input data is something like this ?

`
(1,2,
`

This kind of data can happen when input is chunked, like when reading a socket.

Is this an error or something incomplete ?

That why **nom** provides the streaming mode

Here the same parser but using the streaming version.

```ignore
use nom;
fn parser(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    delimited(
        tag("("),
        separated_list1(
            bytes::streaming::tag(","),
            map_parser(character::streaming::digit1, character::complete::u8),
        ),
        tag(")"),
    )(input)
}
```

Now there is 3 alternatives:

- A success
- An error
- An incomplete data, the parser can't take a decision, he needs more data

As the data are `(1,2,`, the parser return an Incomplete state.

So to get a final decision we need to concatenate `(1,2,` and `3,45)`.

Finally, we get the parsing decision.

## Noising

While this example can seem simple, this action of accumulating data can
become tedious when data are chunked in more than two parts.

And becomes even harder when some data in the buffering process aren't relevant
data but noise.

```ignore
(1,2,3,45)noise(1,2)
```

This data may be chuked as

`(1,2,`   `3,45)noi`  `se(1,2)`

This noise must be evinced.

## Buffering

One of the others problematics is how to handle the buffer growing.

What is the right size of the buffer, is it allows to grow or must
be preallocated and fix sized during the parse.

That's a lot of question when we need to parse incomplete data.

## Downstream parsing results

The stream of data can be infinite, by definition, with no end

*At which moment do we can and must release parsing results ?*

We don't know, and moreover, we need to accumulate all these results.

The solution is to downstream results as the data yields by upstream
data stream are sufficient for yielding a parse result.

This way we don't need to accumulate all results, just the current one.

```ignore
[data stream] ----{data chunk}---->   Parser    ---> [result stream]
   upstream                          buffering         downstream
```

## API

So we want something which takes an [Iterator] or a [std::io::Read] and return another [Iterator].