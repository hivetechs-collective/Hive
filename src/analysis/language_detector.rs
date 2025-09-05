//! Language detection for source files
//!
//! This module provides accurate language detection based on:
//! - File extensions
//! - File content analysis
//! - Shebang lines
//! - Magic bytes

use crate::core::Language;
use anyhow::{anyhow, Result};
use std::path::Path;

/// Standalone function for detecting language (compatibility export)
pub fn detect_language(path: &Path, content: Option<&str>) -> Result<Language> {
    let detector = LanguageDetector::new();
    detector.detect_language(path, content)
}

/// Language detector
pub struct LanguageDetector {
    /// Extension mappings
    extension_map: std::collections::HashMap<String, Language>,
    /// Filename mappings (for files like Makefile, Dockerfile)
    filename_map: std::collections::HashMap<String, Language>,
}

impl LanguageDetector {
    /// Create a new language detector
    pub fn new() -> Self {
        let mut extension_map = std::collections::HashMap::new();

        // Rust
        extension_map.insert("rs".to_string(), Language::Rust);

        // JavaScript/TypeScript
        extension_map.insert("js".to_string(), Language::JavaScript);
        extension_map.insert("jsx".to_string(), Language::JavaScript);
        extension_map.insert("mjs".to_string(), Language::JavaScript);
        extension_map.insert("cjs".to_string(), Language::JavaScript);
        extension_map.insert("ts".to_string(), Language::TypeScript);
        extension_map.insert("tsx".to_string(), Language::TypeScript);

        // Python
        extension_map.insert("py".to_string(), Language::Python);
        extension_map.insert("pyw".to_string(), Language::Python);
        extension_map.insert("pyi".to_string(), Language::Python);

        // Go
        extension_map.insert("go".to_string(), Language::Go);

        // Java
        extension_map.insert("java".to_string(), Language::Java);

        // C/C++
        extension_map.insert("c".to_string(), Language::C);
        extension_map.insert("h".to_string(), Language::C);
        extension_map.insert("cpp".to_string(), Language::Cpp);
        extension_map.insert("cxx".to_string(), Language::Cpp);
        extension_map.insert("cc".to_string(), Language::Cpp);
        extension_map.insert("hpp".to_string(), Language::Cpp);
        extension_map.insert("hxx".to_string(), Language::Cpp);
        extension_map.insert("hh".to_string(), Language::Cpp);
        extension_map.insert("c++".to_string(), Language::Cpp);
        extension_map.insert("h++".to_string(), Language::Cpp);

        // Ruby
        extension_map.insert("rb".to_string(), Language::Ruby);
        extension_map.insert("rake".to_string(), Language::Ruby);

        // PHP
        extension_map.insert("php".to_string(), Language::PHP);
        extension_map.insert("php3".to_string(), Language::PHP);
        extension_map.insert("php4".to_string(), Language::PHP);
        extension_map.insert("php5".to_string(), Language::PHP);
        extension_map.insert("php7".to_string(), Language::PHP);
        extension_map.insert("phtml".to_string(), Language::PHP);

        // Swift
        extension_map.insert("swift".to_string(), Language::Swift);

        let mut filename_map = std::collections::HashMap::new();
        filename_map.insert("Makefile".to_string(), Language::Unknown);
        filename_map.insert("makefile".to_string(), Language::Unknown);
        filename_map.insert("Dockerfile".to_string(), Language::Unknown);
        filename_map.insert("dockerfile".to_string(), Language::Unknown);
        filename_map.insert("Rakefile".to_string(), Language::Ruby);
        filename_map.insert("Gemfile".to_string(), Language::Ruby);

        Self {
            extension_map,
            filename_map,
        }
    }

    /// Detect language from file path
    pub fn detect_from_path(&self, path: &Path) -> Result<Language> {
        // Check filename first
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if let Some(lang) = self.filename_map.get(filename) {
                return Ok(*lang);
            }
        }

