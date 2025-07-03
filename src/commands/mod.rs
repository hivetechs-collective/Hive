//! Command modules for Hive AI CLI

pub mod search;
pub mod analyze;
pub mod index;
pub mod cost;
pub mod performance;
pub mod models;
pub mod improve;
pub mod memory;
pub mod analytics;
pub mod hooks;
pub mod consensus;
pub mod planning;
pub mod lsp;
pub mod security;
pub mod migrate;
pub mod shell;
// pub mod mcp; // Temporarily disabled
// pub mod mode; // Temporarily disabled

pub use search::{handle_search, handle_references, handle_call_graph};
pub use analyze::{handle_analyze, handle_dependency_analysis, handle_find_circular_deps};
pub use index::{handle_index_build, handle_index_stats};
pub use cost::estimate_cost;
pub use performance::run_benchmarks;
pub use improve::{handle_improve, handle_undo, handle_redo, handle_transform_history, list_aspects};
pub use hooks::{handle_hooks, generate_hook_examples, show_available_events};
pub use consensus::{
    handle_consensus_test, handle_temporal_test, handle_consensus_metrics,
    handle_stage_prompts, handle_consensus_benchmark
};
pub use migrate::handle_migrate;
pub use planning::{
    handle_plan, handle_decompose, handle_analyze_risks, 
    handle_timeline, handle_collaborate
};
pub use lsp::{handle_lsp, LspCommands, generate_editor_config};
pub use security::{handle_security, SecurityCommands};
pub use shell::{handle_shell, ShellCommands, generate_manual_instructions, verify_installation, get_integration_report};
// pub use mcp::{
//     start_server as mcp_start_server, check_status as mcp_check_status,
//     list_tools as mcp_list_tools, test_tool as mcp_test_tool,
//     show_logs as mcp_show_logs, list_resources as mcp_list_resources,
//     show_protocol_info as mcp_protocol_info
// }; // Temporarily disabled
// pub use mode::{ModeCommands, execute as execute_mode_command}; // Temporarily disabled