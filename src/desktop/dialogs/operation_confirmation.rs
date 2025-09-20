//! Operation Confirmation Dialog
//!
//! Displays file operations that require user confirmation before execution.
//! Users can approve or reject individual operations or all operations at once.

use crate::consensus::ai_operation_parser::FileOperationWithMetadata;
use crate::consensus::stages::file_aware_curator::FileOperation;
use crate::desktop::components::operation_preview::OperationPreview;
use crate::desktop::styles::theme::ThemeColors;
use dioxus::prelude::*;

/// Props for the operation confirmation dialog
#[component]
pub fn OperationConfirmationDialog(
    operations: Vec<FileOperationWithMetadata>,
    on_approve: EventHandler<Vec<FileOperation>>,
    on_reject: EventHandler<()>,
    theme: ThemeColors,
) -> Element {
    // Track which operations are selected for approval
    let mut selected_operations = use_signal(|| {
        // By default, select all operations
        operations.iter().map(|_| true).collect::<Vec<bool>>()
    });

    // Track if we're showing detailed previews
    let mut show_previews = use_signal(|| true);

    // Calculate how many operations are selected
    let selected_count = selected_operations.read().iter().filter(|&&x| x).count();
    let total_count = operations.len();

    rsx! {
        // Modal overlay
        div {
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0, 0, 0, 0.8); display: flex; align-items: center; justify-content: center; z-index: 10000;",
            onclick: move |_| {
                // Clicking outside closes the dialog (reject)
                on_reject.call(());
            },

            // Dialog container
            div {
                style: "background: {theme.background}; border: 1px solid {theme.border}; border-radius: 8px; padding: 24px; max-width: 800px; max-height: 80vh; overflow: hidden; display: flex; flex-direction: column; box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);",
                onclick: move |e| {
                    // Prevent closing when clicking inside the dialog
                    e.stop_propagation();
                },

                // Header
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;",

                    h2 {
                        style: "margin: 0; color: {theme.text}; font-size: 20px;",
                        "Confirm File Operations"
                    }

                    button {
                        style: "background: none; border: none; color: {theme.text_secondary}; font-size: 24px; cursor: pointer; padding: 0; width: 32px; height: 32px; display: flex; align-items: center; justify-content: center; border-radius: 4px; transition: all 0.2s;",
                        onclick: move |_| {
                            on_reject.call(());
                        },
                        "×"
                    }
                }

                // Info bar
                div {
                    style: "background: {theme.background_secondary}; padding: 12px 16px; border-radius: 6px; margin-bottom: 16px;",

                    div {
                        style: "display: flex; justify-content: space-between; align-items: center;",

                        div {
                            style: "color: {theme.text};",
                            "{selected_count} of {total_count} operations selected"
                        }

                        div {
                            style: "display: flex; gap: 12px;",

                            button {
                                style: "background: none; border: none; color: {theme.primary}; cursor: pointer; font-size: 14px; text-decoration: underline;",
                                onclick: move |_| {
                                    // Select all
                                    selected_operations.write().iter_mut().for_each(|x| *x = true);
                                },
                                "Select All"
                            }

                            button {
                                style: "background: none; border: none; color: {theme.primary}; cursor: pointer; font-size: 14px; text-decoration: underline;",
                                onclick: move |_| {
                                    // Select none
                                    selected_operations.write().iter_mut().for_each(|x| *x = false);
                                },
                                "Select None"
                            }

                            button {
                                style: "background: none; border: none; color: {theme.primary}; cursor: pointer; font-size: 14px; text-decoration: underline;",
                                onclick: move |_| {
                                    let current = *show_previews.read();
                                    *show_previews.write() = !current;
                                },
                                if *show_previews.read() { "Hide Previews" } else { "Show Previews" }
                            }
                        }
                    }
                }

                // Operations list (scrollable)
                div {
                    style: "flex: 1; overflow-y: auto; margin-bottom: 16px; max-height: 400px;",

                    for (idx, op_with_metadata) in operations.iter().enumerate() {
                        div {
                            style: "margin-bottom: 12px; padding: 12px; background: {theme.background_secondary}; border-radius: 6px; border: 1px solid {theme.border};",

                            // Operation header with checkbox
                            div {
                                style: "display: flex; align-items: center; gap: 12px; margin-bottom: 8px;",

                                input {
                                    r#type: "checkbox",
                                    checked: selected_operations.read()[idx],
                                    onchange: move |_| {
                                        let current = selected_operations.read()[idx];
                                        selected_operations.write()[idx] = !current;
                                    }
                                }

                                div {
                                    style: "flex: 1;",

                                    // Operation type and path
                                    div {
                                        style: "display: flex; align-items: center; gap: 8px;",

                                        span {
                                            style: "font-weight: bold; color: {get_operation_color(&op_with_metadata.operation, &theme)};",
                                            {get_operation_type_display(&op_with_metadata.operation)}
                                        }

                                        span {
                                            style: "color: {theme.text}; font-family: monospace;",
                                            {get_operation_path(&op_with_metadata.operation)}
                                        }
                                    }

                                    // Confidence and rationale
                                    div {
                                        style: "display: flex; align-items: center; gap: 16px; margin-top: 4px;",

                                        span {
                                            style: "color: {theme.text_secondary}; font-size: 12px;",
                                            "Confidence: {op_with_metadata.confidence:.0}%"
                                        }

                                        if let Some(rationale) = &op_with_metadata.rationale {
                                            span {
                                                style: "color: {theme.text_secondary}; font-size: 12px; font-style: italic;",
                                                "{rationale}"
                                            }
                                        }
                                    }
                                }
                            }

                            // Operation preview (if enabled)
                            if *show_previews.read() {
                                div {
                                    style: "margin-top: 12px; margin-left: 32px;",

                                    OperationPreview {
                                        operation: op_with_metadata.clone(),
                                        preview: None,
                                        theme: theme.clone(),
                                        on_approve: EventHandler::new(|_| {}),
                                        on_reject: EventHandler::new(|_| {}),
                                        is_selected: selected_operations.read()[idx],
                                    }
                                }
                            }
                        }
                    }
                }

                // Action buttons
                div {
                    style: "display: flex; justify-content: flex-end; gap: 12px; padding-top: 16px; border-top: 1px solid {theme.border};",

                    button {
                        style: "background: {theme.background_secondary}; color: {theme.text}; border: 1px solid {theme.border}; padding: 8px 16px; border-radius: 4px; cursor: pointer; font-size: 14px; transition: all 0.2s;",
                        onclick: move |_| {
                            on_reject.call(());
                        },
                        "Cancel"
                    }

                    button {
                        style: if selected_count > 0 {
                            format!("background: {}; color: white; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer; font-size: 14px; font-weight: bold; transition: all 0.2s;", theme.success)
                        } else {
                            format!("background: {}; color: {}; border: 1px solid {}; padding: 8px 16px; border-radius: 4px; cursor: not-allowed; font-size: 14px; opacity: 0.5;", theme.background_secondary, theme.text_secondary, theme.border)
                        },
                        disabled: selected_count == 0,
                        onclick: move |_| {
                            // Collect approved operations
                            let approved_ops: Vec<FileOperation> = operations
                                .iter()
                                .enumerate()
                                .filter(|(idx, _)| selected_operations.read()[*idx])
                                .map(|(_, op)| op.operation.clone())
                                .collect();

                            if !approved_ops.is_empty() {
                                on_approve.call(approved_ops);
                            }
                        },
                        "Execute {selected_count} Operations"
                    }
                }
            }
        }
    }
}

/// Get the color for an operation type
fn get_operation_color(operation: &FileOperation, theme: &ThemeColors) -> String {
    match operation {
        FileOperation::Create { .. } => theme.success.clone(),
        FileOperation::Update { .. } => theme.primary.clone(),
        FileOperation::Delete { .. } => theme.error.clone(),
        FileOperation::Rename { .. } => theme.warning.clone(),
        FileOperation::Append { .. } => theme.info.clone(),
    }
}

/// Get the display text for an operation type
fn get_operation_type_display(operation: &FileOperation) -> &'static str {
    match operation {
        FileOperation::Create { .. } => "CREATE",
        FileOperation::Update { .. } => "UPDATE",
        FileOperation::Delete { .. } => "DELETE",
        FileOperation::Rename { .. } => "RENAME",
        FileOperation::Append { .. } => "APPEND",
    }
}

/// Get the path(s) for an operation
fn get_operation_path(operation: &FileOperation) -> String {
    match operation {
        FileOperation::Create { path, .. }
        | FileOperation::Update { path, .. }
        | FileOperation::Delete { path }
        | FileOperation::Append { path, .. } => path.display().to_string(),
        FileOperation::Rename { from, to } => format!("{} → {}", from.display(), to.display()),
    }
}
