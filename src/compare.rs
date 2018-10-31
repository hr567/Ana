pub fn compare(output: &str, answer: &str) -> bool {
        output.trim_end_matches('\n')
                .split_terminator('\n')
                .zip(answer.trim_end_matches('\n').split_terminator('\n'))
                .all(|(output, answer)| {
                        output.trim_end_matches(' ') == answer.trim_end_matches(' ')
                })
}
