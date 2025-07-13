//! Embedded assets for the desktop application

use base64::{Engine as _, engine::general_purpose::STANDARD};

/// Embedded HiveTechs logo (small version)
pub const HIVE_LOGO_SMALL: &[u8] = include_bytes!("../../assets/Hive-Logo-small.jpg");

/// Get the logo as a data URL for use in HTML/CSS
pub fn get_logo_data_url() -> String {
    let base64 = STANDARD.encode(HIVE_LOGO_SMALL);
    format!("data:image/jpeg;base64,{}", base64)
}

/// Get the logo as an HTML image element
pub fn get_logo_html() -> String {
    format!(
        r#"<img src="{}" alt="HiveTechs Logo" style="width: 100%; height: 100%; object-fit: contain;" />"#,
        get_logo_data_url()
    )
}