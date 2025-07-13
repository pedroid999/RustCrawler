# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Universal Development Guidelines

### Code Quality Standards
- Write clean, readable, and maintainable code
- Follow consistent naming conventions across the project
- Use meaningful variable and function names
- Keep functions focused and single-purpose
- Add comments for complex logic and business rules

### Git Workflow
- Use descriptive commit messages following conventional commits format
- Create feature branches for new development
- Keep commits atomic and focused on single changes
- Use pull requests for code review before merging
- Maintain a clean commit history

### Documentation
- Keep README.md files up to date
- Document public APIs and interfaces
- Include usage examples for complex features
- Maintain inline code documentation
- Update documentation when making changes

### Testing Approach
- Write tests for new features and bug fixes
- Maintain good test coverage
- Use descriptive test names that explain the expected behavior
- Organize tests logically by feature or module
- Run tests before committing changes

### Security Best Practices
- Never commit sensitive information (API keys, passwords, tokens)
- Use environment variables for configuration
- Validate input data and sanitize outputs
- Follow principle of least privilege
- Keep dependencies updated

## Project Structure Guidelines

### File Organization
- Group related files in logical directories
- Use consistent file and folder naming conventions
- Separate source code from configuration files
- Keep build artifacts out of version control
- Organize assets and resources appropriately

### Configuration Management
- Use configuration files for environment-specific settings
- Centralize configuration in dedicated files
- Use environment variables for sensitive or environment-specific data
- Document configuration options and their purposes
- Provide example configuration files

## Development Workflow

### Before Starting Work
1. Pull latest changes from main branch
2. Create a new feature branch
3. Review existing code and architecture
4. Plan the implementation approach

### During Development
1. Make incremental commits with clear messages
2. Run tests frequently to catch issues early
3. Follow established coding standards
4. Update documentation as needed

### Before Submitting
1. Run full test suite
2. Check code quality and formatting
3. Update documentation if necessary
4. Create clear pull request description

## Common Patterns

### Error Handling
- Use appropriate error handling mechanisms for the language
- Provide meaningful error messages
- Log errors appropriately for debugging
- Handle edge cases gracefully
- Don't expose sensitive information in error messages

### Performance Considerations
- Profile code for performance bottlenecks
- Optimize database queries and API calls
- Use caching where appropriate
- Consider memory usage and resource management
- Monitor and measure performance metrics

### Code Reusability
- Extract common functionality into reusable modules
- Use dependency injection for better testability
- Create utility functions for repeated operations
- Design interfaces for extensibility
- Follow DRY (Don't Repeat Yourself) principle

## Rust-Specific Guidelines

### Build and Test Commands
- `cargo build` - Build the project
- `cargo test` - Run all tests
- `cargo clippy` - Run linter (required before commits)
- `cargo fmt` - Format code (required before commits)
- `cargo check` - Type check without building (fast feedback)
- `cargo run` - Build and run the binary

### Development Workflow
- Always run `cargo fmt` and `cargo clippy` before committing
- Use `cargo check` frequently during development for fast type checking
- Run the full test suite with `cargo test` before pushing
- Use `cargo build --release` for production builds

### Dependency Management
- Use `cargo add <crate>` for adding new dependencies
- Keep `Cargo.lock` in version control for reproducible builds
- Update dependencies with `cargo update`
- Check for outdated dependencies with `cargo outdated` (if installed)

### Error Handling Patterns
- Use `anyhow::Result<T>` for application errors
- Use `thiserror::Error` for custom error types
- Prefer `?` operator for error propagation
- Handle errors gracefully in async contexts

### Async/Await Best Practices
- Use `tokio::spawn` for concurrent tasks
- Prefer `async fn` over `impl Future`
- Use `Arc<Mutex<T>>` or `Arc<RwLock<T>>` for shared state
- Use `DashMap` for concurrent HashMap operations (already in dependencies)

### Code Quality Standards for Rust
- Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types)
- Use `#[derive(Debug)]` on all structs and enums
- Prefer immutable variables when possible
- Use `const` for compile-time constants
- Add `#[allow(dead_code)]` only when necessary during development

### Testing Approach for Rust
- Write unit tests in the same file with `#[cfg(test)]` module
- Use `tokio-test` for async test utilities
- Create integration tests in `tests/` directory
- Use `cargo test --doc` to test documentation examples
- Mock external dependencies in tests

## Review Checklist

Before marking any task as complete:
- [ ] Code follows established conventions
- [ ] Tests are written and passing
- [ ] `cargo fmt` has been run
- [ ] `cargo clippy` passes without warnings
- [ ] Documentation is updated
- [ ] Security considerations are addressed
- [ ] Performance impact is considered
- [ ] Code is reviewed for maintainability