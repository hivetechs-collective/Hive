//! Model browser and selection UI for consensus profiles
//!
//! Provides a rich interface for browsing and selecting models from OpenRouter
//! with search, filtering, and intelligent recommendations.

use anyhow::Result;
use dioxus::prelude::*;
use rusqlite::{params, Connection};
use std::collections::HashMap;

/// Model information from the database
#[derive(Debug, Clone, PartialEq)]
pub struct ModelInfo {
    pub internal_id: i64,
    pub openrouter_id: String,
    pub name: String,
    pub provider_name: String,
    pub description: Option<String>,
    pub context_window: Option<i64>,
    pub pricing_input: f64,
    pub pricing_output: f64,
    pub is_active: bool,
}

/// Stage recommendations for model selection
#[derive(Debug, Clone)]
pub struct StageRecommendation {
    pub stage: &'static str,
    pub purpose: &'static str,
    pub best_for: Vec<&'static str>,
    pub temperature_range: (f32, f32),
    pub hints: Vec<&'static str>,
}

/// Get stage recommendations
pub fn get_stage_recommendations() -> HashMap<&'static str, StageRecommendation> {
    let mut recommendations = HashMap::new();

    recommendations.insert(
        "generator",
        StageRecommendation {
            stage: "Generator",
            purpose: "Initial analysis and creative problem decomposition",
            best_for: vec!["Complex reasoning", "Creative thinking", "Problem analysis"],
            temperature_range: (0.6, 0.9),
            hints: vec![
                "Best for initial brainstorming and analysis",
                "Models like Claude 3.5 Sonnet, GPT-4, DeepSeek R1 excel here",
                "Higher creativity helps with ideation",
            ],
        },
    );

    recommendations.insert(
        "refiner",
        StageRecommendation {
            stage: "Refiner",
            purpose: "Technical enhancement and specification",
            best_for: vec![
                "Code generation",
                "Technical depth",
                "Implementation details",
            ],
            temperature_range: (0.2, 0.5),
            hints: vec![
                "Perfect for technical refinement and coding",
                "GPT-4, Codestral, Claude Sonnet work well",
                "Lower temperature for precise responses",
            ],
        },
    );

    recommendations.insert(
        "validator",
        StageRecommendation {
            stage: "Validator",
            purpose: "Fact-checking and validation with different perspective",
            best_for: vec!["Fact checking", "Error detection", "Quality assurance"],
            temperature_range: (0.0, 0.3),
            hints: vec![
                "Choose models with different training data",
                "Gemini, Llama, Qwen provide good validation",
                "Very low temperature for consistent checking",
            ],
        },
    );

    recommendations.insert(
        "curator",
        StageRecommendation {
            stage: "Curator",
            purpose: "Final synthesis and polishing",
            best_for: vec!["Writing quality", "Communication", "Final polish"],
            temperature_range: (0.3, 0.7),
            hints: vec![
                "Excels at final writing and communication",
                "Claude Sonnet, Grok, GPT-4 are excellent here",
                "Moderate temperature for natural output",
            ],
        },
    );

    recommendations
}

/// Load all models from the database
pub async fn load_all_models() -> Result<Vec<ModelInfo>> {
    use crate::core::database::get_database;

    let db = get_database().await?;
    let conn = db.get_connection()?;

    let mut models = Vec::new();

    {
        let mut stmt = conn.prepare(
            "SELECT internal_id, openrouter_id, name, provider_name, description,
                    context_window, pricing_input, pricing_output, is_active
             FROM openrouter_models
             WHERE is_active = 1
             ORDER BY provider_name, name",
        )?;

        let model_iter = stmt.query_map([], |row| {
            Ok(ModelInfo {
                internal_id: row.get(0)?,
                openrouter_id: row.get(1)?,
                name: row.get(2)?,
                provider_name: row.get(3)?,
                description: row.get(4)?,
                context_window: row.get(5)?,
                pricing_input: row.get(6)?,
                pricing_output: row.get(7)?,
                is_active: row.get(8)?,
            })
        })?;

        for model in model_iter {
            models.push(model?);
        }
    }

    Ok(models)
}