        // Check extension
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if let Some(lang) = self.extension_map.get(&ext.to_lowercase()) {
                return Ok(*lang);
            }
        }

        Ok(Language::Unknown)
    }

    /// Detect language from file content
    pub fn detect_from_content(&self, path: &Path, content: &str) -> Result<Language> {
        // First try path-based detection
        let path_lang = self.detect_from_path(path)?;
        if path_lang != Language::Unknown {
            return Ok(path_lang);
        }

        // Check shebang
        if let Some(lang) = self.detect_from_shebang(content) {
            return Ok(lang);
        }

        // Check content patterns
        if let Some(lang) = self.detect_from_patterns(content) {
            return Ok(lang);
        }

        Ok(Language::Unknown)
    }

    /// Detect language from shebang line
    pub fn detect_from_shebang(&self, content: &str) -> Option<Language> {
        let first_line = content.lines().next()?;
        if !first_line.starts_with("#!") {
            return None;
        }

        let shebang = first_line.to_lowercase();

        if shebang.contains("python") || shebang.contains("python3") {
            Some(Language::Python)
        } else if shebang.contains("node") || shebang.contains("deno") {
            Some(Language::JavaScript)
        } else if shebang.contains("ruby") {
            Some(Language::Ruby)
        } else if shebang.contains("php") {
            Some(Language::PHP)
        } else if shebang.contains("bash") || shebang.contains("sh") {
            Some(Language::Unknown) // Shell scripts
        } else {
            None
        }
    }

    /// Detect language from path or content (convenience method)
    pub fn detect_language(&self, path: &Path, content: Option<&str>) -> Result<Language> {
        if let Some(content) = content {
            self.detect_from_content(path, content)
        } else {
            self.detect_from_path(path)
        }
    }

    /// Detect language from content patterns
    pub fn detect_from_patterns(&self, content: &str) -> Option<Language> {
        // Rust patterns
        if content.contains("fn main()")
            || content.contains("impl ")
            || content.contains("use std::")
        {
            return Some(Language::Rust);
        }

        // Python patterns
        if content.contains("def ")
            || content.contains("import ")
            || content.contains("from ") && content.contains(" import ")
        {
            return Some(Language::Python);
        }

        // JavaScript/TypeScript patterns
        if content.contains("function ")
            || content.contains("const ")
            || content.contains("let ")
            || content.contains("var ")
        {
            if content.contains("interface ")
                || content.contains(": string")
                || content.contains(": number")
            {
                return Some(Language::TypeScript);
            }
            return Some(Language::JavaScript);
        }

        // Go patterns
        if content.contains("package ") && content.contains("func ") {
            return Some(Language::Go);
        }

        // Java patterns
        if content.contains("public class ")
            || content.contains("private class ")
            || content.contains("public static void main")
        {
            return Some(Language::Java);
        }

        // C++ patterns
        if content.contains("#include <")
            || content.contains("std::")
            || content.contains("namespace ")
        {
            return Some(Language::Cpp);
        }

        // C patterns
        if content.contains("#include <") && !content.contains("std::") {
            return Some(Language::C);
        }

        // Ruby patterns
        if content.contains("def ") && content.contains("end")
            || content.contains("class ") && content.contains("end")
        {
            return Some(Language::Ruby);
        }

        // PHP patterns
        if content.contains("<?php") || content.contains("<?=") {
            return Some(Language::PHP);
        }

        // Swift patterns
        if content.contains("import ")
            && (content.contains("UIKit") || content.contains("Foundation"))
        {
            return Some(Language::Swift);
        }

        None
    }

    /// Get confidence score for language detection
    pub fn get_confidence(&self, path: &Path, content: &str, detected: Language) -> f32 {
        let mut score: f32 = 0.0;

        // Path-based detection is highly confident
        if let Ok(path_lang) = self.detect_from_path(path) {
            if path_lang == detected && path_lang != Language::Unknown {
                score += 0.8;
            }
        }

        // Shebang is very reliable
        if let Some(shebang_lang) = self.detect_from_shebang(content) {
            if shebang_lang == detected {
                score += 0.9;
            }
        }

        // Pattern matching adds confidence
        if let Some(pattern_lang) = self.detect_from_patterns(content) {
            if pattern_lang == detected {
                score += 0.6;
            }
        }

        score.min(1.0f32)
    }
}

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

// Add Ruby, PHP, and Swift to the Language enum in core/mod.rs
impl Language {
    /// Get all supported languages
    pub fn all() -> Vec<Language> {
        vec![
            Language::Rust,
            Language::TypeScript,
            Language::JavaScript,
            Language::Python,
            Language::Go,
            Language::Java,
            Language::Cpp,
            Language::C,
            Language::Ruby,
            Language::PHP,
            Language::Swift,
        ]
    }

