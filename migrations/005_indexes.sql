-- @ version: 005
-- @ name: indexes
-- @ description: Create all performance indexes for optimal query performance
-- @ rollback: DROP INDEX IF EXISTS idx_activity_log_stage; DROP INDEX IF EXISTS idx_activity_log_user; DROP INDEX IF EXISTS idx_activity_log_conversation; DROP INDEX IF EXISTS idx_activity_log_created; DROP INDEX IF EXISTS idx_activity_log_event; DROP INDEX IF EXISTS idx_performance_metrics_cost; DROP INDEX IF EXISTS idx_performance_metrics_duration; DROP INDEX IF EXISTS idx_performance_metrics_timestamp; DROP INDEX IF EXISTS idx_performance_metrics_conv; DROP INDEX IF EXISTS idx_model_selection_created; DROP INDEX IF EXISTS idx_model_selection_feedback; DROP INDEX IF EXISTS idx_model_selection_template; DROP INDEX IF EXISTS idx_model_selection_conv; DROP INDEX IF EXISTS idx_profile_templates_expert; DROP INDEX IF EXISTS idx_profile_templates_usage; DROP INDEX IF EXISTS idx_profile_templates_effectiveness; DROP INDEX IF EXISTS idx_profile_templates_auto_update; DROP INDEX IF EXISTS idx_profile_templates_name; DROP INDEX IF EXISTS idx_feature_usage_switches; DROP INDEX IF EXISTS idx_feature_usage_fallbacks; DROP INDEX IF EXISTS idx_feature_usage_created; DROP INDEX IF EXISTS idx_feature_usage_conv; DROP INDEX IF EXISTS idx_cost_analytics_budget; DROP INDEX IF EXISTS idx_cost_analytics_created; DROP INDEX IF EXISTS idx_cost_analytics_efficiency; DROP INDEX IF EXISTS idx_cost_analytics_cost; DROP INDEX IF EXISTS idx_cost_analytics_conv; DROP INDEX IF EXISTS idx_consensus_metrics_created; DROP INDEX IF EXISTS idx_consensus_metrics_rating; DROP INDEX IF EXISTS idx_consensus_metrics_category; DROP INDEX IF EXISTS idx_consensus_metrics_complexity; DROP INDEX IF EXISTS idx_consensus_metrics_improvement; DROP INDEX IF EXISTS idx_consensus_metrics_conv; DROP INDEX IF EXISTS idx_model_rankings_score; DROP INDEX IF EXISTS idx_model_rankings_period; DROP INDEX IF EXISTS idx_model_rankings_collected; DROP INDEX IF EXISTS idx_model_rankings_position; DROP INDEX IF EXISTS idx_model_rankings_source; DROP INDEX IF EXISTS idx_model_rankings_model; DROP INDEX IF EXISTS idx_provider_performance_latency; DROP INDEX IF EXISTS idx_provider_performance_success; DROP INDEX IF EXISTS idx_provider_performance_measured; DROP INDEX IF EXISTS idx_provider_performance_variant; DROP INDEX IF EXISTS idx_provider_performance_provider; DROP INDEX IF EXISTS idx_provider_performance_model; DROP INDEX IF EXISTS idx_pending_sync_attempts; DROP INDEX IF EXISTS idx_pending_sync_created; DROP INDEX IF EXISTS idx_pending_sync_user; DROP INDEX IF EXISTS idx_conversation_threads_created; DROP INDEX IF EXISTS idx_conversation_threads_type; DROP INDEX IF EXISTS idx_conversation_threads_parent; DROP INDEX IF EXISTS idx_conversation_threads_child; DROP INDEX IF EXISTS idx_improvement_patterns_stages; DROP INDEX IF EXISTS idx_improvement_patterns_question_type; DROP INDEX IF EXISTS idx_improvement_patterns_type; DROP INDEX IF EXISTS idx_improvement_patterns_conv; DROP INDEX IF EXISTS idx_conversation_keywords_conv; DROP INDEX IF EXISTS idx_conversation_topics_conv; DROP INDEX IF EXISTS idx_conversation_context_type; DROP INDEX IF EXISTS idx_conversation_context_ref; DROP INDEX IF EXISTS idx_conversation_context_conv; DROP INDEX IF EXISTS idx_curator_truths_confidence; DROP INDEX IF EXISTS idx_curator_truths_conv; DROP INDEX IF EXISTS idx_knowledge_conversations_question; DROP INDEX IF EXISTS idx_knowledge_conversations_created; DROP INDEX IF EXISTS idx_knowledge_conversations_id; DROP INDEX IF EXISTS idx_usage_timestamp_legacy; DROP INDEX IF EXISTS idx_usage_conversation; DROP INDEX IF EXISTS idx_profiles_default; DROP INDEX IF EXISTS idx_profiles_name; DROP INDEX IF EXISTS idx_models_provider_name; DROP INDEX IF EXISTS idx_models_name; DROP INDEX IF EXISTS idx_budget_user; DROP INDEX IF EXISTS idx_usage_timestamp; DROP INDEX IF EXISTS idx_usage_user; DROP INDEX IF EXISTS idx_messages_conversation; DROP INDEX IF EXISTS idx_conversations_profile; DROP INDEX IF EXISTS idx_conversations_user; DROP INDEX IF EXISTS idx_consensus_profiles_pipeline; DROP INDEX IF EXISTS idx_pipeline_profiles_user; DROP INDEX IF EXISTS idx_models_provider; DROP INDEX IF EXISTS idx_configurations_user;

