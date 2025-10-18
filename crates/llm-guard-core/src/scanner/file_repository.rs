use std::{collections::HashSet, fs, path::PathBuf};

use anyhow::{Context, Result};

use super::{Rule, RuleKind, RuleRepository};
use once_cell::sync::OnceCell;

/// Loads rules from filesystem files (`keywords.txt` and `patterns.json`) located under a base directory.
pub struct FileRuleRepository {
    base_path: PathBuf,
    cache: OnceCell<Vec<Rule>>,
}

impl FileRuleRepository {
    /// Create a repository rooted at the given directory.
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
            cache: OnceCell::new(),
        }
    }

    fn keywords_path(&self) -> PathBuf {
        self.base_path.join("keywords.txt")
    }

    fn patterns_path(&self) -> PathBuf {
        self.base_path.join("patterns.json")
    }

    fn load_keywords(&self, seen: &mut HashSet<String>) -> Result<Vec<Rule>> {
        let mut rules = Vec::new();
        let path = self.keywords_path();
        if !path.exists() {
            return Ok(rules);
        }
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read keyword rule file at {}", path.display()))?;
        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let parts: Vec<_> = trimmed.splitn(4, '|').map(str::trim).collect();
            if parts.len() != 4 {
                return Err(anyhow::anyhow!(
                    "invalid keyword rule format at {}:{} (expected id|weight|description|pattern)",
                    path.display(),
                    idx + 1
                ));
            }
            let id = parts[0].to_string();
            if !seen.insert(id.clone()) {
                return Err(anyhow::anyhow!("duplicate rule id `{id}`"));
            }
            let weight: f32 = parts[1].parse().with_context(|| {
                format!(
                    "invalid weight `{}` for rule `{}` at {}:{}",
                    parts[1],
                    id,
                    path.display(),
                    idx + 1
                )
            })?;
            let rule = Rule::new(id, parts[2], RuleKind::Keyword, parts[3], weight, None)?;
            rules.push(rule);
        }
        Ok(rules)
    }

    fn load_patterns(&self, seen: &mut HashSet<String>) -> Result<Vec<Rule>> {
        let mut rules = Vec::new();
        let path = self.patterns_path();
        if !path.exists() {
            return Ok(rules);
        }
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read pattern rule file at {}", path.display()))?;
        let items: Vec<JsonRule> = serde_json::from_str(&raw).with_context(|| {
            format!(
                "invalid JSON structure in pattern rule file at {}",
                path.display()
            )
        })?;
        for item in items {
            if !seen.insert(item.id.clone()) {
                return Err(anyhow::anyhow!("duplicate rule id `{}`", item.id));
            }
            let rule = Rule::new(
                item.id,
                item.description,
                RuleKind::Regex,
                item.pattern,
                item.weight,
                item.window,
            )?;
            rules.push(rule);
        }
        Ok(rules)
    }
}

#[async_trait::async_trait]
impl RuleRepository for FileRuleRepository {
    async fn load_rules(&self) -> Result<Vec<Rule>> {
        let rules = self.cache.get_or_try_init(|| {
            let mut seen = HashSet::new();
            let mut rules = self.load_keywords(&mut seen)?;
            rules.extend(self.load_patterns(&mut seen)?);
            Ok::<_, anyhow::Error>(rules)
        })?;
        Ok(rules.clone())
    }

    async fn get_rule(&self, rule_id: &str) -> Result<Option<Rule>> {
        let rules = self.load_rules().await?;
        Ok(rules.into_iter().find(|rule| rule.id == rule_id))
    }
}

#[derive(serde::Deserialize)]
struct JsonRule {
    id: String,
    description: String,
    pattern: String,
    weight: f32,
    #[serde(default)]
    window: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use serde_json::json;
    use std::path::Path;

    fn write(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }

    #[test]
    fn loads_keyword_and_pattern_rules() {
        let temp = tempfile::tempdir().unwrap();
        write(
            &temp.path().join("keywords.txt"),
            r#"
# comment
INSTR_OVERRIDE|25|Attempts to override instructions|ignore previous
DATA_EXFIL|30|Tries to exfiltrate secrets|api key
"#,
        );
        write(
            &temp.path().join("patterns.json"),
            r#"
[
    {
        "id": "STEALTH_REGEX",
        "description": "Regex pattern",
        "pattern": "(?i)system message",
        "weight": 45,
        "window": 64
    }
]
"#,
        );

        let repo = FileRuleRepository::new(temp.path());
        let mut rules = futures::executor::block_on(RuleRepository::load_rules(&repo)).unwrap();
        rules.sort_by(|a, b| a.id.cmp(&b.id));

        assert_eq!(rules.len(), 3);
        assert_eq!(rules[0].id, "DATA_EXFIL");
        assert_eq!(rules[1].id, "INSTR_OVERRIDE");
        assert_eq!(rules[2].id, "STEALTH_REGEX");
        assert_eq!(rules[0].kind, RuleKind::Keyword);
        assert_eq!(rules[2].kind, RuleKind::Regex);
    }

