use anyhow::Result as AnyResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Byte span within the scanned text `(start, end)` where `start <= end`.
pub type Span = (usize, usize);

/// Classification buckets for overall risk scoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskBand {
    Low,
    Medium,
    High,
}

impl RiskBand {
    /// Map a numeric risk score (0–100) into a risk band.
    pub fn from_score(score: f32) -> Self {
        match score {
            s if s >= 60.0 => Self::High,
            s if s >= 25.0 => Self::Medium,
            _ => Self::Low,
        }
    }
}

/// Distinguishes between literal keyword and regular-expression rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleKind {
    Keyword,
    Regex,
}

/// Definition of a single detection rule used during scanning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Unique identifier (namespaced, e.g. `INSTR_OVERRIDE`).
    pub id: String,
    /// Human-readable summary shown in reports.
    pub description: String,
    /// Rule category (keyword or regex).
    pub kind: RuleKind,
    /// Pattern literal or regex source.
    pub pattern: String,
    /// Contribution to risk score (0.0–100.0 inclusive).
    pub weight: f32,
    /// Optional character window to capture around matches.
    pub window: Option<usize>,
}

impl Rule {
    /// Construct a new rule, validating invariants before returning.
    pub fn new(
        id: impl Into<String>,
        description: impl Into<String>,
        kind: RuleKind,
        pattern: impl Into<String>,
        weight: f32,
        window: Option<usize>,
    ) -> Result<Self, RuleValidationError> {
        let rule = Self {
            id: id.into(),
            description: description.into(),
            kind,
            pattern: pattern.into(),
            weight,
            window,
        };
        rule.validate()?;
        Ok(rule)
    }

    /// Validate invariants for existing rule definitions.
    pub fn validate(&self) -> Result<(), RuleValidationError> {
        if self.id.trim().is_empty() {
            return Err(RuleValidationError::EmptyId);
        }
        if self.pattern.is_empty() {
            return Err(RuleValidationError::EmptyPattern {
                rule_id: self.id.clone(),
            });
        }
        if !(0.0..=100.0).contains(&self.weight) {
            return Err(RuleValidationError::InvalidWeight {
                rule_id: self.id.clone(),
                weight: self.weight,
            });
        }
        if let Some(window) = self.window {
            if window == 0 {
                return Err(RuleValidationError::InvalidWindow {
                    rule_id: self.id.clone(),
                    window,
                });
            }
        }
        Ok(())
    }
}

/// Errors emitted while validating rule definitions.
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RuleValidationError {
    #[error("rule id must not be blank")]
    EmptyId,
    #[error("rule `{rule_id}` pattern must not be empty")]
    EmptyPattern { rule_id: String },
    #[error("rule `{rule_id}` weight must be within 0.0..=100.0 (got {weight})")]
    InvalidWeight { rule_id: String, weight: f32 },
    #[error("rule `{rule_id}` window must be > 0 when specified (got {window})")]
    InvalidWindow { rule_id: String, window: usize },
}

/// A feature triggered during scanning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub rule_id: String,
    pub span: Span,
    pub excerpt: String,
    pub weight: f32,
}

impl Finding {
    /// Validate span invariants and score bounds.
    pub fn validate(&self) -> Result<(), FindingValidationError> {
        if self.span.0 > self.span.1 {
            return Err(FindingValidationError::InvalidSpan {
                rule_id: self.rule_id.clone(),
                span: self.span,
            });
        }
        if !(0.0..=100.0).contains(&self.weight) {
            return Err(FindingValidationError::InvalidWeight {
                rule_id: self.rule_id.clone(),
                weight: self.weight,
            });
        }
        Ok(())
    }
}

/// Validation errors for findings emitted by the scanner.
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FindingValidationError {
    #[error("finding for rule `{rule_id}` has invalid span ({span:?})")]
    InvalidSpan { rule_id: String, span: Span },
    #[error("finding for rule `{rule_id}` weight must be within 0.0..=100.0 (got {weight})")]
    InvalidWeight { rule_id: String, weight: f32 },
}

