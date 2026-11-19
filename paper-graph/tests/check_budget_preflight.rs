use anyhow::{bail, Result};

#[derive(Debug, Clone)]
struct Budget {
    context_window: u64,
    p95_prompt_tokens: u64,
    output_budget: u64,
    max_tokens: u64,
    timeout_seconds: f64,
    measured_p95_latency_seconds: f64,
    measured_tokens_per_second: f64,
    cold_start_margin_seconds: f64,
    batch_limit: usize,
    largest_input_record: usize,
    attempts: u8,
}

fn required_timeout_seconds(budget: &Budget) -> Result<f64> {
    for (field, value) in [
        ("timeout_seconds", budget.timeout_seconds),
        (
            "measured_p95_latency_seconds",
            budget.measured_p95_latency_seconds,
        ),
        (
            "measured_tokens_per_second",
            budget.measured_tokens_per_second,
        ),
        (
            "cold_start_margin_seconds",
            budget.cold_start_margin_seconds,
        ),
    ] {
        if !value.is_finite() || value < 0.0 {
            bail!("{field} must be finite and non-negative");
        }
    }
    if budget.measured_tokens_per_second == 0.0 {
        bail!("measured_tokens_per_second must be positive");
    }
    let generation_seconds = budget.output_budget as f64 / budget.measured_tokens_per_second;
    Ok(budget.measured_p95_latency_seconds.max(generation_seconds)
        + budget.cold_start_margin_seconds)
}

fn preflight(budget: &Budget) -> Result<()> {
    let required_context = budget
        .p95_prompt_tokens
        .checked_add(budget.output_budget)
        .ok_or_else(|| anyhow::anyhow!("prompt + output budget overflows"))?;
    if budget.context_window < required_context {
        bail!(
            "context_window {} < p95_prompt_tokens + output_budget {}",
            budget.context_window,
            required_context
        );
    }
    if budget.max_tokens < budget.output_budget {
        bail!(
            "max_tokens {} < output_budget {}",
            budget.max_tokens,
            budget.output_budget
        );
    }
    if budget.batch_limit < budget.largest_input_record {
        bail!(
            "batch_limit {} < largest_input_record {}",
            budget.batch_limit,
            budget.largest_input_record
        );
    }
    if !(1..=3).contains(&budget.attempts) {
        bail!(
            "attempts {} exceeds the three-attempt cap or is zero",
            budget.attempts
        );
    }
    let required_timeout = required_timeout_seconds(budget)?;
    if budget.timeout_seconds < required_timeout {
        bail!(
            "timeout_seconds {} < measured p95/throughput requirement {}",
            budget.timeout_seconds,
            required_timeout
        );
    }
    Ok(())
}

fn valid_budget() -> Budget {
    Budget {
        context_window: 16_384,
        p95_prompt_tokens: 8_192,
        output_budget: 4_096,
        max_tokens: 4_096,
        timeout_seconds: 46.0,
        measured_p95_latency_seconds: 40.0,
        measured_tokens_per_second: 100.0,
        cold_start_margin_seconds: 5.0,
        batch_limit: 32_768,
        largest_input_record: 32_000,
        attempts: 3,
    }
}

#[test]
fn valid_budget_accepts_exact_max_token_and_three_attempt_boundaries() {
    let budget = valid_budget();
    assert!((required_timeout_seconds(&budget).expect("timeout") - 45.96).abs() < 1e-9);
    preflight(&budget).expect("valid budget");
}

#[test]
fn seeded_context_window_deficit_fails_before_execution() {
    let mut budget = valid_budget();
    budget.context_window = 12_287;
    let error = preflight(&budget).expect_err("seeded context deficit must fail");
    assert!(error.to_string().contains("context_window"), "{error:#}");
}

#[test]
fn seeded_max_tokens_deficit_fails() {
    let mut budget = valid_budget();
    budget.max_tokens = 4_095;
    let error = preflight(&budget).expect_err("seeded max_tokens deficit must fail");
    assert!(error.to_string().contains("max_tokens"), "{error:#}");
}

#[test]
fn seeded_timeout_below_p95_and_throughput_requirement_fails() {
    let mut budget = valid_budget();
    budget.timeout_seconds = 45.0;
    let error = preflight(&budget).expect_err("seeded arbitrary timeout must fail");
    assert!(error.to_string().contains("timeout_seconds"), "{error:#}");
}

#[test]
fn seeded_batch_limit_smaller_than_record_fails() {
    let mut budget = valid_budget();
    budget.batch_limit = 31_999;
    let error = preflight(&budget).expect_err("seeded oversized record must fail");
    assert!(error.to_string().contains("batch_limit"), "{error:#}");
}

#[test]
fn seeded_fourth_attempt_fails() {
    let mut budget = valid_budget();
    budget.attempts = 4;
    let error = preflight(&budget).expect_err("attempts <= 3 including retries");
    assert!(error.to_string().contains("three-attempt"), "{error:#}");
}

#[test]
fn seeded_unmeasured_throughput_fails_closed() {
    let mut budget = valid_budget();
    budget.measured_tokens_per_second = 0.0;
    let error = preflight(&budget).expect_err("zero throughput cannot size a timeout");
    assert!(error.to_string().contains("must be positive"), "{error:#}");
}
