mod common;
use common::*;

const PROBLEM: &str = "example/spj_problem_0.json";

#[test]
fn test_spj_0_with_ac() {
    let judge_task = generate_judge_task(SOURCE_AC, PROBLEM);
    let judge = Judge::new("test_special_judge_0_with_ac");
    judge.send_judge(&judge_task);
    for _i in 0..judge_task.problem.len() {
        let report = judge.receive_report();
        assert_report_with_limit(
            &report.into(),
            &judge_task.id,
            "AC",
            (1.0 * NS_PER_SEC) as u64,
            (32.0 * BYTES_PER_MB) as u64,
        );
    }
}

#[test]
fn test_spj_0_with_ce() {
    let judge_task = generate_judge_task(SOURCE_CE, PROBLEM);
    let judge = Judge::new("test_special_judge_0_with_ce");
    judge.send_judge(&judge_task);
    let report = judge.receive_report();
    assert_report_with_limit(
        &report.into(),
        &judge_task.id,
        "CE",
        (1.0 * NS_PER_SEC) as u64,
        (32.0 * BYTES_PER_MB) as u64,
    );
}

#[test]
fn test_spj_0_with_mle() {
    let judge_task = generate_judge_task(SOURCE_MLE, PROBLEM);
    let judge = Judge::new("test_special_judge_0_with_mle");
    judge.send_judge(&judge_task);
    for _i in 0..judge_task.problem.len() {
        let report = judge.receive_report();
        assert_report_with_limit(
            &report.into(),
            &judge_task.id,
            "MLE",
            (1.0 * NS_PER_SEC) as u64,
            (32.0 * MEMORY_EPS * BYTES_PER_MB) as u64,
        );
    }
}

#[test]
fn test_spj_0_with_re() {
    let judge_task = generate_judge_task(SOURCE_RE, PROBLEM);
    let judge = Judge::new("test_special_judge_0_with_re");
    judge.send_judge(&judge_task);
    for _i in 0..judge_task.problem.len() {
        let report = judge.receive_report();
        assert_report_with_limit(
            &report.into(),
            &judge_task.id,
            "RE",
            (1.0 * NS_PER_SEC) as u64,
            (32.0 * BYTES_PER_MB) as u64,
        );
    }
}

#[test]
fn test_spj_0_with_tle() {
    let judge_task = generate_judge_task(SOURCE_TLE, PROBLEM);
    let judge = Judge::new("test_special_judge_0_with_tle");
    judge.send_judge(&judge_task);
    for _i in 0..judge_task.problem.len() {
        let report = judge.receive_report();
        assert_report_with_limit(
            &report.into(),
            &judge_task.id,
            "TLE",
            (1.0 * TIME_EPS * NS_PER_SEC) as u64,
            (32.0 * BYTES_PER_MB) as u64,
        );
    }
}

#[test]
fn test_spj_0_with_wa() {
    let judge_task = generate_judge_task(SOURCE_WA, PROBLEM);
    let judge = Judge::new("test_special_judge_0_with_wa");
    judge.send_judge(&judge_task);
    for _i in 0..judge_task.problem.len() {
        let report = judge.receive_report();
        assert_report_with_limit(
            &report.into(),
            &judge_task.id,
            "WA",
            (1.0 * NS_PER_SEC) as u64,
            (32.0 * BYTES_PER_MB) as u64,
        );
    }
}
