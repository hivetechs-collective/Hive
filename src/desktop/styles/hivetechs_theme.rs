//! HiveTechs Brand Theme

/// Get the HiveTechs theme CSS
pub fn get_hivetechs_theme() -> &'static str {
    r##"
    /* ===== HiveTechs Brand Theme ===== */
    
    /* Brand Colors */
    :root {
        --hive-yellow: #FFC107;
        --hive-yellow-light: #FFD54F;
        --hive-yellow-dark: #FFAD00;
        --hive-blue: #007BFF;
        --hive-green: #28A745;
        --hive-dark-bg: #0E1414;
        --hive-dark-bg-secondary: #181E21;
    }
    
    /* Menu Bar Branding */
    .app-container > div:first-child {
        background-color: var(--hive-dark-bg) !important;
        border-bottom-color: var(--hive-yellow-dark) !important;
    }
    
    /* Brand Title Text */
    .app-container > div:first-child span:last-child {
        background: linear-gradient(135deg, var(--hive-yellow) 0%, var(--hive-yellow-light) 100%);
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
        font-weight: 600;
    }
    
    /* Status Bar Branding */
    .status-bar {
        background-color: var(--hive-dark-bg) !important;
        border-top: 1px solid var(--hive-yellow-dark) !important;
    }
    
    /* Connected Status Indicator */
    .status-indicator.connected {
        background-color: var(--hive-green) !important;
    }
    
    /* Button Hover Effects */
    .btn-primary {
        background-color: var(--hive-yellow) !important;
        color: #000 !important;
        font-weight: 500;
    }
    
    .btn-primary:hover {
        background-color: var(--hive-yellow-light) !important;
    }
    
    /* Dialog Headers */
    .dialog-header {
        background: linear-gradient(135deg, var(--hive-dark-bg) 0%, var(--hive-dark-bg-secondary) 100%);
        border-bottom: 2px solid var(--hive-yellow);
    }
    
    /* Active Tab Indicator */
    .tab-active {
        border-bottom: 2px solid var(--hive-yellow) !important;
    }
    
    /* Consensus Progress Branding */
    .consensus-stage.active {
        border-color: var(--hive-yellow) !important;
        background-color: rgba(255, 193, 7, 0.1) !important;
    }
    
    .consensus-stage.completed {
        border-color: var(--hive-green) !important;
        background-color: rgba(40, 167, 69, 0.1) !important;
    }
    "##
}