-- Foreign key indexes
CREATE INDEX idx_configurations_user ON configurations(user_id);
CREATE INDEX idx_models_provider ON openrouter_models(provider_id);
CREATE INDEX idx_pipeline_profiles_user ON pipeline_profiles(user_id);
CREATE INDEX idx_consensus_profiles_pipeline ON consensus_profiles(pipeline_profile_id);
CREATE INDEX idx_conversations_user ON conversations(user_id);
CREATE INDEX idx_conversations_profile ON conversations(consensus_profile_id);
CREATE INDEX idx_messages_conversation ON messages(conversation_id);
CREATE INDEX idx_usage_user ON usage_records(user_id);
CREATE INDEX idx_usage_timestamp ON usage_records(timestamp);
CREATE INDEX idx_budget_user ON budget_limits(user_id);

-- Search indexes
CREATE INDEX idx_models_name ON openrouter_models(name);
CREATE INDEX idx_models_provider_name ON openrouter_models(provider_name);
CREATE INDEX idx_profiles_name ON pipeline_profiles(name);
CREATE INDEX idx_profiles_default ON pipeline_profiles(is_default);
CREATE INDEX idx_usage_conversation ON conversation_usage(conversation_id);
CREATE INDEX idx_usage_timestamp_legacy ON conversation_usage(timestamp);

-- Knowledge database indexes
CREATE INDEX idx_knowledge_conversations_id ON knowledge_conversations(conversation_id);
CREATE INDEX idx_knowledge_conversations_created ON knowledge_conversations(created_at DESC);
CREATE INDEX idx_knowledge_conversations_question ON knowledge_conversations(question);
CREATE INDEX idx_curator_truths_conv ON curator_truths(conversation_id);
CREATE INDEX idx_curator_truths_confidence ON curator_truths(confidence_score DESC);
CREATE INDEX idx_conversation_context_conv ON conversation_context(conversation_id);
CREATE INDEX idx_conversation_context_ref ON conversation_context(referenced_conversation_id);
CREATE INDEX idx_conversation_context_type ON conversation_context(context_type);
CREATE INDEX idx_conversation_topics_conv ON conversation_topics(conversation_id);
CREATE INDEX idx_conversation_keywords_conv ON conversation_keywords(conversation_id);

-- Improvement patterns indexes
CREATE INDEX idx_improvement_patterns_conv ON improvement_patterns(conversation_id);
CREATE INDEX idx_improvement_patterns_type ON improvement_patterns(improvement_type);
CREATE INDEX idx_improvement_patterns_question_type ON improvement_patterns(question_type);
CREATE INDEX idx_improvement_patterns_stages ON improvement_patterns(from_stage, to_stage);

-- Pending sync indexes
CREATE INDEX idx_conversation_threads_child ON conversation_threads(child_conversation_id);
CREATE INDEX idx_conversation_threads_parent ON conversation_threads(parent_conversation_id);
CREATE INDEX idx_conversation_threads_type ON conversation_threads(thread_type);
CREATE INDEX idx_conversation_threads_created ON conversation_threads(created_at DESC);

CREATE INDEX idx_pending_sync_user ON pending_sync(user_id);
CREATE INDEX idx_pending_sync_created ON pending_sync(created_at);
CREATE INDEX idx_pending_sync_attempts ON pending_sync(sync_attempts);

