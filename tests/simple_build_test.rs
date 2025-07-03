//! Simple build test to verify security system compiles

#[test]
fn test_security_modules_compile() {
    // Just check that our modules compile correctly
    use hive_ai::{
        TrustManager, TrustLevel, TrustDecision, SecurityPolicy,
        SecureFileAccess, SecurityConfig
    };
    
    // Test enum values
    let _trust_level = TrustLevel::Trusted;
    let _decision = TrustDecision::TrustGranted;
    
    // Test default configurations
    let _policy = SecurityPolicy::default();
    let _config = SecurityConfig::default();
    
    // This test passes if everything compiles
    assert!(true);
}