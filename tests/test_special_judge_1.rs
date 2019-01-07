use std::io;

use serde_json;

mod common;
use self::common::*;

#[test]
fn test_spj_1_with_ac() -> io::Result<()> {
    let judge_info = common::generate_judge_info(
        "example/source.cpp",
        "example/spj_problem.json",
        Some("example/spj.1.cpp"),
    )?;
    let judge = Judge::new("test_special_judge_1_with_ac");
    judge.send_judge_info(&judge_info);
    for i in 0..=judge_info.problem.len() {
        let report = judge.receive_report();
        assert_report_with_limit(&report.into(), &judge_info.id, i, "AC", 1.0, 32.0);
    }
    Ok(())
}

#[test]
fn test_spj_1_with_ce() -> io::Result<()> {
    let judge_info = common::generate_judge_info(
        "example/source.ce.cpp",
        "example/spj_problem.json",
        Some("example/spj.1.cpp"),
    )?;
    let judge = Judge::new("test_special_judge_1_with_ce");
    judge.send_judge_info(&judge_info);
    let report = judge.receive_report();
    assert_report_with_limit(&report.into(), &judge_info.id, 0, "CE", 1.0, 32.0);
    Ok(())
}

#[test]
fn test_spj_1_with_mle() -> io::Result<()> {
    let judge_info = common::generate_judge_info(
        "example/source.mle.cpp",
        "example/spj_problem.json",
        Some("example/spj.1.cpp"),
    )?;
    let judge = Judge::new("test_special_judge_1_with_mle");
    judge.send_judge_info(&judge_info);
    for i in 0..=judge_info.problem.len() {
        let report = judge.receive_report();
        assert_report_with_limit(&report.into(), &judge_info.id, i, "MLE", 1.0, 32.0);
    }
    Ok(())
}

#[test]
fn test_spj_1_with_re() -> io::Result<()> {
    let judge_info = common::generate_judge_info(
        "example/source.re.cpp",
        "example/spj_problem.json",
        Some("example/spj.1.cpp"),
    )?;
    let judge = Judge::new("test_special_judge_1_with_re");
    judge.send_judge_info(&judge_info);
    for i in 0..=judge_info.problem.len() {
        let report = judge.receive_report();
        assert_report_with_limit(&report.into(), &judge_info.id, i, "RE", 1.0, 32.0);
    }
    Ok(())
}

#[test]
fn test_spj_1_with_tle() -> io::Result<()> {
    let judge_info = common::generate_judge_info(
        "example/source.tle.cpp",
        "example/spj_problem.json",
        Some("example/spj.1.cpp"),
    )?;
    let judge = Judge::new("test_special_judge_1_with_tle");
    judge.send_judge_info(&judge_info);
    for i in 0..=judge_info.problem.len() {
        let report = judge.receive_report();
        assert_report_with_limit(&report.into(), &judge_info.id, i, "tle", 1.0, 32.0);
    }
    Ok(())
}

#[test]
fn test_spj_1_with_wa() -> io::Result<()> {
    let judge_info = common::generate_judge_info(
        "example/source.wa.cpp",
        "example/spj_problem.json",
        Some("example/spj.1.cpp"),
    )?;
    let judge = Judge::new("test_special_judge_1_with_wa");
    judge.send_judge_info(&judge_info);
    for i in 0..=judge_info.problem.len() {
        let report = judge.receive_report();
        assert_report_with_limit(&report.into(), &judge_info.id, i, "wa", 1.0, 32.0);
    }
    Ok(())
}
