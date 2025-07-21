# Pre-Task Checklist: Mandatory Deep Understanding Protocol

## 🎯 Purpose

This document establishes a **MANDATORY** checklist that must be followed before starting ANY development task. This ensures total understanding of the codebase, reduces bugs, minimizes compilation errors, and maintains consistency with existing patterns.

## ⚠️ CRITICAL RULE

**NEVER** start implementing without completing this checklist. Taking 10-15 minutes to understand the codebase saves hours of debugging and rework.

## ✅ The Mandatory Pre-Task Checklist

### 1. **Understand the Task Context**
- [ ] Read the task description completely
- [ ] Identify all components that will be affected
- [ ] List all files that will need modification
- [ ] Determine if this is a new feature or modification

### 2. **Research Existing Code Structure**
```
⏺ Search for related implementations:
- [ ] Search for similar patterns in the codebase
- [ ] Check for existing utilities that can be reused
- [ ] Identify the module structure and organization
- [ ] Note the import patterns used
```

### 3. **Understand the Type System**
```
⏺ For each type/struct/trait you'll interact with:
- [ ] Read the complete type definition
- [ ] Check all fields and their types
- [ ] Understand available methods
- [ ] Note any trait implementations
- [ ] Check for builder patterns or constructors
```

### 4. **Verify API Contracts**
```
⏺ For each external API or module:
- [ ] Read the function signatures completely
- [ ] Understand parameter types and counts
- [ ] Check return types and error handling
- [ ] Look for usage examples in tests or other code
```

### 5. **Check the Tech Stack Version**
```
⏺ Verify framework/library versions:
- [ ] Check Cargo.toml for exact versions
- [ ] Read relevant documentation for that version
- [ ] Note any version-specific patterns or limitations
- [ ] Understand async/sync requirements
```

### 6. **Understand Error Handling Patterns**
```
⏺ Check how errors are handled:
- [ ] Custom error types used
- [ ] Result<T, E> patterns
- [ ] Error propagation methods
- [ ] Logging and error reporting
```

### 7. **Review Code Style and Conventions**
```
⏺ Understand project conventions:
- [ ] Naming conventions (snake_case, CamelCase)
- [ ] Module organization patterns
- [ ] Comment style and documentation
- [ ] Import grouping and ordering
```

### 8. **UI Framework Syntax Rules (CRITICAL FOR RSX/DIOXUS)**
```
⏺ For any UI/frontend code (Dioxus RSX, React JSX, etc.):
- [ ] Identify the UI framework version (Dioxus 0.6+, React, etc.)
- [ ] Find working component examples in the SAME file
- [ ] Understand conditional rendering patterns used
- [ ] Note how Rust code is separated from UI markup
- [ ] Check component props/parameter passing syntax
- [ ] Verify event handler patterns
- [ ] Understand state management approach
```

#### RSX/Dioxus Specific Rules (MANDATORY):
```
⚠️ CRITICAL DIOXUS PATTERNS:

1. **Conditional Rendering Structure**:
   ❌ WRONG: Complex logic mixed with RSX
   if condition {
       let data = complex_computation();
       div { "content: {data}" }
   }
   
   ✅ RIGHT: Logic separated from RSX
   let data = if condition {
       complex_computation()
   } else {
       default_value()
   };
   
   if condition {
       div { "content: {data}" }
   }

2. **Component Parameter Syntax**:
   ❌ WRONG: Assuming struct syntax without verification
   MyComponent {
       field1: value1,
       field2: value2,
   }
   
   ✅ RIGHT: Check existing working examples first
   - Search for other component calls in same file
   - Verify #[component] macro expectations
   - Check if Props struct is auto-generated correctly

3. **RSX Context Rules**:
   ❌ WRONG: Complex Rust statements inside RSX
   div {
       let result = function_call();
       "Value: {result}"
   }
   
   ✅ RIGHT: Calculations outside RSX context
   let result = function_call();
   div {
       "Value: {result}"
   }
```

## 📋 Implementation Workflow

### Phase 1: Deep Research (MANDATORY)
```rust
// BEFORE writing any code:

⏺ Search(pattern: "struct TargetType", path: "src/", output_mode: "content")
  ⎿ Understanding the complete type definition

⏺ Search(pattern: "impl TargetType", path: "src/", output_mode: "content")
  ⎿ Finding all available methods

⏺ Search(pattern: "TargetType::new|TargetType::builder", path: "src/", output_mode: "content")
  ⎿ Understanding construction patterns

⏺ Read(src/module/target.rs)
  ⎿ Reading complete implementation
```

