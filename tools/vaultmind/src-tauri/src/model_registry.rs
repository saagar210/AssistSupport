use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedModel {
    pub name: String,
    pub description: String,
    pub use_case: String,
    pub min_ram_gb: u32,
    pub parameters: String,
    pub quantization: String,
}

/// Returns the full curated list of recommended Ollama models.
pub fn get_recommended_models() -> Vec<RecommendedModel> {
    vec![
        // Chat models
        RecommendedModel {
            name: "llama3.2:3b".to_string(),
            description: "Compact Llama 3.2 model, great for lightweight chat tasks".to_string(),
            use_case: "chat".to_string(),
            min_ram_gb: 4,
            parameters: "3B".to_string(),
            quantization: "Q4_K_M".to_string(),
        },
        RecommendedModel {
            name: "llama3.1:8b".to_string(),
            description: "Llama 3.1 8B, strong general-purpose chat model".to_string(),
            use_case: "chat".to_string(),
            min_ram_gb: 8,
            parameters: "8B".to_string(),
            quantization: "Q4_K_M".to_string(),
        },
        RecommendedModel {
            name: "mistral:7b".to_string(),
            description: "Mistral 7B, fast and capable chat model".to_string(),
            use_case: "chat".to_string(),
            min_ram_gb: 8,
            parameters: "7B".to_string(),
            quantization: "Q4_K_M".to_string(),
        },
        RecommendedModel {
            name: "deepseek-r1:7b".to_string(),
            description: "DeepSeek R1 7B, reasoning-focused chat model".to_string(),
            use_case: "chat".to_string(),
            min_ram_gb: 8,
            parameters: "7B".to_string(),
            quantization: "Q4_K_M".to_string(),
        },
        RecommendedModel {
            name: "phi4:14b".to_string(),
            description: "Microsoft Phi-4 14B, high-quality responses for complex tasks".to_string(),
            use_case: "chat".to_string(),
            min_ram_gb: 16,
            parameters: "14B".to_string(),
            quantization: "Q4_K_M".to_string(),
        },
        // Embedding models
        RecommendedModel {
            name: "nomic-embed-text".to_string(),
            description: "Nomic Embed Text, versatile embedding model with long context".to_string(),
            use_case: "embedding".to_string(),
            min_ram_gb: 1,
            parameters: "137M".to_string(),
            quantization: "F16".to_string(),
        },
        RecommendedModel {
            name: "mxbai-embed-large".to_string(),
            description: "MixedBread embed large, high-quality embeddings".to_string(),
            use_case: "embedding".to_string(),
            min_ram_gb: 2,
            parameters: "334M".to_string(),
            quantization: "F16".to_string(),
        },
        RecommendedModel {
            name: "all-minilm".to_string(),
            description: "All-MiniLM, lightweight and fast embedding model".to_string(),
            use_case: "embedding".to_string(),
            min_ram_gb: 1,
            parameters: "23M".to_string(),
            quantization: "F32".to_string(),
        },
        // Code models
        RecommendedModel {
            name: "qwen2.5-coder:7b".to_string(),
            description: "Qwen 2.5 Coder 7B, specialized for code generation and analysis".to_string(),
            use_case: "code".to_string(),
            min_ram_gb: 8,
            parameters: "7B".to_string(),
            quantization: "Q4_K_M".to_string(),
        },
        RecommendedModel {
            name: "deepseek-coder-v2:16b".to_string(),
            description: "DeepSeek Coder V2 16B, advanced code model for complex programming tasks".to_string(),
            use_case: "code".to_string(),
            min_ram_gb: 16,
            parameters: "16B".to_string(),
            quantization: "Q4_K_M".to_string(),
        },
    ]
}

/// Filter recommended models by use case (chat, embedding, code, general).
pub fn get_models_by_use_case(use_case: &str) -> Vec<RecommendedModel> {
    get_recommended_models()
        .into_iter()
        .filter(|m| m.use_case == use_case)
        .collect()
}

/// Filter recommended models that fit within the given RAM budget.
#[allow(dead_code)]
pub fn get_models_for_ram(available_ram_gb: u32) -> Vec<RecommendedModel> {
    get_recommended_models()
        .into_iter()
        .filter(|m| m.min_ram_gb <= available_ram_gb)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommended_models_not_empty() {
        let models = get_recommended_models();
        assert!(!models.is_empty(), "Recommended models list should not be empty");
        // Verify we have models across all use cases
        let use_cases: Vec<&str> = models.iter().map(|m| m.use_case.as_str()).collect();
        assert!(use_cases.contains(&"chat"), "Should have chat models");
        assert!(use_cases.contains(&"embedding"), "Should have embedding models");
        assert!(use_cases.contains(&"code"), "Should have code models");
    }

    #[test]
    fn test_filter_by_use_case() {
        let chat_models = get_models_by_use_case("chat");
        assert!(!chat_models.is_empty(), "Should have chat models");
        assert!(
            chat_models.iter().all(|m| m.use_case == "chat"),
            "All filtered models should be chat models"
        );

        let embedding_models = get_models_by_use_case("embedding");
        assert!(!embedding_models.is_empty(), "Should have embedding models");
        assert!(
            embedding_models.iter().all(|m| m.use_case == "embedding"),
            "All filtered models should be embedding models"
        );

        let code_models = get_models_by_use_case("code");
        assert!(!code_models.is_empty(), "Should have code models");
        assert!(
            code_models.iter().all(|m| m.use_case == "code"),
            "All filtered models should be code models"
        );

        let nonexistent = get_models_by_use_case("nonexistent");
        assert!(nonexistent.is_empty(), "Unknown use case should return empty");
    }

    #[test]
    fn test_filter_by_ram() {
        let tiny_ram = get_models_for_ram(0);
        assert!(tiny_ram.is_empty(), "0 GB RAM should return no models");

        let small_ram = get_models_for_ram(1);
        assert!(!small_ram.is_empty(), "1 GB RAM should return some models");
        assert!(
            small_ram.iter().all(|m| m.min_ram_gb <= 1),
            "All models should fit within 1 GB RAM"
        );

        let medium_ram = get_models_for_ram(8);
        assert!(
            medium_ram.len() > small_ram.len(),
            "8 GB RAM should return more models than 1 GB"
        );
        assert!(
            medium_ram.iter().all(|m| m.min_ram_gb <= 8),
            "All models should fit within 8 GB RAM"
        );

        let large_ram = get_models_for_ram(64);
        let all_models = get_recommended_models();
        assert_eq!(
            large_ram.len(),
            all_models.len(),
            "64 GB RAM should return all models"
        );
    }

    #[test]
    fn test_model_serialization() {
        let models = get_recommended_models();
        let first = &models[0];
        let json = serde_json::to_string(first).expect("Should serialize RecommendedModel");
        let deserialized: RecommendedModel =
            serde_json::from_str(&json).expect("Should deserialize RecommendedModel");
        assert_eq!(deserialized.name, first.name);
        assert_eq!(deserialized.use_case, first.use_case);
        assert_eq!(deserialized.min_ram_gb, first.min_ram_gb);
    }
}
