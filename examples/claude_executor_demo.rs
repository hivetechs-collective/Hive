//! Demonstration of the ClaudeCodeExecutor capabilities
//! 
//! This example shows how the hybrid Claude-Consensus integration works
//! without requiring actual API keys or running processes.

use hive_ai::consensus::{ClaudeCodeExecutor, ClaudeExecutionMode, ConsensusInvocationCriteria};

fn main() {
    println!("\n=== Claude Code Executor Demonstration ===\n");
    
    // 1. Show the three execution modes
    println!("📋 Execution Modes:");
    println!("  • Direct: Claude executes autonomously");
    println!("  • ConsensusAssisted: Claude validates major decisions through consensus");
    println!("  • ConsensusRequired: All plans go through consensus validation\n");
    
    // 2. Demonstrate consensus invocation criteria
    println!("🔍 Consensus Invocation Criteria:");
    
    let test_scenarios = vec![
        (
            "Create a simple README file",
            ConsensusInvocationCriteria {
                architectural_change: false,
                high_risk_operation: false,
                confidence_level: 0.9,
                multiple_approaches: false,
                user_requested_analysis: false,
                complexity_score: 0.2,
            },
            false,
            "Low risk, high confidence - executes directly"
        ),
        (
            "Delete all user data from production database",
            ConsensusInvocationCriteria {
                architectural_change: false,
                high_risk_operation: true,
                confidence_level: 0.9,
                multiple_approaches: false,
                user_requested_analysis: false,
                complexity_score: 0.3,
            },
            true,
            "High risk operation - requires consensus validation"
        ),
        (
            "Refactor the entire authentication system",
            ConsensusInvocationCriteria {
                architectural_change: true,
                high_risk_operation: false,
                confidence_level: 0.7,
                multiple_approaches: true,
                user_requested_analysis: false,
                complexity_score: 0.9,
            },
            true,
            "Architectural change with high complexity - requires consensus"
        ),
        (
            "Fix a typo in a comment",
            ConsensusInvocationCriteria {
                architectural_change: false,
                high_risk_operation: false,
                confidence_level: 0.95,
                multiple_approaches: false,
                user_requested_analysis: false,
                complexity_score: 0.1,
            },
            false,
            "Trivial change - executes directly"
        ),
        (
            "Analyze and explain the codebase architecture",
            ConsensusInvocationCriteria {
                architectural_change: false,
                high_risk_operation: false,
                confidence_level: 0.8,
                multiple_approaches: false,
                user_requested_analysis: true,
                complexity_score: 0.6,
            },
            true,
            "User requested analysis - uses consensus for diverse perspectives"
        ),
    ];
    
    for (request, criteria, should_invoke, reason) in test_scenarios {
        println!("\n  Request: \"{}\"", request);
        
        // Simulate the decision logic
        let invokes_consensus = criteria.high_risk_operation || 
                               criteria.architectural_change ||
                               criteria.user_requested_analysis ||
                               criteria.confidence_level < 0.6 ||
                               criteria.complexity_score > 0.8 ||
                               criteria.multiple_approaches;
        
        println!("  Decision: {} consensus", 
                 if invokes_consensus { "INVOKE" } else { "SKIP" });
        println!("  Reason: {}", reason);
        
        // Show the criteria breakdown
        if invokes_consensus {
            println!("  Factors:");
            if criteria.high_risk_operation { println!("    • High risk operation") }
            if criteria.architectural_change { println!("    • Architectural change") }
            if criteria.user_requested_analysis { println!("    • User requested analysis") }
            if criteria.confidence_level < 0.6 { println!("    • Low confidence ({:.0}%)", criteria.confidence_level * 100.0) }
            if criteria.complexity_score > 0.8 { println!("    • High complexity ({:.0}%)", criteria.complexity_score * 100.0) }
            if criteria.multiple_approaches { println!("    • Multiple valid approaches") }
        }
        
        assert_eq!(invokes_consensus, should_invoke, "Decision logic mismatch!");
    }
    
    // 3. Show the knowledge accumulation concept
    println!("\n\n💾 Knowledge Accumulation:");
    println!("  When consensus is invoked:");
    println!("  1. Claude generates a plan");
    println!("  2. Consensus pipeline validates through 4 stages");
    println!("  3. Curator provides authoritative guidance");
    println!("  4. Claude executes with curator-validated approach");
    println!("  5. Result stored in knowledge_conversations table");
    println!("  6. Future similar requests benefit from this knowledge");
    
    // 4. Show the execution flow
    println!("\n\n🔄 Execution Flow Example:");
    println!("  User: \"Refactor the authentication system to use JWT tokens\"");
    println!("  ");
    println!("  Claude (ConsensusAssisted mode):");
    println!("  1. Analyzes: \"This is a high-impact architectural change\"");
    println!("  2. Confidence: 70% (multiple approaches possible)");
    println!("  3. Decision: INVOKE CONSENSUS");
    println!("  4. Generates comprehensive refactoring plan");
    println!("  5. Sends plan to consensus pipeline");
    println!("  6. Consensus evaluates through 4 stages:");
    println!("     • Generator: Creates initial approach");
    println!("     • Refiner: Improves and adds details");
    println!("     • Validator: Checks for issues");
    println!("     • Curator: Provides authoritative guidance");
    println!("  7. Claude executes with curator-validated approach");
    println!("  8. Stores curator output as permanent knowledge");
    
    println!("\n\n✅ Benefits of This Hybrid Approach:");
    println!("  • Best of both worlds: Claude's speed + consensus wisdom");
    println!("  • Intelligent decision making about when to validate");
    println!("  • Growing knowledge base from every consensus run");
    println!("  • User control via execution modes");
    println!("  • Transparency about when/why consensus is used");
    
    println!("\n=== Demo Complete ===\n");
}