### Phase 2: Verification
```rust
⏺ Check related test files:
- tests/integration/target_test.rs
- src/module/target.rs (look for #[cfg(test)])

⏺ Find usage examples:
- Search for where this type is used
- How other code interacts with it
- Common patterns and idioms
```

### Phase 3: Plan Implementation
```rust
⏺ Create implementation plan:
1. List all required imports
2. Define function signatures
3. Plan error handling
4. Consider edge cases
5. Design test cases
```

### Phase 4: UI Framework Specific Research (IF APPLICABLE)
```rust
⏺ For UI/frontend components (Dioxus, React, etc.):

1. Framework Version Check:
   ⏺ Grep(pattern: "dioxus.*=", path: "Cargo.toml")
   ⏺ Check exact version and documentation

2. Working Component Research:
   ⏺ Grep(pattern: "#\\[component\\]", output_mode: "content", -A: 5)
     ⎿ Find all component definitions
   
   ⏺ Grep(pattern: "ComponentName \\{", output_mode: "content", -A: 3)
     ⎿ Find actual component usage examples

3. RSX Pattern Analysis:
   ⏺ Search for conditional rendering patterns:
     - if statements in RSX
     - How complex logic is handled
     - Variable declaration placement
     
4. State Management Check:
   ⏺ Look for use_signal, use_state patterns
   ⏺ Understand how data flows between components

5. Event Handler Patterns:
   ⏺ Find onclick, on_submit, EventHandler usage
   ⏺ Note callback/closure patterns
```

## 🚨 Common Pitfalls to Avoid

### 1. **Assuming Method Names**
❌ **Wrong**: Assuming `get_model_for_stage()` exists
✅ **Right**: Search for actual methods: `select_optimal_model()`

### 2. **Guessing Field Names**
❌ **Wrong**: Using `token_count`, `cost`, `duration`
✅ **Right**: Check struct definition: `usage`, `analytics`

### 3. **Incorrect API Usage**
❌ **Wrong**: Expecting `create_completion_stream()` to return a receiver
✅ **Right**: Understanding it uses callbacks

### 4. **Type Mismatches**
❌ **Wrong**: Passing wrong number of arguments
✅ **Right**: Count parameters in function signature

### 5. **RSX/UI Framework Context Violations** ⚠️ **CRITICAL**
❌ **Wrong**: Writing Rust logic inside RSX conditional blocks
```rust
if !data.is_empty() {
    let processed = process_data(&data);
    let status = calculate_status(processed);
    div { "Status: {status}" }
}
```
✅ **Right**: Separate logic from UI rendering
```rust
let processed = if !data.is_empty() {
    process_data(&data)
} else {
    default_data()
};
let status = calculate_status(processed);

if !data.is_empty() {
    div { "Status: {status}" }
}
```

### 6. **Component Usage Without Research**
❌ **Wrong**: Assuming component syntax without checking existing usage
```rust
MyComponent {
    prop1: value1,
    prop2: value2,
}
```
✅ **Right**: Search for working component examples first
```bash
⏺ Grep(pattern: "MyComponent \\{", output_mode: "content", -A: 3)
  ⎿ Find how component is actually used in codebase
```

## 📊 Time Investment Guide

| Task Complexity | Research Time | Implementation Time | Total Time Saved |
|----------------|---------------|-------------------|------------------|
| Simple Fix     | 5-10 mins     | 10-20 mins       | 30-60 mins      |
| New Feature    | 15-30 mins    | 1-2 hours        | 2-4 hours       |
| Refactoring    | 30-60 mins    | 2-4 hours        | 4-8 hours       |

## 🔄 Continuous Learning

### After Each Task:
1. **Document Learnings**: Add new patterns discovered
2. **Update Examples**: Add code examples for future reference
3. **Share Knowledge**: Update team documentation

### Regular Reviews:
- Weekly: Review common errors and how to avoid them
- Monthly: Update checklist based on team experiences

## 💡 Example 1: DirectExecutor Implementation

### ❌ What NOT to do:
```rust
// Starting implementation without research:
let model = self.model_manager.get_model_for_stage("generator")?; // Doesn't exist!
let response_stream = self.client.create_completion_stream(...).await?; // Wrong API!
```

