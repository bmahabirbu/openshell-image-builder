// Copyright (C) 2026 Red Hat, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use super::Agent;
use crate::inference;

const GOOSE_CONFIG_FILE: &str = ".config/goose/config.yaml";

pub struct GooseAgent;

impl Agent for GooseAgent {
    fn id(&self) -> &str {
        "goose"
    }

    fn install(&self) -> String {
        "RUN ARCH=$(uname -m) && \\\n    \
             curl -fsSL -o /tmp/goose.tar.gz \\\n    \
               \"https://github.com/aaif-goose/goose/releases/download/stable/goose-${ARCH}-unknown-linux-gnu.tar.gz\" && \\\n    \
             mkdir -p /sandbox/.local/bin /sandbox/.config/goose && \\\n    \
             tar -xzf /tmp/goose.tar.gz -C /sandbox/.local/bin && \\\n    \
             rm /tmp/goose.tar.gz\n\
         ENV PATH=/sandbox/.local/bin:$PATH"
            .to_string()
    }

    fn binary_path(&self) -> &str {
        "/sandbox/.local/bin/goose"
    }

    fn skip_onboarding(&self, mut files: HashMap<String, String>) -> HashMap<String, String> {
        let mut config = parse_config(files.get(GOOSE_CONFIG_FILE));

        let telemetry_key = serde_yml::Value::String("GOOSE_TELEMETRY_ENABLED".to_string());
        if !config.contains_key(&telemetry_key) {
            config.insert(telemetry_key, serde_yml::Value::Bool(false));
        }

        files.insert(GOOSE_CONFIG_FILE.to_string(), serialize_config(&config));
        files
    }

    fn supported_inference(&self) -> Vec<inference::InferenceKind> {
        vec![
            inference::InferenceKind::Anthropic,
            inference::InferenceKind::Ollama,
            inference::InferenceKind::OpenAi,
            inference::InferenceKind::VertexAi,
        ]
    }

    fn set_inference(
        &self,
        mut files: HashMap<String, String>,
        _inference: Option<&inference::InferenceKind>,
        _base_url: Option<&str>,
        model: Option<&str>,
    ) -> HashMap<String, String> {
        let Some(m) = model else {
            return files;
        };

        let mut config = parse_config(files.get(GOOSE_CONFIG_FILE));
        config.insert(
            serde_yml::Value::String("GOOSE_MODEL".to_string()),
            serde_yml::Value::String(m.to_string()),
        );
        files.insert(GOOSE_CONFIG_FILE.to_string(), serialize_config(&config));
        files
    }

    fn env_vars(
        &self,
        inference: Option<&inference::InferenceKind>,
        _endpoint: Option<&str>,
        _model: Option<&str>,
    ) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        if let Some(kind) = inference {
            let provider = match kind {
                inference::InferenceKind::Anthropic => "anthropic",
                inference::InferenceKind::Ollama => "ollama",
                inference::InferenceKind::OpenAi => "openai",
                inference::InferenceKind::VertexAi => "gcp_vertex_ai",
            };
            vars.insert("GOOSE_PROVIDER".to_string(), provider.to_string());
        }
        vars
    }

    fn skills_dir(&self) -> &str {
        "/sandbox/.config/goose/skills"
    }
}

fn parse_config(content: Option<&String>) -> serde_yml::Mapping {
    content
        .and_then(|c| serde_yml::from_str(c).ok())
        .unwrap_or_default()
}