    #[test]
    fn duplicate_ids_error() {
        let temp = tempfile::tempdir().unwrap();
        write(
            &temp.path().join("keywords.txt"),
            "DUP|10|desc|pattern\nDUP|15|dup again|another",
        );
        let repo = FileRuleRepository::new(temp.path());
        let err = futures::executor::block_on(RuleRepository::load_rules(&repo)).unwrap_err();
        assert!(err.to_string().contains("duplicate rule id `DUP`"));
    }

    #[test]
    fn loads_sample_rule_pack_from_repo() {
        let repo_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../rules")
            .canonicalize()
            .expect("rules directory should exist");
        let repo = FileRuleRepository::new(repo_path);
        let rules = futures::executor::block_on(RuleRepository::load_rules(&repo))
            .expect("sample rules should parse");
        assert!(
            rules.iter().any(|rule| rule.id == "INSTR_OVERRIDE"),
            "keywords.txt should provide INSTR_OVERRIDE rule"
        );
        assert!(
            rules.iter().any(|rule| rule.id == "CODE_INJECTION"),
            "patterns.json should provide CODE_INJECTION rule"
        );
    }

    fn text_without_delimiter() -> impl Strategy<Value = String> {
        proptest::string::string_regex("[A-Za-z0-9 _\\-]{3,48}")
            .unwrap()
            .prop_filter("pattern must contain non-whitespace", |s| {
                !s.trim().is_empty()
            })
    }

    proptest! {
        #[test]
        fn keyword_rules_round_trip(
            entries in proptest::collection::vec(
                (
                    text_without_delimiter(),
                    0.1f32..99.9f32,
                    text_without_delimiter(),
                    text_without_delimiter()
                ),
                1..12
            )
        ) {
            let temp = tempfile::tempdir().unwrap();
            let mut buffer = String::new();
            for (idx, (id, weight, desc, pattern)) in entries.iter().enumerate() {
                let unique_id = format!("AUTO{}_{}", idx, id);
                buffer.push_str(&format!(
                    "{id}|{weight:.3}|{desc}|{pattern}\n",
                    id = unique_id,
                    weight = weight.clamp(0.1, 99.9),
                    desc = desc,
                    pattern = pattern
                ));
            }
            write(&temp.path().join("keywords.txt"), &buffer);

            let repo = FileRuleRepository::new(temp.path());
            let rules = futures::executor::block_on(RuleRepository::load_rules(&repo))
                .expect("keyword rules should parse");

            prop_assert_eq!(rules.len(), entries.len());
            for rule in rules {
                prop_assert!(rule.weight >= 0.0 && rule.weight <= 100.0);
                prop_assert_eq!(rule.kind, RuleKind::Keyword);
            }
        }
    }

    proptest! {
        #[test]
        fn pattern_rules_round_trip(
            entries in proptest::collection::vec(
                (
                    text_without_delimiter(),
                    0.1f32..99.9f32,
                    text_without_delimiter(),
                    proptest::option::of(1usize..256usize)
                ),
                1..12
            )
        ) {
            let temp = tempfile::tempdir().unwrap();
            let mut json_rules = Vec::new();
            for (idx, (desc, weight, pattern, window)) in entries.iter().enumerate() {
                json_rules.push(json!({
                    "id": format!("REGEX_AUTO_{}", idx),
                    "description": desc,
                    "pattern": regex::escape(pattern),
                    "weight": weight.clamp(0.1, 99.9),
                    "window": window,
                }));
            }
            write(
                &temp.path().join("patterns.json"),
                &serde_json::to_string(&json_rules).unwrap(),
            );

            let repo = FileRuleRepository::new(temp.path());
            let rules = futures::executor::block_on(RuleRepository::load_rules(&repo))
                .expect("pattern rules should parse");

            prop_assert_eq!(rules.len(), entries.len());
            for rule in rules {
                prop_assert!(rule.weight >= 0.0 && rule.weight <= 100.0);
                prop_assert_eq!(rule.kind, RuleKind::Regex);
            }
        }
    }

    proptest! {
        #[test]
        fn duplicate_ids_trigger_error(pattern in text_without_delimiter()) {
            let temp = tempfile::tempdir().unwrap();
            let duplicate = "DUPLICATE_ID";
            let keywords = format!(
                "{id}|10|desc|{pattern}\n{id}|12|desc2|{pattern}_again\n",
                id = duplicate,
                pattern = pattern
            );
            write(&temp.path().join("keywords.txt"), &keywords);
            let repo = FileRuleRepository::new(temp.path());
            let err = futures::executor::block_on(RuleRepository::load_rules(&repo))
                .expect_err("duplicate ids should error");
            prop_assert!(err.to_string().contains("duplicate rule id"));
        }
    }
}
