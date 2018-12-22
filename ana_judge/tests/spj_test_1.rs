use std::io;

use serde_json;

use ana_mtp as mtp;

mod common;
use self::common::{check_report_with_limit, Communicator, Container};

#[test]
fn test_spj_1_judge_with_ac() -> io::Result<()> {
    let ana = Container::new()?;
    let communicator = Communicator::new(&ana.ip_address()?)?;
    let judge_info = common::generate_judge_info(
        "example/source.cpp",
        "example/spj_problem.json",
        Some("example/spj.1.cpp"),
    )?;
    communicator.send(&serde_json::to_string(&judge_info)?)?;

    for i in 0..judge_info.problem.len() {
        let report = communicator.receive()?;
        let report: mtp::ReportInfo = serde_json::from_str(&report)?;
        check_report_with_limit(&report, &judge_info.id, i, "AC", 1.0, 32.0);
    }

    let report = communicator.receive()?;
    let report: mtp::ReportInfo = serde_json::from_str(&report)?;
    check_report_with_limit(
        &report,
        &judge_info.id,
        judge_info.problem.len(),
        "AC",
        1.0,
        32.0,
    );

    Ok(())
}

#[test]
fn test_spj_1_judge_with_ce() -> io::Result<()> {
    let ana = Container::new()?;
    let communicator = Communicator::new(&ana.ip_address()?)?;
    let judge_info = common::generate_judge_info(
        "example/source.ce.cpp",
        "example/spj_problem.json",
        Some("example/spj.1.cpp"),
    )?;
    communicator.send(&serde_json::to_string(&judge_info)?)?;

    let report = communicator.receive()?;
    let report: mtp::ReportInfo = serde_json::from_str(&report)?;
    check_report_with_limit(&report, &judge_info.id, 0, "CE", 1.0, 32.0);

    Ok(())
}

#[test]
fn test_spj_1_judge_with_mle() -> io::Result<()> {
    let ana = Container::new()?;
    let communicator = Communicator::new(&ana.ip_address()?)?;
    let judge_info = common::generate_judge_info(
        "example/source.mle.cpp",
        "example/spj_problem.json",
        Some("example/spj.1.cpp"),
    )?;
    communicator.send(&serde_json::to_string(&judge_info)?)?;

    for i in 0..judge_info.problem.len() {
        let report = communicator.receive()?;
        let report: mtp::ReportInfo = serde_json::from_str(&report)?;
        check_report_with_limit(&report, &judge_info.id, i, "MLE", 1.0, 32.0);
    }

    let report = communicator.receive()?;
    let report: mtp::ReportInfo = serde_json::from_str(&report)?;
    check_report_with_limit(
        &report,
        &judge_info.id,
        judge_info.problem.len(),
        "MLE",
        1.0,
        32.0,
    );

    Ok(())
}

#[test]
fn test_spj_1_judge_with_re() -> io::Result<()> {
    let ana = Container::new()?;
    let communicator = Communicator::new(&ana.ip_address()?)?;
    let judge_info = common::generate_judge_info(
        "example/source.re.cpp",
        "example/spj_problem.json",
        Some("example/spj.1.cpp"),
    )?;
    communicator.send(&serde_json::to_string(&judge_info)?)?;

    for i in 0..judge_info.problem.len() {
        let report = communicator.receive()?;
        let report: mtp::ReportInfo = serde_json::from_str(&report)?;
        check_report_with_limit(&report, &judge_info.id, i, "RE", 1.0, 32.0);
    }

    let report = communicator.receive()?;
    let report: mtp::ReportInfo = serde_json::from_str(&report)?;
    check_report_with_limit(
        &report,
        &judge_info.id,
        judge_info.problem.len(),
        "RE",
        1.0,
        32.0,
    );

    Ok(())
}

#[test]
fn test_spj_1_judge_with_tle() -> io::Result<()> {
    let ana = Container::new()?;
    let communicator = Communicator::new(&ana.ip_address()?)?;
    let judge_info = common::generate_judge_info(
        "example/source.tle.cpp",
        "example/spj_problem.json",
        Some("example/spj.1.cpp"),
    )?;
    communicator.send(&serde_json::to_string(&judge_info)?)?;

    for i in 0..judge_info.problem.len() {
        let report = communicator.receive()?;
        let report: mtp::ReportInfo = serde_json::from_str(&report)?;
        check_report_with_limit(&report, &judge_info.id, i, "TLE", 1.0, 32.0);
    }

    let report = communicator.receive()?;
    let report: mtp::ReportInfo = serde_json::from_str(&report)?;
    check_report_with_limit(
        &report,
        &judge_info.id,
        judge_info.problem.len(),
        "TLE",
        1.0,
        32.0,
    );

    Ok(())
}

#[test]
fn test_spj_1_judge_with_wa() -> io::Result<()> {
    let ana = Container::new()?;
    let communicator = Communicator::new(&ana.ip_address()?)?;
    let judge_info = common::generate_judge_info(
        "example/source.wa.cpp",
        "example/spj_problem.json",
        Some("example/spj.1.cpp"),
    )?;
    communicator.send(&serde_json::to_string(&judge_info)?)?;

    for i in 0..judge_info.problem.len() {
        let report = communicator.receive()?;
        let report: mtp::ReportInfo = serde_json::from_str(&report)?;
        check_report_with_limit(&report, &judge_info.id, i, "WA", 1.0, 32.0);
    }

    let report = communicator.receive()?;
    let report: mtp::ReportInfo = serde_json::from_str(&report)?;
    check_report_with_limit(
        &report,
        &judge_info.id,
        judge_info.problem.len(),
        "WA",
        1.0,
        32.0,
    );

    Ok(())
}
