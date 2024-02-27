use std::fmt::{Display, Formatter};

use rand::{Rng, RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::source::Source;

pub struct SeederConfig {
    number_of_groups: usize,
    probability_to_generate_garbage_between_groups: u64,
    max_probability_to_generate_failed_group: u64,
    max_probability_per_element_to_generate_garbage: u64,
    max_garbage_element_between_groups: u64,
    max_element_per_group_number: usize,
    debug: bool,
}

impl Display for SeederConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("groups = {}, max_group_element = {}, garbage_prob = {}%, failed_group_prob = {}%, garbage_element_prob = {}%, max_garbage_len = {} ",
                                 self.number_of_groups,
                                 self.max_element_per_group_number,
                                 self.probability_to_generate_garbage_between_groups,
                                 self.max_probability_to_generate_failed_group,
                                 self.max_probability_per_element_to_generate_garbage,
                                 self.max_garbage_element_between_groups))
    }
}

impl SeederConfig {
    pub fn new(
        number_of_groups: usize,
        probability_to_generate_garbage_between_groups: u64,
        max_probability_to_generate_failed_group: u64,
        max_probability_per_element_to_generate_garbage: u64,
        max_garbage_element_between_groups: u64,
        max_element_per_group_number: usize,
        debug: bool,
    ) -> Self {
        SeederConfig {
            number_of_groups,
            probability_to_generate_garbage_between_groups,
            max_probability_to_generate_failed_group,
            max_probability_per_element_to_generate_garbage,
            max_garbage_element_between_groups,
            max_element_per_group_number,
            debug,
        }
    }

    pub fn generate(&self, rng: &mut ChaCha8Rng) -> (Vec<u8>, Vec<Vec<u8>>) {
        generate_groups_data(
            rng,
            self.number_of_groups,
            self.probability_to_generate_garbage_between_groups,
            self.max_probability_to_generate_failed_group,
            self.max_probability_per_element_to_generate_garbage,
            self.max_garbage_element_between_groups,
            self.max_element_per_group_number,
            self.debug,
        )
    }
}

fn generate_garbage_between_groups(
    amount_of_garbage: usize,
    rng: &mut ChaCha8Rng,
    _debug: bool,
) -> Vec<u8> {
    (0..amount_of_garbage).fold(vec![], |mut acc, _| {
        let mut garbage = rng.gen_range(32..126) as u8;

        if (47..=57).contains(&garbage) {
            garbage += 11;
        }

        acc.push(garbage);
        acc
    })
}

#[allow(clippy::too_many_arguments)]
pub fn generate_groups_data(
    rng: &mut ChaCha8Rng,
    number_of_groups: usize,
    probability_to_generate_garbage_between_groups: u64,
    max_probability_to_generate_failed_group: u64,
    max_probability_per_element_to_generate_garbage: u64,
    max_garbage_element_between_groups: u64,
    max_element_per_group_number: usize,
    debug: bool,
) -> (Vec<u8>, Vec<Vec<u8>>) {
    let mut result_data = vec![];

    if bool_with_prob(probability_to_generate_garbage_between_groups, rng) {
        let number_of_garbage_between_groups =
            rng.gen_range(0..max_garbage_element_between_groups) as usize;
        let garbage = generate_garbage_between_groups(number_of_garbage_between_groups, rng, debug);
        result_data.extend_from_slice(&garbage);
    }

    let mut expected = vec![];

    for _ in 0..number_of_groups {
        let element_per_group = rng.gen_range(0..max_element_per_group_number);
        let probability_of_failed_group =
            rng.gen_range(0..max_probability_to_generate_failed_group);
        let probability_of_failed_element_in_group =
            rng.gen_range(0..max_probability_per_element_to_generate_garbage);

        let (group, noise, expected_group_value) = generate_group_data(
            rng,
            element_per_group,
            probability_of_failed_group,
            probability_of_failed_element_in_group,
            debug,
        );

        result_data.extend_from_slice(&group);

        if !noise && !expected_group_value.is_empty() {
            if debug {
                println!("Push {:?}", expected_group_value);
            }
            expected.push(expected_group_value)
        }

        if bool_with_prob(probability_to_generate_garbage_between_groups, rng) {
            let number_of_garbage_between_groups =
                rng.gen_range(0..max_garbage_element_between_groups) as usize;
            let garbage =
                generate_garbage_between_groups(number_of_garbage_between_groups, rng, debug);
            if debug {
                dbg!(debug!(&garbage));
            }
            result_data.extend_from_slice(&garbage);
        }
    }
    (result_data, expected)
}

fn generate_group_data(
    rng: &mut ChaCha8Rng,
    element_per_group_number: usize,
    probability_to_generate_failed_group: u64,
    probability_per_element_to_generate_garbage: u64,
    debug: bool,
) -> (Vec<u8>, bool, Vec<u8>) {
    #[derive(Debug, Clone)]
    enum Value {
        Number(u8),
        Garbage(char),
    }

    impl Value {
        fn to_bytes(&self) -> Vec<u8> {
            match self {
                Value::Number(number) => number_to_ascii(number),
                Value::Garbage(character) => vec![*character as u8],
            }
        }

        fn to_number(&self) -> Option<u8> {
            if let Value::Number(number) = self {
                Some(*number)
            } else {
                None
            }
        }
    }

    let mut group_elements = (0..element_per_group_number).fold(vec![], |mut acc, _| {
        acc.push(Value::Number(rng.gen_range(0..99) as u8));
        acc
    });

    let mut noise = false;
    let expected = group_elements.iter().flat_map(|x| x.to_number()).collect();

    // Generate noise into group
    if bool_with_prob(probability_to_generate_failed_group, rng) {
        let index_to_replace = (0..element_per_group_number).fold(vec![], |mut acc, index| {
            if bool_with_prob(probability_per_element_to_generate_garbage, rng) {
                noise = true;
                acc.push(index)
            }
            acc
        });

        for index in index_to_replace {
            let mut garbage = rng.gen_range(97..=122) as u8;

            if (47..=57).contains(&garbage) {
                garbage += 11;
            }

            group_elements[index] = Value::Garbage(garbage as char);
        }
    }

    if debug {
        dbg!(&group_elements);
        dbg!(noise);
    }

    let mut result_data: Vec<u8> = vec![];

    result_data.push(b'(');
    for part in group_elements {
        let binary_repr = part.to_bytes();
        result_data.extend_from_slice(&binary_repr);
        result_data.push(b',')
    }
    result_data.pop();
    result_data.push(b')');

    (result_data, noise, expected)
}

/// Generate a true value with the defined probability
fn bool_with_prob(prob: u64, rng: &mut ChaCha8Rng) -> bool {
    let random = (rng.next_u64() % 100) + 1;
    random <= prob
}

pub fn source_data<F>(config: &SeederConfig, seed: u64, chunk_size: usize, mut closure: F)
where
    F: FnMut(Source),
{
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let (data, _) = config.generate(&mut rng);
    closure(Source::new(&data).with_chunk_size(chunk_size))
}

fn number_to_ascii(num: &u8) -> Vec<u8> {
    let num_str = format!("{}", num);
    num_str.bytes().collect::<Vec<u8>>()
}

#[test]
fn test_number_to_ascii() {
    let result = number_to_ascii(&45);
    dbg!(debug!(&result));
    assert_eq!(vec![52, 53], result);
}
