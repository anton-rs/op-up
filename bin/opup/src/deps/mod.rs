/// Manager for dependencies
pub mod manager;
pub use manager::DependencyManager;

/// Docker and Docker Compose Dependency
pub mod docker {}

/// Make Dependency
pub mod make {}

/// JQ Dependency
pub mod jq {}