fn serialize_config(config: &serde_yml::Mapping) -> String {
    serde_yml::to_string(config).expect("valid yaml mapping")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_id_is_goose() {
        assert_eq!(GooseAgent.id(), "goose");
    }

    #[test]
    fn install_downloads_tar_gz_from_github() {
        assert!(
            GooseAgent
                .install()
                .contains("github.com/aaif-goose/goose/releases/download/stable/")
        );
        assert!(GooseAgent.install().contains(".tar.gz"));
    }

    #[test]
    fn install_creates_config_dir() {
        assert!(
            GooseAgent
                .install()
                .contains("mkdir -p /sandbox/.local/bin /sandbox/.config/goose")
        );
    }

    #[test]
    fn install_adds_local_bin_to_path() {
        assert!(
            GooseAgent
                .install()
                .contains("ENV PATH=/sandbox/.local/bin:$PATH")
        );
    }

    #[test]
    fn binary_path_is_local_bin_goose() {
        assert_eq!(GooseAgent.binary_path(), "/sandbox/.local/bin/goose");
    }

    #[test]
    fn policy_yaml_is_empty() {
        assert!(GooseAgent.policy_yaml().is_empty());
    }

    #[test]
    fn skills_dir_is_config_goose_skills() {
        assert_eq!(GooseAgent.skills_dir(), "/sandbox/.config/goose/skills");
    }

    // supported_inference

    #[test]
    fn supported_inference_includes_anthropic() {
        assert!(
            GooseAgent
                .supported_inference()
                .contains(&inference::InferenceKind::Anthropic)
        );
    }

    #[test]
    fn supported_inference_includes_vertexai() {
        assert!(
            GooseAgent
                .supported_inference()
                .contains(&inference::InferenceKind::VertexAi)
        );
    }

    #[test]
    fn supported_inference_includes_ollama() {
        assert!(
            GooseAgent
                .supported_inference()
                .contains(&inference::InferenceKind::Ollama)
        );
    }

    #[test]
    fn supported_inference_includes_openai() {
        assert!(
            GooseAgent
                .supported_inference()
                .contains(&inference::InferenceKind::OpenAi)
        );
    }

    // skip_onboarding

    #[test]
    fn skip_onboarding_creates_config_file() {
        let result = GooseAgent.skip_onboarding(HashMap::new());
        assert!(result.contains_key(GOOSE_CONFIG_FILE));
    }

    #[test]
    fn skip_onboarding_disables_telemetry() {
        let result = GooseAgent.skip_onboarding(HashMap::new());
        let content = &result[GOOSE_CONFIG_FILE];
        assert!(content.contains("GOOSE_TELEMETRY_ENABLED"));
        assert!(content.contains("false"));
    }

    #[test]
    fn skip_onboarding_preserves_existing_telemetry_setting() {
        let mut files = HashMap::new();
        files.insert(
            GOOSE_CONFIG_FILE.to_string(),
            "GOOSE_TELEMETRY_ENABLED: true\n".to_string(),
        );
        let result = GooseAgent.skip_onboarding(files);
        let content = &result[GOOSE_CONFIG_FILE];
        assert!(content.contains("true"));
    }

    #[test]
    fn skip_onboarding_preserves_existing_fields() {
        let mut files = HashMap::new();
        files.insert(
            GOOSE_CONFIG_FILE.to_string(),
            "GOOSE_MODEL: claude-sonnet-4-6\n".to_string(),
        );
        let result = GooseAgent.skip_onboarding(files);
        let content = &result[GOOSE_CONFIG_FILE];
        assert!(content.contains("claude-sonnet-4-6"));
        assert!(content.contains("GOOSE_TELEMETRY_ENABLED"));
    }

    // set_inference

    #[test]
    fn set_inference_without_model_returns_files_unchanged() {
        let mut files = HashMap::new();
        files.insert("other.txt".to_string(), "content".to_string());
        let result = GooseAgent.set_inference(files.clone(), None, None, None);
        assert_eq!(result, files);
    }

    #[test]
    fn set_inference_with_model_writes_goose_model() {
        let result = GooseAgent.set_inference(
            HashMap::new(),
            Some(&inference::InferenceKind::Anthropic),
            None,
            Some("claude-sonnet-4-6"),
        );
        let content = &result[GOOSE_CONFIG_FILE];
        assert!(content.contains("GOOSE_MODEL"));
        assert!(content.contains("claude-sonnet-4-6"));
    }

    #[test]
    fn set_inference_preserves_existing_config() {
        let mut files = HashMap::new();
        files.insert(
            GOOSE_CONFIG_FILE.to_string(),
            "GOOSE_TELEMETRY_ENABLED: false\n".to_string(),
        );
        let result = GooseAgent.set_inference(
            files,
            Some(&inference::InferenceKind::Anthropic),
            None,
            Some("claude-sonnet-4-6"),
        );
        let content = &result[GOOSE_CONFIG_FILE];
        assert!(content.contains("GOOSE_TELEMETRY_ENABLED"));
        assert!(content.contains("claude-sonnet-4-6"));
    }

    // env_vars

    #[test]
    fn env_vars_with_anthropic_sets_goose_provider() {
        let vars = GooseAgent.env_vars(Some(&inference::InferenceKind::Anthropic), None, None);
        assert_eq!(vars.get("GOOSE_PROVIDER").unwrap(), "anthropic");
    }

    #[test]
    fn env_vars_with_ollama_sets_goose_provider() {
        let vars = GooseAgent.env_vars(Some(&inference::InferenceKind::Ollama), None, None);
        assert_eq!(vars.get("GOOSE_PROVIDER").unwrap(), "ollama");
    }

    #[test]
    fn env_vars_with_openai_sets_goose_provider() {
        let vars = GooseAgent.env_vars(Some(&inference::InferenceKind::OpenAi), None, None);
        assert_eq!(vars.get("GOOSE_PROVIDER").unwrap(), "openai");
    }

    #[test]
    fn env_vars_with_vertexai_sets_goose_provider() {
        let vars = GooseAgent.env_vars(Some(&inference::InferenceKind::VertexAi), None, None);
        assert_eq!(vars.get("GOOSE_PROVIDER").unwrap(), "gcp_vertex_ai");
    }

    #[test]
    fn env_vars_without_inference_returns_empty() {
        let vars = GooseAgent.env_vars(None, None, None);
        assert!(vars.is_empty());
    }
}
