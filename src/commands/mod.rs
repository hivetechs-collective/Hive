//! Command modules for Hive AI CLI

pub mod analytics;
pub mod analyze;
pub mod consensus;
pub mod cost;
pub mod hooks;
pub mod improve;
pub mod index;
pub mod install;
pub mod lsp;
pub mod maintenance;
pub mod memory;
pub mod migrate;
pub mod models;
pub mod performance;
pub mod planning;
pub mod search;
pub mod security;
pub mod shell;
// pub mod mcp; // Temporarily disabled
// pub mod mode; // Temporarily disabled

pub use analyze::{handle_analyze, handle_dependency_analysis, handle_find_circular_deps};
pub use consensus::{
    handle_consensus_benchmark, handle_consensus_metrics, handle_consensus_test,
    handle_stage_prompts, handle_temporal_test,
};
pub use cost::estimate_cost;
pub use hooks::{generate_hook_examples, handle_hooks, show_available_events};
pub use improve::{
    handle_improve, handle_redo, handle_transform_history, handle_undo, list_aspects,
};
pub use index::{handle_index_build, handle_index_stats};
pub use install::{handle_install_command, InstallArgs};
pub use lsp::{generate_editor_config, handle_lsp, LspCommands};
pub use maintenance::run_maintenance_command;
pub use migrate::handle_migrate;
pub use performance::run_benchmarks;
pub use planning::{
    handle_analyze_risks, handle_collaborate, handle_decompose, handle_plan, handle_timeline,
};
pub use search::{handle_call_graph, handle_references, handle_search};
pub use security::{handle_security, SecurityCommands};
pub use shell::{
    generate_manual_instructions, get_integration_report, handle_shell, verify_installation,
    ShellCommands,
};
// pub use mcp::{
//     start_server as mcp_start_server, check_status as mcp_check_status,
//     list_tools as mcp_list_tools, test_tool as mcp_test_tool,
//     show_logs as mcp_show_logs, list_resources as mcp_list_resources,
//     show_protocol_info as mcp_protocol_info
// }; // Temporarily disabled
// pub use mode::{ModeCommands, execute as execute_mode_command}; // Temporarily disabled
