use std::fmt::Write;

use serde::Serialize;

use crate::scanner::{FamilyContribution, Finding, RiskBand, ScanReport};

/// Format styles supported in default reporter implementations.
#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Human,
    Json,
}

/// Produce a report string from a `ScanReport` using the desired format.
pub fn render_report(report: &ScanReport, format: OutputFormat) -> anyhow::Result<String> {
    match format {
        OutputFormat::Human => render_human(report),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(&JsonReport::from(report))?),
    }
}

fn render_human(report: &ScanReport) -> anyhow::Result<String> {
    let mut out = String::new();
    writeln!(
        out,
        "Risk Score: {:.1} ({:?})",
        report.risk_score, report.risk_band
    )?;
    writeln!(out, "Normalized Length: {} chars", report.normalized_len)?;
    writeln!(out)?;

    if report.findings.is_empty() {
        writeln!(out, "No findings detected.")?;
    } else {
        writeln!(out, "Findings:")?;
        for finding in &report.findings {
            writeln!(
                out,
                "  - {id} [{weight:.1}] @ {start}..{end}",
                id = finding.rule_id,
                weight = finding.weight,
                start = finding.span.0,
                end = finding.span.1,
            )?;
            if !finding.excerpt.trim().is_empty() {
                writeln!(out, "    \"{}\"", sanitize_excerpt(&finding.excerpt))?;
            }
        }
    }

    writeln!(out)?;
    writeln!(out, "Family Contributions:")?;
    for family in &report.score_breakdown.family_contributions {
        writeln!(
            out,
            "  - {family:>12}: raw {raw:.1}, adjusted {adj:.1} (occurrences: {count})",
            family = family.family,
            raw = family.raw_weight,
            adj = family.adjusted_weight,
            count = family.occurrences
        )?;
    }

    writeln!(
        out,
        "\nLength factor: {:.2} • Adjusted total: {:.1}",
        report.score_breakdown.length_factor, report.score_breakdown.adjusted_total
    )?;

    if let Some(verdict) = &report.llm_verdict {
        writeln!(out, "\nLLM Verdict: {}", verdict.label)?;
        writeln!(out, "  Rationale: {}", verdict.rationale)?;
        writeln!(out, "  Mitigation: {}", verdict.mitigation)?;
    }

    Ok(out)
}

fn sanitize_excerpt(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            '\n' | '\r' => ' ',
            _ => c,
        })
        .collect()
}

#[derive(Debug, Serialize)]
struct JsonReport<'a> {
    risk_score: f32,
    risk_band: RiskBand,
    normalized_len: usize,
    findings: &'a [Finding],
    family_contributions: &'a [FamilyContribution],
    breakdown: &'a crate::scanner::ScoreBreakdown,
    llm_verdict: Option<&'a crate::scanner::LlmVerdict>,
}

impl<'a> From<&'a ScanReport> for JsonReport<'a> {
    fn from(report: &'a ScanReport) -> Self {
        Self {
            risk_score: report.risk_score,
            risk_band: report.risk_band,
            normalized_len: report.normalized_len,
            findings: &report.findings,
            family_contributions: &report.score_breakdown.family_contributions,
            breakdown: &report.score_breakdown,
            llm_verdict: report.llm_verdict.as_ref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{FamilyContribution, Finding, ScanReport, ScoreBreakdown};

    fn sample_report() -> ScanReport {
        let findings = vec![Finding {
            rule_id: "TEST_RULE".into(),
            span: (0, 10),
            excerpt: "example excerpt".into(),
            weight: 10.0,
        }];
        let breakdown = ScoreBreakdown {
            raw_total: 10.0,
            adjusted_total: 10.0,
            length_factor: 1.0,
            family_contributions: vec![FamilyContribution {
                family: "TEST".into(),
                occurrences: 1,
                raw_weight: 10.0,
                adjusted_weight: 10.0,
            }],
        };
        ScanReport::from_breakdown(
            findings,
            100,
            None,
            breakdown,
            &crate::scanner::RiskThresholds::default(),
        )
    }

    #[test]
    fn human_report_contains_findings() {
        let report = sample_report();
        let output = render_report(&report, OutputFormat::Human).unwrap();
        assert!(output.contains("Risk Score"));
        assert!(output.contains("TEST_RULE"));
        assert!(output.contains("Family Contributions"));
    }

    #[test]
    fn json_report_serializes() {
        let report = sample_report();
        let output = render_report(&report, OutputFormat::Json).unwrap();
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(value["risk_score"], serde_json::json!(report.risk_score));
        assert!(value["findings"].is_array());
    }
}
