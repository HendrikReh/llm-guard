# Release Notes: LLM-Guard v0.4.1

> **AI Coding Hackathon Release** — Production-ready prompt injection detection with multi-provider LLM enrichment

---

## 🎯 Overview

LLM-Guard v0.4.1 is a **fast, explainable Rust CLI** for detecting prompt injection and jailbreak attempts in LLM applications. This release delivers production-grade multi-provider LLM integration, enhanced detection rules, and comprehensive debug capabilities.

**Developed in ~7 hours** during the [AI Coding Accelerator](https://maven.com/nila/ai-coding-accelerator) hackathon using AI-assisted development (GPT-5 Codex + Claude Code).

---

## ✨ Key Features

### Core Capabilities
- ⚡ **Fast Scanning:** <100ms for typical prompts using Aho-Corasick + compiled regex
- 📊 **Transparent Risk Scoring:** 0-100 scale with detailed rule attribution and text excerpts
- 🔌 **Multi-Provider LLM Support:** OpenAI, Anthropic, Google Gemini, Azure OpenAI via `rig.rs`
- 🏥 **Provider Health Checks:** Built-in diagnostics for validating connectivity and configuration
- 📁 **Flexible Input Sources:** Files, stdin, streaming logs (tail mode)
- 📤 **Multiple Output Formats:** Human-readable CLI or JSON for CI/CD automation
- 🚦 **Exit Code Integration:** 0=low, 2=medium, 3=high, 1=error

### Detection Coverage
- **Instruction Override:** `INSTR_IGNORE`, `INSTR_OVERRIDE` patterns
- **Data Exfiltration:** `PROMPT_LEAK` detection with flexible regex
- **Policy Subversion:** `MODEL_OVERRIDE` jailbreak patterns
- **Obfuscation Techniques:** `CODE_INJECTION` payload recognition

---

## 🐛 Critical Fixes

### Gemini Provider Integration
**Problem:** Rig.rs deserialization errors (`missing field generationConfig`) and API rejection of function calling with JSON MIME type
**Solution:** Bypassed rig entirely; implemented standalone HTTP client using Gemini's native REST API
**Impact:** Gemini now fully functional with `generationConfig.responseMimeType: "application/json"`

### OpenAI GPT-5 Reasoning Models
**Problem:** Models returned only reasoning traces (no textual content) with `json_schema` response format
**Solution:** Switched from strict `json_schema` to flexible `json_object` format
**Impact:** Full compatibility with GPT-5 reasoning models; cleaner codebase

### Detection Rules Gap
**Problem:** Keyword "ignore previous instructions" missed variations like "ignore **your** previous instructions"
**Solution:** Added flexible regex patterns `INSTR_IGNORE` and `PROMPT_LEAK` to `rules/patterns.json`
**Impact:** Scanner now catches attack variations; heuristic and LLM verdicts align

**Example:**
```
Before: Risk Score: 0.0 (Low), No findings
After:  Risk Score: 37.5 (Medium)
        Findings: PROMPT_LEAK [40.0], INSTR_IGNORE [35.0]
```

### Debug Logging Enhancement
**Problem:** `--debug` flag only logged errors, not all raw LLM responses
**Solution:** Added universal debug logging for all providers (raw response + extracted content)
**Impact:** Easier diagnosis of parsing issues and provider behavior quirks

---

## 📦 What's Included

### Binaries
```bash
# Build from source
cargo build --release
./target/release/llm-guard --version  # v0.4.1
```

### Configuration Files
- `llm_providers.example.yaml` — Multi-provider config template
- `rules/keywords.txt` — Exact-match keyword database
- `rules/patterns.json` — Regex patterns for flexible detection

### Documentation
- `README.md` — Complete project overview with hackathon context
- `docs/USAGE.md` — Comprehensive CLI reference
- `docs/TESTING_GUIDE.md` — Testing protocols and provider health checks
- `AGENTS.md` — AI assistant onboarding guide
- `PLAN.md` — Implementation roadmap and phase tracking
- `PROJECT_SUMMARY.md` — Current state snapshot

---

## 🚀 Quick Start

### Installation
```bash
git clone https://github.com/HendrikReh/llm-guard
cd llm-guard
cargo build --release
```

### Basic Usage
```bash
# Scan a file
./target/release/llm-guard scan --file examples/chat.txt

# LLM-enhanced scan with Gemini
export LLM_GUARD_PROVIDER=gemini
export LLM_GUARD_API_KEY=your_key_here
./target/release/llm-guard scan --file examples/chat.txt --with-llm

# Debug mode (dump raw responses)
./target/release/llm-guard scan --file examples/chat.txt --with-llm --debug

# Provider health check
./target/release/llm-guard health --providers-config llm_providers.yaml
```

### CI/CD Integration
```bash
# Generate JSON output
./target/release/llm-guard scan --file input.txt --json > report.json

# Exit codes: 0=low, 2=medium, 3=high, 1=error
if [ $? -ge 2 ]; then
  echo "Security risk detected!"
  exit 1
fi
```

---

## 🔧 Configuration

### Environment Variables
| Variable | Description | Default |
|----------|-------------|---------|
| `LLM_GUARD_PROVIDER` | Provider (`openai`, `anthropic`, `gemini`, `azure`) | `openai` |
| `LLM_GUARD_API_KEY` | API key/token | - |
| `LLM_GUARD_MODEL` | Model name (e.g., `gpt-4o-mini`) | Provider default |
| `LLM_GUARD_ENDPOINT` | Custom endpoint URL | Provider default |
| `LLM_GUARD_TIMEOUT_SECS` | HTTP timeout in seconds | `30` |
| `LLM_GUARD_MAX_RETRIES` | Retry count for failed calls | `2` |

### Provider Profiles (`llm_providers.yaml`)
```yaml
providers:
  - name: "openai"
    api_key: "OPENAI_API_KEY"
    model: "gpt-4o-mini"
  - name: "gemini"
    api_key: "GEMINI_API_KEY"
    model: "gemini-1.5-flash"
  - name: "azure"
    api_key: "AZURE_OPENAI_KEY"
    endpoint: "https://your-resource.openai.azure.com"
    deployment: "gpt-4o-production"
    api_version: "2024-02-15-preview"
```

**Configuration Precedence:** CLI flags → Environment variables → Provider profile

---

## 📊 Technical Metrics

| Metric | Value |
|--------|-------|
| **Lines of Code** | ~4,000 (Rust) |
| **Source Files** | 25 `.rs` files |
| **Test Coverage** | 44 tests (34 passing, 10 ignored) |
| **Dependencies** | Production-grade (tokio, reqwest, rig, clap) |
| **Detection Rules** | 4 patterns + keyword database |
| **Supported Providers** | 4 (OpenAI, Anthropic, Gemini, Azure) |
| **Performance** | <100ms for typical prompts |

---

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run library tests only
cargo test --lib

# Run with ignored tests (requires network)
cargo test -- --include-ignored

# Provider health checks
cargo run --bin llm-guard-cli -- health --providers-config llm_providers.yaml
```

See [`docs/TESTING_GUIDE.md`](./docs/TESTING_GUIDE.md) for comprehensive testing protocols.

---

## 🤖 AI-Assisted Development

This release demonstrates the capabilities of **AI-assisted software development**:

**Workflow:**
- **Primary Agent:** GPT-5 Codex (core logic, LLM adapters, CLI)
- **Review Agent:** Claude Code (code reviews, documentation, debugging)
- **Context Management:** RepoPrompt + Context7 MCP servers

**What Worked:**
- ✅ Functional CLI with 4 LLM providers in <7 hours
- ✅ Multi-agent collaboration (coding vs. review separation)
- ✅ MCP integration eliminated manual file navigation
- ✅ PRD-driven development prevented scope creep

**Challenges:**
- ⚠️ Provider API quirks (Gemini, OpenAI reasoning models)
- ⚠️ Testing gaps due to time pressure (10 ignored tests)
- ⚠️ Rig.rs limitations required Gemini bypass

---

## 🔮 Known Limitations

- **Rule Coverage:** Only 4 detection patterns (expandable via `rules/patterns.json`)
- **Context Windows:** Limited to 200-char proximity for synergy bonuses
- **Test Coverage:** 10 tests ignored (require network or specific environments)
- **Production Readiness:** Prototype for research/education; not audited for production security workloads

---

## 📚 Resources

- **Main Documentation:** [README.md](./README.md)
- **Usage Reference:** [docs/USAGE.md](./docs/USAGE.md)
- **Testing Guide:** [docs/TESTING_GUIDE.md](./docs/TESTING_GUIDE.md)
- **Implementation Plan:** [PLAN.md](./PLAN.md)
- **AI Onboarding:** [AGENTS.md](./AGENTS.md)
- **Project Summary:** [PROJECT_SUMMARY.md](./PROJECT_SUMMARY.md)

---

## 🙏 Acknowledgments

**Hackathon:** [AI Coding Accelerator](https://maven.com/nila/ai-coding-accelerator) (Maven)
**Instructors:** [Vignesh Mohankumar](https://x.com/vig_xyz), [Jason Liu](https://x.com/jxnlco)

**Built with:**
- [Cursor](https://cursor.sh) + GPT-5 Codex
- [Claude Code](https://claude.ai)
- [RepoPrompt MCP](https://repoprompt.com/)
- [Context7 MCP](https://context7.com/)

---

## 📄 License

Apache-2.0 OR MIT

**Security Disclaimer:** This tool is a prototype for research/education. Use at your own risk.

**AI Development Notice:** Codebase primarily generated via AI assistants (GPT-5 Codex, Claude Code) with human oversight for architecture, testing, and quality validation.

---

## 🔗 Links

- **Repository:** https://github.com/HendrikReh/llm-guard
- **Issues:** https://github.com/HendrikReh/llm-guard/issues
- **Releases:** https://github.com/HendrikReh/llm-guard/releases

---

**Full Changelog:** https://github.com/HendrikReh/llm-guard/compare/v0.4.0...v0.4.1
