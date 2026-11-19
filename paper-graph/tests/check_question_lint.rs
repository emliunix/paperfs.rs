use anyhow::{bail, Result};
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
struct SchemaField {
    name: String,
    description: String,
}

#[derive(Debug, Clone)]
struct Question {
    case_id: String,
    text: String,
    schema_fields: Vec<SchemaField>,
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn lint_question(question: &Question) -> Result<()> {
    if question.case_id.trim().is_empty() || question.text.trim().is_empty() {
        bail!("question ID and text must be non-empty");
    }
    let text = question.text.to_ascii_lowercase();
    let asks_for_created_artifact = contains_any(
        &text,
        &["what file", "which file", "what path", "which path"],
    ) && contains_any(
        &text,
        &["create", "creates", "created", "produce", "generat"],
    );
    let explicitly_conditional = contains_any(
        &text,
        &["if any", "if one", "if so", "whether", "no file is created"],
    );
    if asks_for_created_artifact && !explicitly_conditional {
        bail!(
            "question {} presupposes a created artifact; ask whether one exists first",
            question.case_id
        );
    }

    let mut names = BTreeSet::new();
    for field in &question.schema_fields {
        if field.name.trim().is_empty() || field.description.trim().is_empty() {
            bail!("schema field names and descriptions must be non-empty");
        }
        if !names.insert(&field.name) {
            bail!("duplicate schema field {}", field.name);
        }
        let definition = format!("{} {}", field.name, field.description).to_ascii_lowercase();
        let stage_dimension = contains_any(&definition, &["stage", "phase", "status"]);
        let decision_dimension =
            contains_any(&definition, &["decision", "decides", "verdict", "outcome"]);
        if stage_dimension && decision_dimension && definition.contains(" and ") {
            bail!(
                "schema field {} conflates stage/status with decision/outcome",
                field.name
            );
        }
    }
    Ok(())
}

fn neutral_question() -> Question {
    Question {
        case_id: "retro-outcome".to_string(),
        text: "Does the retro create a file, and if so, what path is reported? Answer none when no file is created.".to_string(),
        schema_fields: vec![
            SchemaField {
                name: "stage".to_string(),
                description: "Which lifecycle stage is running.".to_string(),
            },
            SchemaField {
                name: "decision".to_string(),
                description: "What that stage decides.".to_string(),
            },
        ],
    }
}

#[test]
fn neutral_conditional_question_and_split_schema_pass() {
    lint_question(&neutral_question()).expect("neutral question with one dimension per field");
}

#[test]
fn seeded_outcome_presupposing_question_fails() {
    let mut question = neutral_question();
    question.text = "What file does the retro create?".to_string();
    let error = lint_question(&question).expect_err("seeded presupposition must fail");
    assert!(error.to_string().contains("presupposes"), "{error:#}");
}

#[test]
fn seeded_schema_field_conflation_fails() {
    let mut question = neutral_question();
    question.schema_fields = vec![SchemaField {
        name: "next_action".to_string(),
        description: "Which stage is running and what it decides.".to_string(),
    }];
    let error = lint_question(&question).expect_err("seeded conflated field must fail");
    assert!(error.to_string().contains("conflates"), "{error:#}");
}

#[test]
fn unconditional_path_question_fails_even_with_split_fields() {
    let mut question = neutral_question();
    question.text = "Which path does the report generator produce?".to_string();
    let error = lint_question(&question).expect_err("an assumed report path bends the trap");
    assert!(error.to_string().contains("presupposes"), "{error:#}");
}
