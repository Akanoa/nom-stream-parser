use nom::{
    bytes::streaming::tag, character, character::streaming::digit1, combinator::map_parser,
    multi::separated_list1, sequence::delimited, IResult,
};

fn parse_digit(input: &[u8]) -> IResult<&[u8], u8> {
    let (remain, result) = map_parser(digit1, character::complete::u8)(input)?;
    Ok((remain, result))
}

pub fn parse_data(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    delimited(tag("("), separated_list1(tag(","), parse_digit), tag(")"))(input)
}

pub fn start_group_parenthesis(input: &[u8]) -> IResult<&[u8], &[u8]> {
    nom::bytes::streaming::take_until("(")(input)
}
