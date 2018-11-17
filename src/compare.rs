pub enum CompareResult {
    AC,
    WA,
}

pub fn compare(output: &str, answer: &str) -> CompareResult {
    if output
        .trim_end_matches('\n')
        .split_terminator('\n')
        .zip(answer.trim_end_matches('\n').split_terminator('\n'))
        .all(|(output, answer)| output.trim_end_matches(' ') == answer.trim_end_matches(' '))
    {
        CompareResult::AC
    } else {
        CompareResult::WA
    }
}
