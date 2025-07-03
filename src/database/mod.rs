//! Database Optimization Module
//! 
//! Provides comprehensive database optimization for achieving <1ms latency.

pub mod optimize;

pub use optimize::{
    OptimizedDatabase, DatabaseOptimizationConfig, DatabaseMetrics,
    PoolHealthMonitor, PoolHealth
};