/// Get top recommended models for a stage
pub fn get_top_recommendations(models: &[ModelInfo], stage: &str, limit: usize) -> Vec<ModelInfo> {
    // Popular providers for quality
    let popular_providers = [
        "openai",
        "anthropic",
        "google",
        "meta-llama",
        "mistralai",
        "deepseek",
    ];

    let mut scored_models: Vec<(ModelInfo, i32)> = models
        .iter()
        .filter(|m| m.is_active)
        .map(|model| {
            let provider = model.openrouter_id.split('/').next().unwrap_or("");

            // Calculate score based on various factors
            let mut score = 0;

            // Free models get bonus points
            if model.pricing_input == 0.0 && model.pricing_output == 0.0 {
                score += 50;
            }

            // Popular providers get high scores
            if popular_providers.contains(&provider) {
                score += 100;
            }

            // Stage-specific scoring
            match stage {
                "generator" => {
                    // Prefer creative models with large context
                    if model.name.contains("Claude")
                        || model.name.contains("GPT-4")
                        || model.name.contains("DeepSeek")
                    {
                        score += 200;
                    }
                    if let Some(ctx) = model.context_window {
                        if ctx > 100000 {
                            score += 50;
                        }
                    }
                }
                "refiner" => {
                    // Prefer technical/coding models
                    if model.name.contains("Code")
                        || model.name.contains("GPT-4")
                        || model.name.contains("Claude")
                    {
                        score += 200;
                    }
                }
                "validator" => {
                    // Prefer diverse providers for validation
                    if provider == "google" || provider == "meta-llama" || provider == "qwen" {
                        score += 150;
                    }
                }
                "curator" => {
                    // Prefer writing-focused models
                    if model.name.contains("Claude")
                        || model.name.contains("GPT")
                        || model.name.contains("Grok")
                    {
                        score += 200;
                    }
                }
                _ => {}
            }

            (model.clone(), score)
        })
        .collect();

    // Sort by score descending
    scored_models.sort_by(|a, b| b.1.cmp(&a.1));

    // Ensure provider diversity by limiting models per provider
    let mut provider_counts: HashMap<String, usize> = HashMap::new();
    let mut results = Vec::new();

    for (model, _score) in scored_models {
        let provider = model
            .openrouter_id
            .split('/')
            .next()
            .unwrap_or("")
            .to_string();
        let count = provider_counts.entry(provider.clone()).or_insert(0);

        if *count < 2 {
            // Max 2 models per provider
            results.push(model);
            *count += 1;

            if results.len() >= limit {
                break;
            }
        }
    }

    results
}

/// Model browser dialog component
#[component]
pub fn ModelBrowserDialog(
    show_browser: Signal<bool>,
    stage_name: String,
    stage_key: String,
    on_select: EventHandler<ModelInfo>,
) -> Element {
    let mut models = use_signal(|| Vec::<ModelInfo>::new());
    let mut filtered_models = use_signal(|| Vec::<ModelInfo>::new());
    let mut search_query = use_signal(String::new);
    let mut selected_provider = use_signal(|| String::from("all"));
    let mut show_free_only = use_signal(|| false);
    let mut loading = use_signal(|| true);
    let mut error_message = use_signal(|| None::<String>);

    // Load models on first render
    use_effect({
        let stage_key_clone = stage_key.clone();
        move || {
            let stage_key = stage_key_clone.clone();
            spawn(async move {
                match load_all_models().await {
                    Ok(loaded_models) => {
                        let top_recommendations =
                            get_top_recommendations(&loaded_models, &stage_key, 10);
                        *filtered_models.write() = top_recommendations.clone();
                        *models.write() = loaded_models;
                        *loading.write() = false;
                    }
                    Err(e) => {
                        *error_message.write() = Some(format!("Failed to load models: {}", e));
                        *loading.write() = false;
                    }
                }
            });
        }
    });

    // Filter models when search/filters change
    use_effect({
        let stage_key_clone = stage_key.clone();
        move || {
            let all_models = models.read().clone();
            let query = search_query.read().to_lowercase();
            let provider = selected_provider.read().clone();
            let free_only = *show_free_only.read();
            let stage_key = stage_key_clone.clone();

            let mut filtered: Vec<ModelInfo> = all_models
                .into_iter()
                .filter(|model| {
                    // Provider filter
                    if provider != "all" && model.provider_name != provider {
                        return false;
                    }

                    // Free filter
                    if free_only && (model.pricing_input > 0.0 || model.pricing_output > 0.0) {
                        return false;
                    }

                    // Search filter
                    if !query.is_empty() {
                        let search_text = format!(
                            "{} {} {}",
                            model.name.to_lowercase(),
                            model.provider_name.to_lowercase(),
                            model
                                .description
                                .as_ref()
                                .unwrap_or(&String::new())
                                .to_lowercase()
                        );
                        if !search_text.contains(&query) {
                            return false;
                        }
                    }

                    true
                })
                .collect();

            // If no search query and showing all providers, show top recommendations
            if query.is_empty() && provider == "all" && !free_only {
                filtered = get_top_recommendations(&filtered, &stage_key, 20);
            }

            *filtered_models.write() = filtered;
        }
    });

    // Get unique providers for filter dropdown
    let providers = {
        let mut provider_set: Vec<String> = models
            .read()
            .iter()
            .map(|m| m.provider_name.clone())
            .collect();
        provider_set.sort();
        provider_set.dedup();
        provider_set
    };

    let recommendations = get_stage_recommendations();
    let stage_info = recommendations.get(stage_key.as_str());

    rsx! {
        if *show_browser.read() {
            div {
                class: "dialog-overlay",
                onclick: move |_| *show_browser.write() = false,

                div {
                    class: "dialog-content model-browser",
                    style: "width: 900px; max-width: 90vw; height: 700px; max-height: 90vh; display: flex; flex-direction: column;",
                    onclick: move |e| e.stop_propagation(),

                    // Header
                    div {
                        class: "dialog-header",
                        style: "padding: 20px; border-bottom: 1px solid #3e3e42;",
                        h2 {
                            style: "margin: 0 0 10px 0;",
                            "🎯 Select Model for {stage_name}"
                        }
                        if let Some(info) = stage_info {
                            div {
                                style: "color: #cccccc; font-size: 14px; margin-bottom: 10px;",
                                "{info.purpose}"
                            }
                            div {
                                style: "display: flex; gap: 20px; font-size: 13px; color: #858585;",
                                div {
                                    "💡 Best for: {info.best_for.join(\", \")}"
                                }
                                div {
                                    "🌡️ Temperature: {info.temperature_range.0}-{info.temperature_range.1}"
                                }
                            }
                        }
                    }

                    // Filters
                    div {
                        style: "padding: 15px 20px; background: #2d2d30; border-bottom: 1px solid #3e3e42;",
                        div {
                            style: "display: flex; gap: 15px; align-items: center;",

                            // Search
                            input {
                                r#type: "text",
                                placeholder: "Search models...",
                                value: "{search_query.read()}",
                                style: "flex: 1; padding: 8px 12px; background: #3c3c3c; border: 1px solid #3e3e42; border-radius: 4px; color: #cccccc;",
                                oninput: move |evt| *search_query.write() = evt.value(),
                            }

                            // Provider filter
                            select {
                                style: "padding: 8px 12px; background: #3c3c3c; border: 1px solid #3e3e42; border-radius: 4px; color: #cccccc;",
                                value: "{selected_provider.read()}",
                                onchange: move |evt| *selected_provider.write() = evt.value(),
                                option { value: "all", "All Providers" }
                                for provider in &providers {
                                    option { value: "{provider}", "{provider}" }
                                }
                            }

                            // Free only checkbox
                            label {
                                style: "display: flex; align-items: center; gap: 5px; cursor: pointer;",
                                input {
                                    r#type: "checkbox",
                                    checked: *show_free_only.read(),
                                    onchange: move |evt| *show_free_only.write() = evt.checked(),
                                }
                                "🆓 Free only"
                            }
                        }
                    }

                    // Model list
                    div {
                        style: "flex: 1; overflow-y: auto; padding: 20px;",

                        if *loading.read() {
                            div {
                                style: "text-align: center; padding: 50px; color: #858585;",
                                "Loading models..."
                            }
                        } else if let Some(error) = error_message.read().as_ref() {
                            div {
                                class: "error",
                                style: "margin: 20px;",
                                "{error}"
                            }
                        } else if filtered_models.read().is_empty() {
                            div {
                                style: "text-align: center; padding: 50px; color: #858585;",
                                "No models found matching your criteria"
                            }
                        } else {
                            for model in filtered_models.read().iter() {
                                ModelCard {
                                    model: model.clone(),
                                    on_select: move |model| {
                                        on_select.call(model);
                                        *show_browser.write() = false;
                                    }
                                }
                            }
                        }
                    }

                    // Footer
                    div {
                        style: "padding: 15px 20px; border-top: 1px solid #3e3e42; text-align: right;",
                        button {
                            class: "button-secondary",
                            onclick: move |_| *show_browser.write() = false,
                            "Cancel"
                        }
                    }
                }
            }
        }
    }
}

