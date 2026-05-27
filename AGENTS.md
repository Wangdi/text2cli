# Text2CLI Project

AI-powered command suggestion tool integrated with shell (bash/zsh/PowerShell).

## Architecture

- **Trigger Parser**: Detects `@@@` pattern in shell
- **Context Collector**: Gathers git status, recent files, environment
- **Agent Adapter**: Interfaces with Claude Code, Codex, etc.
- **Executor**: Runs agents with timeout/retry
- **Shell Hook**: Injects commands into terminal buffer

## Review Guidelines

### Priority Levels

**P0 - Critical** (Must fix before merge)
- Security vulnerabilities (command injection, unsafe deserialization)
- Breaking changes to public API
- Data loss risks
- Authentication/authorization bypasses

**P1 - High** (Should fix before merge)
- Error handling missing `.expect()` or proper error types
- Resource leaks (file handles, connections)
- Race conditions in async code
- Missing input validation
- Unwrapped `unwrap()` in production code paths

**P2 - Medium** (Nice to have)
- Performance optimizations
- Code duplication
- Missing documentation for public APIs
- Test coverage gaps

### Code Quality Standards

1. **Error Handling**
   - Use `Result<T, Error>` with proper error types
   - Avoid `.unwrap()` in production code
   - Use `.expect()` with descriptive messages for prototyping
   - Implement `From` trait for error conversions

2. **Async Code**
   - Check for potential deadlocks
   - Ensure proper cancellation handling
   - Use `tokio::select!` carefully

3. **Shell Integration**
   - Validate all user inputs
   - Sanitize commands before injection
   - Handle edge cases (empty strings, special characters)

4. **Testing**
   - Unit tests for core logic
   - Integration tests for shell hooks
   - Mock external dependencies

### Review Checklist

- [ ] No `.unwrap()` without proper error handling
- [ ] All `unsafe` blocks have safety comments
- [ ] Public APIs have documentation comments
- [ ] New dependencies are justified and minimal
- [ ] No hardcoded secrets or credentials
- [ ] Shell commands are properly escaped
- [ ] Async code handles cancellation gracefully
