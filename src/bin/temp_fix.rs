/// Cost Analysis Report Component
#[component]
fn CostAnalysisReport(analytics_data: Signal<AnalyticsData>) -> Element {
    let provider_costs = use_resource(move || fetch_provider_costs());
    
    rsx! {
        div {
            h2 {
                style: "color: #FFC107; margin-bottom: 20px; font-size: 20px;",
                "💰 Cost Analysis & Provider Breakdown"
            }
            
            // Provider Cost Breakdown
            div {
                style: "background: #181E21; padding: 20px; border-radius: 8px; border: 1px solid #3e3e42; margin-bottom: 20px;",
                h3 {
                    style: "color: #cccccc; margin-bottom: 15px; font-size: 16px;",
                    "Provider Cost Breakdown"
                }
                div {
                    style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px;",
                    
                    if let Some(Ok(costs)) = provider_costs.read().as_ref() {
                        if costs.is_empty() {
                            div {
                                style: "background: #181E21; padding: 15px; border-radius: 6px; border: 1px solid #3e3e42; grid-column: 1 / -1;",
                                p { style: "color: #858585; text-align: center;", "No cost data available yet. Run some conversations to see cost breakdown." }
                            }
                        } else {
                            for (provider, cost, _count) in costs {
                                {
                                    let total = analytics_data.read().total_cost;
                                    let percentage = if total > 0.0 { cost / total * 100.0 } else { 0.0 };
                                
                                    div {
                                        style: "background: #181E21; padding: 15px; border-radius: 6px; border: 1px solid #3e3e42;",
                                        h4 { style: "margin: 0 0 8px 0; color: #FFC107; font-size: 12px;", "{provider}" }
                                        div { style: "font-size: 20px; font-weight: bold; color: #cccccc;", "${cost:.4}" }
                                        div { style: "font-size: 10px; color: #858585;", "{percentage:.1}% of total cost" }
                                    }
                                }
                            }
                        }
                    } else {
                        div {
                            style: "background: #181E21; padding: 15px; border-radius: 6px; border: 1px solid #3e3e42; grid-column: 1 / -1;",
                            p { style: "color: #858585; text-align: center;", "Loading provider costs..." }
                        }
                    }
                }
            }
            
            // Cost Optimization Recommendations
            div {
                style: "background: #181E21; padding: 20px; border-radius: 8px; border: 1px solid #3e3e42; margin-bottom: 20px;",
                h3 {
                    style: "color: #4caf50; margin-bottom: 15px; font-size: 16px;",
                    "💡 Optimization Recommendations"
                }
                div {
                    style: "display: grid; gap: 10px;",
                    
                    // Dynamic recommendations based on actual usage
                    if analytics_data.read().conversations_with_cost > 0 {
                        {
                            let avg_cost = analytics_data.read().total_cost / analytics_data.read().conversations_with_cost as f64;
                            let output_tokens = analytics_data.read().total_tokens_output;
                            let input_tokens = analytics_data.read().total_tokens_input;
                            
                            Fragment {
                                if avg_cost > 0.01 {
                                    div {
                                        style: "padding: 10px; background: #0E1414; border-radius: 4px; border-left: 3px solid #f44336;",
                                        div { style: "font-weight: bold; color: #f44336; margin-bottom: 5px;", "High cost per conversation detected" }
                                        div { style: "font-size: 12px; color: #cccccc;", 
                                            "Average cost: ${avg_cost:.4} per conversation. Consider using Claude 3 Haiku for simple queries."
                                        }
                                    }
                                }
                                
                                if output_tokens > input_tokens * 2 {
                                    div {
                                        style: "padding: 10px; background: #0E1414; border-radius: 4px; border-left: 3px solid #ff9800;",
                                        div { style: "font-weight: bold; color: #ff9800; margin-bottom: 5px;", "High output token usage" }
                                        div { style: "font-size: 12px; color: #cccccc;", 
                                            "Output tokens are 2x input tokens. Consider more concise prompts to reduce generation costs."
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    div {
                        style: "padding: 10px; background: #0E1414; border-radius: 4px; border-left: 3px solid #4caf50;",
                        div { style: "font-weight: bold; color: #4caf50; margin-bottom: 5px;", "Enable caching for repeated queries" }
                        div { style: "font-size: 12px; color: #cccccc;", "Save up to 70% on similar questions by caching consensus results" }
                    }
                }
            }
            
            // Budget Progress
            div {
                style: "background: #181E21; padding: 20px; border-radius: 8px; border: 1px solid #3e3e42;",
                h3 {
                    style: "color: #cccccc; margin-bottom: 15px; font-size: 16px;",
                    "Monthly Budget Progress"
                }
                {
                    let total_cost = analytics_data.read().total_cost;
                    let budget = 100.0;
                    let progress = (total_cost / budget * 100.0).min(100.0);
                    
                    div {
                        style: "margin-bottom: 10px;",
                        div { style: "display: flex; justify-content: space-between; margin-bottom: 5px;", 
                            span { style: "color: #cccccc;", "Current Month" }
                            span { style: "color: #FFC107;", "${total_cost:.2} / ${budget:.2}" }
                        }
                        div {
                            style: "background: #0E1414; height: 8px; border-radius: 4px; overflow: hidden;",
                            div {
                                style: format!("background: linear-gradient(90deg, #4caf50, #FFC107); height: 100%; width: {progress}%; transition: width 0.3s;"),
                            }
                        }
                    }
                    div { 
                        style: "font-size: 12px; color: #858585;",
                        "{progress as u32}% of monthly budget used"
                    }
                }
            }
        }
    }
}