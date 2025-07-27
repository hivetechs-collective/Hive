//! Document utilities for Dioxus desktop applications
//! 
//! Provides access to browser-like document APIs in desktop context

/// Re-export eval function for JavaScript execution
pub use dioxus::document::eval;

/// Helper to get window dimensions
pub async fn get_window_dimensions() -> Result<(f64, f64), String> {
    let result = eval(r#"
        JSON.stringify({
            width: window.innerWidth,
            height: window.innerHeight
        })
    "#).await?;
    
    #[derive(serde::Deserialize)]
    struct Dimensions {
        width: f64,
        height: f64,
    }
    
    let dims: Dimensions = serde_json::from_str(&result)
        .map_err(|e| e.to_string())?;
    
    Ok((dims.width, dims.height))
}

/// Helper to get element bounds
pub async fn get_element_bounds(selector: &str) -> Result<ElementBounds, String> {
    let script = format!(r#"
        const el = document.querySelector('{}');
        if (el) {{
            const rect = el.getBoundingClientRect();
            JSON.stringify({{
                left: rect.left,
                top: rect.top,
                width: rect.width,
                height: rect.height
            }})
        }} else {{
            null
        }}
    "#, selector);
    
    let result = eval(&script).await?;
    
    if result == "null" {
        return Err("Element not found".to_string());
    }
    
    serde_json::from_str(&result)
        .map_err(|e| e.to_string())
}

#[derive(serde::Deserialize)]
pub struct ElementBounds {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}