    /// Get file extensions for a language
    pub fn extensions(&self) -> Vec<&'static str> {
        match self {
            Language::Rust => vec!["rs"],
            Language::TypeScript => vec!["ts", "tsx"],
            Language::JavaScript => vec!["js", "jsx", "mjs", "cjs"],
            Language::Python => vec!["py", "pyw", "pyi"],
            Language::Go => vec!["go"],
            Language::Java => vec!["java"],
            Language::Cpp => vec!["cpp", "cxx", "cc", "hpp", "hxx", "hh", "c++", "h++"],
            Language::C => vec!["c", "h"],
            Language::Ruby => vec!["rb", "rake"],
            Language::PHP => vec!["php", "php3", "php4", "php5", "php7", "phtml"],
            Language::Swift => vec!["swift"],
            Language::Unknown => vec![],
        }
    }

    /// Get display name for a language
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::TypeScript => "TypeScript",
            Language::JavaScript => "JavaScript",
            Language::Python => "Python",
            Language::Go => "Go",
            Language::Java => "Java",
            Language::Cpp => "C++",
            Language::C => "C",
            Language::Ruby => "Ruby",
            Language::PHP => "PHP",
            Language::Swift => "Swift",
            Language::Unknown => "Unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_extension_detection() {
        let detector = LanguageDetector::new();

        assert_eq!(
            detector.detect_from_path(Path::new("test.rs")).unwrap(),
            Language::Rust
        );
        assert_eq!(
            detector.detect_from_path(Path::new("test.py")).unwrap(),
            Language::Python
        );
        assert_eq!(
            detector.detect_from_path(Path::new("test.ts")).unwrap(),
            Language::TypeScript
        );
        assert_eq!(
            detector.detect_from_path(Path::new("test.js")).unwrap(),
            Language::JavaScript
        );
        assert_eq!(
            detector.detect_from_path(Path::new("test.go")).unwrap(),
            Language::Go
        );
        assert_eq!(
            detector.detect_from_path(Path::new("test.java")).unwrap(),
            Language::Java
        );
        assert_eq!(
            detector.detect_from_path(Path::new("test.cpp")).unwrap(),
            Language::Cpp
        );
        assert_eq!(
            detector.detect_from_path(Path::new("test.c")).unwrap(),
            Language::C
        );
        assert_eq!(
            detector.detect_from_path(Path::new("test.rb")).unwrap(),
            Language::Ruby
        );
        assert_eq!(
            detector.detect_from_path(Path::new("test.php")).unwrap(),
            Language::PHP
        );
        assert_eq!(
            detector.detect_from_path(Path::new("test.swift")).unwrap(),
            Language::Swift
        );
    }

    #[test]
    fn test_shebang_detection() {
        let detector = LanguageDetector::new();

        let python_content = "#!/usr/bin/env python3\nprint('Hello')";
        assert_eq!(
            detector.detect_from_shebang(python_content),
            Some(Language::Python)
        );

        let node_content = "#!/usr/bin/env node\nconsole.log('Hello')";
        assert_eq!(
            detector.detect_from_shebang(node_content),
            Some(Language::JavaScript)
        );

        let ruby_content = "#!/usr/bin/env ruby\nputs 'Hello'";
        assert_eq!(
            detector.detect_from_shebang(ruby_content),
            Some(Language::Ruby)
        );
    }

    #[test]
    fn test_pattern_detection() {
        let detector = LanguageDetector::new();

        let rust_content = "fn main() {\n    println!(\"Hello\");\n}";
        assert_eq!(
            detector.detect_from_patterns(rust_content),
            Some(Language::Rust)
        );

        let python_content = "def main():\n    print('Hello')";
        assert_eq!(
            detector.detect_from_patterns(python_content),
            Some(Language::Python)
        );

        let go_content = "package main\n\nfunc main() {\n    fmt.Println(\"Hello\")\n}";
        assert_eq!(
            detector.detect_from_patterns(go_content),
            Some(Language::Go)
        );
    }

    #[test]
    fn test_confidence_scoring() {
        let detector = LanguageDetector::new();

        let path = Path::new("test.rs");
        let content = "fn main() {}";
        let confidence = detector.get_confidence(path, content, Language::Rust);
        assert!(confidence > 0.8);
    }
}