-- ===== DYNAMIC CONSENSUS & OPENROUTER INTELLIGENCE INDEXES =====

-- Provider performance indexes
CREATE INDEX idx_provider_performance_model ON provider_performance(model_internal_id);
CREATE INDEX idx_provider_performance_provider ON provider_performance(provider_name);
CREATE INDEX idx_provider_performance_variant ON provider_performance(routing_variant);
CREATE INDEX idx_provider_performance_measured ON provider_performance(measured_at DESC);
CREATE INDEX idx_provider_performance_success ON provider_performance(success_rate DESC);
CREATE INDEX idx_provider_performance_latency ON provider_performance(avg_latency_ms ASC);

-- Model rankings indexes
CREATE INDEX idx_model_rankings_model ON model_rankings(model_internal_id);
CREATE INDEX idx_model_rankings_source ON model_rankings(ranking_source);
CREATE INDEX idx_model_rankings_position ON model_rankings(rank_position ASC);
CREATE INDEX idx_model_rankings_collected ON model_rankings(collected_at DESC);
CREATE INDEX idx_model_rankings_period ON model_rankings(period_start, period_end);
CREATE INDEX idx_model_rankings_score ON model_rankings(relative_score DESC);

-- Consensus metrics indexes
CREATE INDEX idx_consensus_metrics_conv ON consensus_metrics(conversation_id);
CREATE INDEX idx_consensus_metrics_improvement ON consensus_metrics(improvement_score DESC);
CREATE INDEX idx_consensus_metrics_complexity ON consensus_metrics(question_complexity);
CREATE INDEX idx_consensus_metrics_category ON consensus_metrics(question_category);
CREATE INDEX idx_consensus_metrics_rating ON consensus_metrics(user_rating DESC);
CREATE INDEX idx_consensus_metrics_created ON consensus_metrics(created_at DESC);

-- Cost analytics indexes
CREATE INDEX idx_cost_analytics_conv ON cost_analytics(conversation_id);
CREATE INDEX idx_cost_analytics_cost ON cost_analytics(total_cost DESC);
CREATE INDEX idx_cost_analytics_efficiency ON cost_analytics(cost_efficiency_score DESC);
CREATE INDEX idx_cost_analytics_created ON cost_analytics(created_at DESC);
CREATE INDEX idx_cost_analytics_budget ON cost_analytics(budget_utilized_percentage DESC);

-- Feature usage indexes
CREATE INDEX idx_feature_usage_conv ON feature_usage(conversation_id);
CREATE INDEX idx_feature_usage_created ON feature_usage(created_at DESC);
CREATE INDEX idx_feature_usage_fallbacks ON feature_usage(fallback_events DESC);
CREATE INDEX idx_feature_usage_switches ON feature_usage(provider_switches DESC);

-- Profile templates indexes
CREATE INDEX idx_profile_templates_name ON profile_templates(name);
CREATE INDEX idx_profile_templates_auto_update ON profile_templates(auto_update);
CREATE INDEX idx_profile_templates_effectiveness ON profile_templates(effectiveness_score DESC);
CREATE INDEX idx_profile_templates_usage ON profile_templates(usage_count DESC);
CREATE INDEX idx_profile_templates_expert ON profile_templates(expert_mode);

-- Model selection history indexes
CREATE INDEX idx_model_selection_conv ON model_selection_history(conversation_id);
CREATE INDEX idx_model_selection_template ON model_selection_history(template_used);
CREATE INDEX idx_model_selection_feedback ON model_selection_history(user_feedback DESC);
CREATE INDEX idx_model_selection_created ON model_selection_history(created_at DESC);

-- Performance metrics indexes
CREATE INDEX idx_performance_metrics_conv ON performance_metrics(conversation_id);
CREATE INDEX idx_performance_metrics_timestamp ON performance_metrics(timestamp DESC);
CREATE INDEX idx_performance_metrics_duration ON performance_metrics(total_duration);
CREATE INDEX idx_performance_metrics_cost ON performance_metrics(total_cost);

-- Activity log indexes for efficient dashboard queries
CREATE INDEX idx_activity_log_event ON activity_log(event_type);
CREATE INDEX idx_activity_log_created ON activity_log(created_at DESC);
CREATE INDEX idx_activity_log_conversation ON activity_log(conversation_id);
CREATE INDEX idx_activity_log_user ON activity_log(user_id);
CREATE INDEX idx_activity_log_stage ON activity_log(stage);