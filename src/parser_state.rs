/// Define the decision of the master parser at previous iteration
#[derive(Debug)]
pub enum ParsableState {
    /// The data in the work aren't enough to decide the parsing state
    NeedMoreData,
    /// The data in working buffer may lead to parsing decision
    MaybeParsable,
}

/// Command whether the search start group must be run
#[derive(Debug)]
pub enum SearchState {
    /// We are still searching for relevant data to parse
    SearchForStart,
    /// The start of a relevant data to parse have found
    StartFound,
}