/// Optional LLM verdict that augments the heuristic risk score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmVerdict {
    pub label: String,
    pub rationale: String,
    pub mitigation: String,
}

/// End-to-end report produced by the scanner pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub risk_score: f32,
    pub findings: Vec<Finding>,
    pub normalized_len: usize,
    pub risk_band: RiskBand,
    pub llm_verdict: Option<LlmVerdict>,
}

impl ScanReport {
    /// Construct a report while computing the derived risk band.
    pub fn new(
        risk_score: f32,
        findings: Vec<Finding>,
        normalized_len: usize,
        llm_verdict: Option<LlmVerdict>,
    ) -> Self {
        let clamped_score = risk_score.clamp(0.0, 100.0);
        Self {
            risk_band: RiskBand::from_score(clamped_score),
            risk_score: clamped_score,
            findings,
            normalized_len,
            llm_verdict,
        }
    }
}

/// Abstraction over rule loading so different backends (files, HTTP, in-memory) can be swapped transparently.
#[async_trait]
pub trait RuleRepository: Send + Sync {
    /// Retrieve the full rule set currently active.
    async fn load_rules(&self) -> AnyResult<Vec<Rule>>;

    /// Fetch a single rule by identifier if it exists.
    async fn get_rule(&self, rule_id: &str) -> AnyResult<Option<Rule>>;
}

/// Primary scanning interface that transforms raw text into a structured report.
#[async_trait]
pub trait Scanner: Send + Sync {
    /// Execute the scan against provided UTF-8 text, returning findings and risk metrics.
    async fn scan(&self, input: &str) -> AnyResult<ScanReport>;
}

/// Optional provider that enriches heuristic results with LLM judgments.
#[async_trait]
pub trait VerdictProvider: Send + Sync {
    /// Produce an additional verdict given the original input and heuristic report.
    async fn verdict(&self, input: &str, report: &ScanReport) -> AnyResult<LlmVerdict>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_validation_rejects_invalid_weight() {
        let rule = Rule {
            id: "TEST".into(),
            description: "invalid weight".into(),
            kind: RuleKind::Keyword,
            pattern: "override".into(),
            weight: 150.0,
            window: None,
        };

        let err = rule.validate().expect_err("should reject weight > 100");
        assert!(matches!(
            err,
            RuleValidationError::InvalidWeight {
                rule_id,
                weight
            } if rule_id == "TEST" && (weight - 150.0).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn rule_new_enforces_invariants() {
        let rule = Rule::new(
            "INSTR_OVERRIDE",
            "Attempts to override instructions",
            RuleKind::Keyword,
            "ignore previous",
            25.0,
            Some(32),
        )
        .expect("valid rule should be constructed");
        assert_eq!(rule.id, "INSTR_OVERRIDE");
    }

    #[test]
    fn finding_validation_rejects_inverted_span() {
        let finding = Finding {
            rule_id: "TEST".into(),
            span: (10, 2),
            excerpt: "oops".into(),
            weight: 10.0,
        };
        let err = finding
            .validate()
            .expect_err("span start greater than end should be invalid");
        assert!(matches!(
            err,
            FindingValidationError::InvalidSpan { span, .. } if span == (10, 2)
        ));
    }

    #[test]
    fn scan_report_clamps_scores() {
        let report = ScanReport::new(120.0, vec![], 128, None);
        assert!((report.risk_score - 100.0).abs() < f32::EPSILON);
        assert_eq!(report.risk_band, RiskBand::High);
    }

    #[test]
    fn risk_band_thresholds_match_spec() {
        assert_eq!(RiskBand::from_score(10.0), RiskBand::Low);
        assert_eq!(RiskBand::from_score(25.0), RiskBand::Medium);
        assert_eq!(RiskBand::from_score(59.9), RiskBand::Medium);
        assert_eq!(RiskBand::from_score(60.0), RiskBand::High);
    }
}
