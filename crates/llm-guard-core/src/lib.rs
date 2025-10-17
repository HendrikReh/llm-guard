pub mod llm;
pub mod report;
pub mod scanner;

pub use llm::{build_client, LlmClient, LlmSettings, NoopLlmClient, OpenAiClient};
pub use report::{render_report, OutputFormat};
pub use scanner::{
    default_scanner::DefaultScanner, file_repository::FileRuleRepository, FamilyContribution,
    Finding, FindingValidationError, LlmVerdict, RiskBand, RiskConfig, RiskThresholds, Rule,
    RuleKind, RuleRepository, RuleValidationError, ScanReport, Scanner, ScoreBreakdown, Span,
    VerdictProvider,
};
