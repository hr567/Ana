mod common;
use common::*;

use std::time::Duration;

use liboj::structures::Report;

const PROBLEM: &str = "example_data/spj_problem_1.json";

#[test]
fn test_spj_0_with_ac() {
    let task = generate_judge_task(SOURCE_AC, PROBLEM);
    let judge = Judge::new();
    for report in judge.judge(task) {
        match report {
            Report::Accepted {
                resource_usage: usage,
            } => {
                assert!(usage.real_time < Duration::from_secs(1) * 2);
                assert!(usage.cpu_time < Duration::from_secs(1));
                assert!(usage.memory < 32 * BYTES_PER_MB);
            }
            res => panic!("Wrong judge result: {}", res),
        }
    }
}

#[test]
fn test_spj_0_with_ce() {
    let task = generate_judge_task(SOURCE_CE, PROBLEM);
    let judge = Judge::new();
    for report in judge.judge(task) {
        match report {
            Report::CompileError => {}
            res => panic!("Wrong judge result: {}", res),
        }
    }
}

#[test]
fn test_spj_0_with_mle() {
    let task = generate_judge_task(SOURCE_MLE, PROBLEM);
    let judge = Judge::new();
    for report in judge.judge(task) {
        match report {
            Report::MemoryLimitExceeded => {}
            res => panic!("Wrong judge result: {}", res),
        }
    }
}

#[test]
fn test_spj_0_with_re() {
    let task = generate_judge_task(SOURCE_RE, PROBLEM);
    let judge = Judge::new();
    for report in judge.judge(task) {
        match report {
            Report::RuntimeError => {}
            res => panic!("Wrong judge result: {}", res),
        }
    }
}

#[test]
fn test_spj_0_with_tle() {
    let task = generate_judge_task(SOURCE_TLE, PROBLEM);
    let judge = Judge::new();
    for report in judge.judge(task) {
        match report {
            Report::TimeLimitExceeded => {}
            res => panic!("Wrong judge result: {}", res),
        }
    }
}

#[test]
fn test_spj_0_with_wa() {
    let task = generate_judge_task(SOURCE_WA, PROBLEM);
    let judge = Judge::new();
    for report in judge.judge(task) {
        match report {
            Report::WrongAnswer => {}
            res => panic!("Wrong judge result: {}", res),
        }
    }
}
