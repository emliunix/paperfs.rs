use anyhow::{bail, Context, Result};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
struct GoldCase {
    case_id: String,
    expected_answer: String,
}

#[derive(Debug, Clone)]
struct DecisionCitation {
    human_commit: String,
    decision_path: String,
    deciding_sentence: String,
    second_reviewer: String,
}

fn by_id(cases: &[GoldCase]) -> Result<BTreeMap<&str, &str>> {
    let mut indexed = BTreeMap::new();
    for case in cases {
        if case.case_id.trim().is_empty() || case.expected_answer.trim().is_empty() {
            bail!("gold case IDs and expected answers must be non-empty");
        }
        if indexed
            .insert(case.case_id.as_str(), case.expected_answer.as_str())
            .is_some()
        {
            bail!("duplicate gold case ID {}", case.case_id);
        }
    }
    Ok(indexed)
}

fn validate_decision(decision: &DecisionCitation) -> Result<()> {
    for (field, value) in [
        ("human_commit", &decision.human_commit),
        ("decision_path", &decision.decision_path),
        ("deciding_sentence", &decision.deciding_sentence),
        ("second_reviewer", &decision.second_reviewer),
    ] {
        if value.trim().is_empty() {
            bail!("gold mutation lacks cited decision field {field}");
        }
    }
    Ok(())
}

/// Compare a frozen gold baseline with its candidate replacement. One cited
/// human decision authorizes this diff; a model disagreement is not an input.
fn check_gold_diff(
    before: &[GoldCase],
    after: &[GoldCase],
    decision: Option<&DecisionCitation>,
) -> Result<()> {
    let before = by_id(before)?;
    let after = by_id(after)?;
    let ids: BTreeSet<&str> = before.keys().chain(after.keys()).copied().collect();
    for case_id in ids {
        let old = before.get(case_id);
        let new = after.get(case_id);
        if old == new {
            continue;
        }
        let decision = decision.with_context(|| {
            format!("expected answer moved for {case_id} without a cited decision")
        })?;
        validate_decision(decision)
            .with_context(|| format!("unauthorized expected-answer move for {case_id}"))?;
    }
    Ok(())
}

fn case(expected_answer: &str) -> GoldCase {
    GoldCase {
        case_id: "gold-001".to_string(),
        expected_answer: expected_answer.to_string(),
    }
}

fn citation() -> DecisionCitation {
    DecisionCitation {
        human_commit: "human-decision-0007".to_string(),
        decision_path: "docs/decisions.md#gold-001".to_string(),
        deciding_sentence: "The case was out of date; the neutral expected answer is BLOCKED."
            .to_string(),
        second_reviewer: "human-reviewer-2".to_string(),
    }
}

#[test]
fn unchanged_gold_needs_no_mutation_decision() {
    let baseline = vec![case("BLOCKED")];
    check_gold_diff(&baseline, &baseline, None).expect("unchanged gold");
}

#[test]
fn expected_answer_move_with_complete_cited_decision_passes() {
    let before = vec![case("PASS")];
    let after = vec![case("BLOCKED")];
    check_gold_diff(&before, &after, Some(&citation())).expect("authorized gold correction");
}

#[test]
fn seeded_expected_answer_move_without_decision_fails() {
    let error = check_gold_diff(&[case("PASS")], &[case("BLOCKED")], None)
        .expect_err("seeded uncited gold mutation must fail");
    assert!(
        error.to_string().contains("without a cited decision"),
        "{error:#}"
    );
}

#[test]
fn seeded_missing_second_reviewer_fails() {
    let mut decision = citation();
    decision.second_reviewer.clear();
    let error = check_gold_diff(&[case("PASS")], &[case("BLOCKED")], Some(&decision))
        .expect_err("seeded one-person gold mutation must fail");
    let detail = format!("{error:#}");
    assert!(detail.contains("second_reviewer"), "{detail}");
}

#[test]
fn seeded_added_gold_case_also_requires_decision() {
    let error = check_gold_diff(&[], &[case("PASS")], None)
        .expect_err("adding an expected answer changes the oracle");
    assert!(
        error.to_string().contains("without a cited decision"),
        "{error:#}"
    );
}
