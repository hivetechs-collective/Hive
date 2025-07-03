// Test Rust project for repository intelligence analysis

use std::collections::HashMap;

/// Main function - high complexity example
fn main() {
    let password = "hardcoded_secret"; // Security vulnerability
    
    // Nested loops - performance issue
    for i in 0..100 {
        for j in 0..100 {
            for k in 0..100 {
                println!("Computing {}-{}-{}", i, j, k);
            }
        }
    }
    
    // Call other functions
    UserController::new().create_user("test");
    UserModel::load_all();
}

/// Controller component - MVC pattern
pub struct UserController {
    users: HashMap<String, User>,
}

impl UserController {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }
    
    // Method with high complexity
    pub fn create_user(&mut self, name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        
        if name.len() < 3 {
            return Err("Name too short".to_string());
        }
        
        if name.len() > 50 {
            return Err("Name too long".to_string());
        }
        
        if name.contains("@") {
            return Err("Invalid character".to_string());
        }
        
        if name.starts_with("admin") {
            return Err("Reserved name".to_string());
        }
        
        // More conditions...
        if name.to_lowercase() == "root" {
            return Err("Reserved name".to_string());
        }
        
        let user = User {
            id: generate_id(),
            name: name.to_string(),
            email: format!("{}@example.com", name),
        };
        
        self.users.insert(name.to_string(), user);
        Ok(())
    }
}

/// Model component - MVC pattern
pub struct UserModel;

impl UserModel {
    // N+1 query pattern - performance issue
    pub fn load_all() -> Vec<User> {
        let mut users = Vec::new();
        
        for i in 1..=100 {
            // Simulating database query in loop
            let user = Self::find_by_id(i);
            users.push(user);
        }
        
        users
    }
    
    fn find_by_id(id: u32) -> User {
        // Simulate database access
        User {
            id,
            name: format!("User{}", id),
            email: format!("user{}@example.com", id),
        }
    }
}

/// Domain entity
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}

// Missing documentation - quality issue
fn generate_id() -> u32 {
    42 // Placeholder implementation
}

// Duplicate functionality - code smell
fn create_user_id() -> u32 {
    42 // Same as generate_id - duplication
}

// Unused function - dead code
#[allow(dead_code)]
fn unused_function() {
    println!("This function is never called");
}

// Recursive function without proper termination
fn fibonacci(n: u32) -> u32 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2) // Inefficient recursion
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let mut controller = UserController::new();
        assert!(controller.create_user("testuser").is_ok());
    }
    
    // Missing test coverage for edge cases
}