/// Individual model card component
#[component]
fn ModelCard(model: ModelInfo, on_select: EventHandler<ModelInfo>) -> Element {
    let is_free = model.pricing_input == 0.0 && model.pricing_output == 0.0;
    let cost_per_1k = (model.pricing_input + model.pricing_output) * 1000.0;

    rsx! {
        div {
            class: "model-card",
            style: "padding: 15px; margin-bottom: 10px; background: #2d2d30; border: 1px solid #3e3e42; border-radius: 6px; cursor: pointer; transition: all 0.2s;",
            onmouseover: move |_e| {
                // Hover styling handled by CSS
            },
            onmouseout: move |_e| {
                // Hover styling handled by CSS
            },
            onclick: move |_| on_select.call(model.clone()),

            div {
                style: "display: flex; justify-content: space-between; align-items: start; margin-bottom: 8px;",
                div {
                    h4 {
                        style: "margin: 0; color: #ffffff; font-size: 16px;",
                        "{model.name}"
                    }
                    div {
                        style: "color: #858585; font-size: 13px; margin-top: 2px;",
                        "{model.provider_name} • {model.openrouter_id}"
                    }
                }
                div {
                    style: "text-align: right;",
                    if is_free {
                        span {
                            style: "color: #4caf50; font-weight: 600;",
                            "🆓 FREE"
                        }
                    } else {
                        span {
                            style: "color: #cccccc;",
                            "💰 ${cost_per_1k:.4}/1k"
                        }
                    }
                    if let Some(ctx) = model.context_window {
                        div {
                            style: "color: #858585; font-size: 12px; margin-top: 2px;",
                            "{ctx} tokens"
                        }
                    }
                }
            }

            if let Some(desc) = &model.description {
                p {
                    style: "margin: 0; color: #cccccc; font-size: 13px; line-height: 1.4;",
                    "{desc}"
                }
            }
        }
    }
}
