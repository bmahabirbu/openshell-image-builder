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

const OPENCLAW_CONFIG_FILE: &str = ".openclaw/openclaw.json";

pub struct OpenclawAgent;

impl Agent for OpenclawAgent {
    fn id(&self) -> &str {
        "openclaw"
    }

    fn install(&self) -> String {
        "RUN curl -fsSL https://openclaw.ai/install.sh | bash\nENV PATH=/sandbox/.local/bin:$PATH"
            .to_string()
    }

    fn binary_path(&self) -> &str {
        "/sandbox/.local/bin/openclaw"
    }

    fn skip_onboarding(&self, mut files: HashMap<String, String>) -> HashMap<String, String> {
        let content = files
            .get(OPENCLAW_CONFIG_FILE)
            .cloned()
            .unwrap_or_else(|| "{}".to_string());
        let mut config: serde_json::Value =
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}));

        if !config["gateway"].is_object() {
            config["gateway"] = serde_json::json!({});
        }
        config["gateway"]["auth"] = serde_json::json!({
            "mode": "token",
            "token": "openclaw123"
        });
        config["gateway"]["controlUi"] = serde_json::json!({ "enabled": true });
        config["gateway"]["bind"] = serde_json::json!("lan");

        files.insert(
            OPENCLAW_CONFIG_FILE.to_string(),
            serde_json::to_string_pretty(&config).expect("valid json value"),
        );
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
        inference: Option<&inference::InferenceKind>,
        base_url: Option<&str>,
        model: Option<&str>,
    ) -> HashMap<String, String> {
        if model.is_none() && base_url.is_none() {
            return files;
        }

        let content = files
            .get(OPENCLAW_CONFIG_FILE)
            .cloned()
            .unwrap_or_else(|| "{}".to_string());
        let mut config: serde_json::Value =
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}));

        if let Some(m) = model {
            let model_ref = match inference {
                Some(inference::InferenceKind::Ollama) => format!("ollama/{m}"),
                Some(inference::InferenceKind::OpenAi) => format!("openai/{m}"),
                Some(inference::InferenceKind::Anthropic) => format!("anthropic/{m}"),
                _ => m.to_string(),
            };

            if !config["agents"].is_object() {
                config["agents"] = serde_json::json!({});
            }
            if !config["agents"]["defaults"].is_object() {
                config["agents"]["defaults"] = serde_json::json!({});
            }
            config["agents"]["defaults"]["model"] = serde_json::json!(model_ref);
        }

        if let Some(url) = base_url {
            let provider_name = match inference {
                Some(inference::InferenceKind::Ollama) => "ollama",
                Some(inference::InferenceKind::OpenAi) => "openai-custom",
                Some(inference::InferenceKind::Anthropic) => "anthropic-custom",
                _ => "local",
            };

            let model_name = model.unwrap_or("default");

            if !config["models"].is_object() {
                config["models"] = serde_json::json!({});
            }
            if !config["models"]["providers"].is_object() {
                config["models"]["providers"] = serde_json::json!({});
            }
            config["models"]["providers"][provider_name] = serde_json::json!({
                "baseUrl": url,
                "apiKey": "local-no-auth",
                "api": "openai-completions",
                "models": [{ "id": model_name, "name": model_name }]
            });
        }

        files.insert(
            OPENCLAW_CONFIG_FILE.to_string(),
            serde_json::to_string_pretty(&config).expect("valid json value"),
        );
        files
    }

    fn skills_dir(&self) -> &str {
        "/sandbox/.openclaw/skills"
    }

    fn policy_yaml(&self) -> &str {
        r#"version: 1
network_policies:
  openclaw:
    name: openclaw
    endpoints:
      - { host: openclaw.ai, port: 443 }
    binaries:
      - { path: /sandbox/.local/bin/openclaw }
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_id_is_openclaw() {
        assert_eq!(OpenclawAgent.id(), "openclaw");
    }

    #[test]
    fn install_contains_openclaw_installer() {
        assert!(
            OpenclawAgent
                .install()
                .contains("https://openclaw.ai/install.sh")
        );
    }

    #[test]
    fn install_adds_local_bin_to_path() {
        assert!(
            OpenclawAgent
                .install()
                .contains("ENV PATH=/sandbox/.local/bin:$PATH")
        );
    }

    #[test]
    fn binary_path_is_local_bin_openclaw() {
        assert_eq!(OpenclawAgent.binary_path(), "/sandbox/.local/bin/openclaw");
    }

    #[test]
    fn policy_yaml_has_openclaw_name() {
        assert!(OpenclawAgent.policy_yaml().contains("name: openclaw"));
    }

    #[test]
    fn policy_yaml_has_openclaw_ai_endpoint() {
        assert!(OpenclawAgent.policy_yaml().contains("openclaw.ai"));
    }

    #[test]
    fn policy_yaml_has_binary_path() {
        assert!(
            OpenclawAgent
                .policy_yaml()
                .contains("/sandbox/.local/bin/openclaw")
        );
    }

    #[test]
    fn skills_dir_is_openclaw_skills() {
        assert_eq!(OpenclawAgent.skills_dir(), "/sandbox/.openclaw/skills");
    }

    #[test]
    fn supported_inference_includes_anthropic() {
        assert!(
            OpenclawAgent
                .supported_inference()
                .contains(&inference::InferenceKind::Anthropic)
        );
    }

    #[test]
    fn supported_inference_includes_vertexai() {
        assert!(
            OpenclawAgent
                .supported_inference()
                .contains(&inference::InferenceKind::VertexAi)
        );
    }

    #[test]
    fn supported_inference_includes_ollama() {
        assert!(
            OpenclawAgent
                .supported_inference()
                .contains(&inference::InferenceKind::Ollama)
        );
    }

    #[test]
    fn supported_inference_includes_openai() {
        assert!(
            OpenclawAgent
                .supported_inference()
                .contains(&inference::InferenceKind::OpenAi)
        );
    }

    // skip_onboarding

    #[test]
    fn skip_onboarding_creates_openclaw_json() {
        let result = OpenclawAgent.skip_onboarding(HashMap::new());
        assert!(result.contains_key(OPENCLAW_CONFIG_FILE));
    }

    #[test]
    fn skip_onboarding_sets_gateway_auth_token() {
        let result = OpenclawAgent.skip_onboarding(HashMap::new());
        let json: serde_json::Value =
            serde_json::from_str(result[OPENCLAW_CONFIG_FILE].as_str()).unwrap();
        assert_eq!(json["gateway"]["auth"]["mode"], "token");
        assert_eq!(json["gateway"]["auth"]["token"], "openclaw123");
    }

    #[test]
    fn skip_onboarding_enables_control_ui() {
        let result = OpenclawAgent.skip_onboarding(HashMap::new());
        let json: serde_json::Value =
            serde_json::from_str(result[OPENCLAW_CONFIG_FILE].as_str()).unwrap();
        assert_eq!(json["gateway"]["controlUi"]["enabled"], true);
    }

    #[test]
    fn skip_onboarding_sets_bind_to_lan() {
        let result = OpenclawAgent.skip_onboarding(HashMap::new());
        let json: serde_json::Value =
            serde_json::from_str(result[OPENCLAW_CONFIG_FILE].as_str()).unwrap();
        assert_eq!(json["gateway"]["bind"], "lan");
    }

    #[test]
    fn skip_onboarding_preserves_existing_fields() {
        let mut files = HashMap::new();
        files.insert(
            OPENCLAW_CONFIG_FILE.to_string(),
            r#"{"existingField": "value"}"#.to_string(),
        );
        let result = OpenclawAgent.skip_onboarding(files);
        let json: serde_json::Value =
            serde_json::from_str(result[OPENCLAW_CONFIG_FILE].as_str()).unwrap();
        assert_eq!(json["existingField"], "value");
    }

    // set_inference

    #[test]
    fn set_inference_without_model_or_url_returns_files_unchanged() {
        let mut files = HashMap::new();
        files.insert("other.json".to_string(), "content".to_string());
        let result = OpenclawAgent.set_inference(files.clone(), None, None, None);
        assert_eq!(result, files);
    }

    #[test]
    fn set_inference_with_model_sets_agents_defaults_model() {
        let result = OpenclawAgent.set_inference(
            HashMap::new(),
            Some(&inference::InferenceKind::Anthropic),
            None,
            Some("claude-sonnet-4-6"),
        );
        let json: serde_json::Value =
            serde_json::from_str(result[OPENCLAW_CONFIG_FILE].as_str()).unwrap();
        assert_eq!(
            json["agents"]["defaults"]["model"],
            "anthropic/claude-sonnet-4-6"
        );
    }

    #[test]
    fn set_inference_ollama_model_uses_ollama_prefix() {
        let result = OpenclawAgent.set_inference(
            HashMap::new(),
            Some(&inference::InferenceKind::Ollama),
            None,
            Some("qwen3-coder:30b"),
        );
        let json: serde_json::Value =
            serde_json::from_str(result[OPENCLAW_CONFIG_FILE].as_str()).unwrap();
        assert_eq!(
            json["agents"]["defaults"]["model"],
            "ollama/qwen3-coder:30b"
        );
    }

    #[test]
    fn set_inference_openai_model_uses_openai_prefix() {
        let result = OpenclawAgent.set_inference(
            HashMap::new(),
            Some(&inference::InferenceKind::OpenAi),
            None,
            Some("gpt-4o"),
        );
        let json: serde_json::Value =
            serde_json::from_str(result[OPENCLAW_CONFIG_FILE].as_str()).unwrap();
        assert_eq!(json["agents"]["defaults"]["model"], "openai/gpt-4o");
    }

    #[test]
    fn set_inference_with_base_url_configures_provider() {
        let result = OpenclawAgent.set_inference(
            HashMap::new(),
            Some(&inference::InferenceKind::Ollama),
            Some("http://host.openshell.internal:11434/v1"),
            Some("qwen3-coder:30b"),
        );
        let json: serde_json::Value =
            serde_json::from_str(result[OPENCLAW_CONFIG_FILE].as_str()).unwrap();
        assert_eq!(
            json["models"]["providers"]["ollama"]["baseUrl"],
            "http://host.openshell.internal:11434/v1"
        );
        assert_eq!(
            json["models"]["providers"]["ollama"]["api"],
            "openai-completions"
        );
    }

    #[test]
    fn set_inference_preserves_existing_config() {
        let mut files = HashMap::new();
        files.insert(
            OPENCLAW_CONFIG_FILE.to_string(),
            r#"{"gateway": {"bind": "lan"}}"#.to_string(),
        );
        let result = OpenclawAgent.set_inference(
            files,
            Some(&inference::InferenceKind::Anthropic),
            None,
            Some("claude-sonnet-4-6"),
        );
        let json: serde_json::Value =
            serde_json::from_str(result[OPENCLAW_CONFIG_FILE].as_str()).unwrap();
        assert_eq!(json["gateway"]["bind"], "lan");
        assert_eq!(
            json["agents"]["defaults"]["model"],
            "anthropic/claude-sonnet-4-6"
        );
    }

    #[test]
    fn set_inference_openai_with_endpoint_sets_custom_provider() {
        let result = OpenclawAgent.set_inference(
            HashMap::new(),
            Some(&inference::InferenceKind::OpenAi),
            Some("https://my-proxy.example.com/v1"),
            Some("gpt-4o"),
        );
        let json: serde_json::Value =
            serde_json::from_str(result[OPENCLAW_CONFIG_FILE].as_str()).unwrap();
        assert_eq!(
            json["models"]["providers"]["openai-custom"]["baseUrl"],
            "https://my-proxy.example.com/v1"
        );
    }

    #[test]
    fn env_vars_returns_empty() {
        let vars = OpenclawAgent.env_vars(None, None, None);
        assert!(vars.is_empty());
    }
}