### ✅ What TO do:
```rust
⏺ Search(pattern: "pub async fn.*model", path: "src/consensus/models.rs")
  ⎿ Found: select_optimal_model() requires ModelSelectionCriteria

⏺ Search(pattern: "create.*stream", path: "src/consensus/openrouter.rs")  
  ⎿ Found: chat_completion_stream() uses callbacks, not receivers

// Now implement with correct APIs:
let criteria = ModelSelectionCriteria { ... };
let model = self.model_selector.select_optimal_model(&self.db, &criteria, None).await?;
```

## 💡 Example 2: RSX/Dioxus Component Implementation ⚠️ **CRITICAL**

### ❌ What NOT to do:
```rust
// Writing complex logic inside RSX conditional - CAUSES COMPILATION ERRORS!
if !app_state.read().consensus.streaming_content.is_empty() {
    // Parse operations from streaming content
    let operations = parse_operations_from_content(&app_state.read().consensus.streaming_content);
    let operation_statuses: Vec<(FileOperation, OperationStatus)> = operations.into_iter()
        .map(|op| (op, OperationStatus::Completed))
        .collect();
    
    // Get theme
    let theme = ThemeColors::dark_theme();
    
    ResponseSection {
        content: app_state.read().consensus.streaming_content.clone(),
        operations: operation_statuses,
        theme,
    }
}
```

### ✅ What TO do:
```rust
⏺ First - Research existing patterns:
  Grep(pattern: "if.*\\{.*div \\{", path: "src/bin/", output_mode: "content")
  ⎿ Find how other conditionals are structured in same file

⏺ Check working component examples:  
  Grep(pattern: "ResponseSection \\{", output_mode: "content", -A: 3)
  ⎿ See if component exists and how it's used

// Now implement with proper RSX structure:
// 1. Logic OUTSIDE RSX context
let operations = if !app_state.read().consensus.streaming_content.is_empty() {
    let parsed = parse_operations_from_content(&app_state.read().consensus.streaming_content);
    parsed.into_iter().map(|op| (op, OperationStatus::Completed)).collect()
} else {
    Vec::new()
};

let theme = ThemeColors::dark_theme();

// 2. RSX rendering separate and clean
if !app_state.read().consensus.streaming_content.is_empty() {
    div {
        class: "response-content",
        dangerous_inner_html: "{app_state.read().consensus.streaming_content}"
    }
}
```

## 🎯 Success Metrics

Track these metrics to measure effectiveness:
- **Compilation Errors**: Should approach zero
- **API Misuse**: Should be eliminated
- **Rework Time**: Should decrease by 70%+
- **Code Quality**: Should improve consistently

## 📝 Checklist Template

Copy this template for each new task:

```markdown
## Task: [Task Name]
Date: [Date]

### Pre-Implementation Checklist
- [ ] Task context understood
- [ ] Affected components identified: ___________
- [ ] Existing patterns researched
- [ ] Type definitions verified
- [ ] API contracts checked
- [ ] Tech stack versions confirmed
- [ ] Error patterns understood
- [ ] Code conventions reviewed
- [ ] **UI Framework patterns analyzed** (if applicable)
- [ ] **RSX/component syntax verified** (if applicable)
- [ ] **Existing component usage researched** (if applicable)

### Research Notes:
- Key types involved: ___________
- APIs to use: ___________
- Patterns to follow: ___________
- Potential pitfalls: ___________
- **UI Framework version**: ___________ (if applicable)
- **Component usage patterns found**: ___________ (if applicable)
- **RSX conditional rendering approach**: ___________ (if applicable)

### Implementation Plan:
1. ___________
2. ___________
3. ___________

### Post-Implementation:
- [ ] All tests passing
- [ ] No compilation warnings
- [ ] Follows established patterns
- [ ] Documentation updated
```

## 🏁 Conclusion

This checklist is not optional—it's a critical part of our development process. By investing time upfront to understand the codebase, we:

1. **Reduce Bugs**: Catch issues before they're written
2. **Save Time**: Avoid multiple rounds of fixes
3. **Improve Quality**: Write code that fits seamlessly
4. **Learn Continuously**: Build deeper understanding

Remember: **Understanding before implementing is not slow—it's efficient!**