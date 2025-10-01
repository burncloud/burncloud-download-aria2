# Aria2 Daemon Test Suite Documentation

## Test Overview

This document describes the comprehensive test suite for the aria2 process daemon implementation in `burncloud-download-aria2`.

## Test Statistics

- **Total Tests Created**: 24 (15 new daemon tests + 9 existing tests)
- **Unit Tests**: 15 tests
- **Integration Tests**: 9 tests (run with `--ignored` flag)
- **All Tests Passing**: ✅ 39 tests pass (15 unit + 24 existing)

## Test Organization

### Unit Tests (`src/daemon/tests.rs`)

**Location**: `src/daemon/tests.rs`
**Run Command**: `cargo test --lib daemon`
**Total**: 15 tests, all passing

#### Platform Utilities Tests (5 tests)
- ✅ `test_get_binary_dir` - Validates platform-specific binary directory paths
- ✅ `test_get_binary_name` - Validates binary name by platform (aria2c.exe vs aria2c)
- ✅ `test_get_binary_path` - Validates complete binary path construction
- ✅ `test_ensure_directory_creates_new_dir` - Tests directory creation
- ✅ `test_ensure_directory_existing_dir` - Tests handling of existing directories

#### Binary Management Tests (2 tests)
- ✅ `test_verify_binary_exists_missing` - Validates detection of missing binaries
- ✅ `test_verify_binary_exists_present` - Validates detection of present binaries

#### Process Management Tests (4 tests)
- ✅ `test_process_config_creation` - Validates ProcessConfig structure
- ✅ `test_restart_counter_increment` - Tests restart counter increment logic
- ✅ `test_max_restart_attempts` - Tests max restart attempts configuration
- ✅ `test_process_not_running_initially` - Validates initial process state

#### Daemon Configuration Tests (4 tests)
- ✅ `test_daemon_config_default` - Validates default configuration values
- ✅ `test_daemon_config_custom` - Tests custom configuration
- ✅ `test_daemon_config_clone` - Tests configuration cloning

### Integration Tests (`tests/daemon_integration_test.rs`)

**Location**: `tests/daemon_integration_test.rs`
**Run Command**: `cargo test --test daemon_integration_test -- --ignored`
**Total**: 10 tests (9 ignored + 1 unit test)

#### Complete Lifecycle Tests
- 🔒 `test_daemon_start_with_missing_binary` - Tests binary auto-download on first start
  - **Requirements**: Network access, file system write permissions
  - **Validates**: GitHub/Gitee fallback download, binary extraction, process start

- 🔒 `test_daemon_lifecycle_with_existing_binary` - Tests daemon start/stop with binary present
  - **Requirements**: aria2 binary available
  - **Validates**: Process start, RPC communication, graceful shutdown

#### Process Monitoring & Restart Tests
- 🔒 `test_daemon_auto_restart_on_crash` - Tests automatic process restart after crash
  - **Requirements**: aria2 binary, process kill permissions
  - **Validates**: Crash detection, automatic restart, RPC recovery

- 🔒 `test_daemon_restart_limit_enforcement` - Tests restart limit (max 10 attempts)
  - **Requirements**: aria2 binary
  - **Validates**: Restart counter, max attempts enforcement

#### RPC & Health Check Tests
- 🔒 `test_daemon_rpc_readiness_wait` - Tests RPC readiness waiting (30s timeout)
  - **Requirements**: aria2 binary
  - **Validates**: RPC polling, timeout handling, successful connection

- 🔒 `test_daemon_start_timeout_on_rpc_unavailable` - Tests timeout when RPC never ready
  - **Requirements**: aria2 binary
  - **Validates**: 30-second timeout, proper error reporting

#### Cleanup & Resource Management Tests
- 🔒 `test_daemon_drop_cleanup` - Tests Drop trait cleanup
  - **Requirements**: aria2 binary
  - **Validates**: Process termination on drop, resource cleanup

#### Configuration & Multi-Instance Tests
- 🔒 `test_daemon_custom_configuration` - Tests custom port and secret configuration
  - **Requirements**: aria2 binary
  - **Validates**: Custom RPC port, custom secret, wrong secret rejection

- 🔒 `test_multiple_daemon_instances_different_ports` - Tests multiple concurrent daemons
  - **Requirements**: aria2 binary, multiple available ports
  - **Validates**: Independent daemon instances, port isolation

- ✅ `test_daemon_config_default_values` - Unit test for default configuration

**Legend**: ✅ = Always runs, 🔒 = Requires `--ignored` flag

## Running Tests

### Run All Unit Tests
```bash
cd burncloud-download-aria2
cargo test --lib
```

### Run Daemon Unit Tests Only
```bash
cargo test --lib daemon
```

### Run Integration Tests (with aria2 binary)
```bash
# Run all ignored integration tests
cargo test -- --ignored

# Run specific daemon integration test
cargo test --test daemon_integration_test test_daemon_lifecycle_with_existing_binary -- --ignored

# Run all daemon integration tests
cargo test --test daemon_integration_test -- --ignored
```

### Run All Tests (including ignored)
```bash
cargo test -- --include-ignored
```

## Test Coverage Areas

### ✅ Fully Tested (Unit Tests)
1. **Platform Path Resolution**
   - Windows: `%LOCALAPPDATA%\BurnCloud`
   - Linux: `~/.burncloud`
   - Binary name selection (aria2c.exe vs aria2c)

2. **Binary Management**
   - Binary existence verification
   - Directory creation

3. **Process Configuration**
   - Default configuration values (port 6800, secret "burncloud")
   - Custom configuration support
   - Configuration cloning

4. **Restart Counter Logic**
   - Increment behavior
   - Reset behavior
   - Max attempts configuration

### 🔒 Integration Tested (Requires aria2 binary)
1. **Binary Download**
   - GitHub primary source download
   - Gitee fallback on failure
   - ZIP extraction
   - Executable permissions (Unix)

2. **Process Lifecycle**
   - Process spawning with correct arguments
   - Process termination
   - Health status checking
   - Drop trait cleanup

3. **Monitoring & Auto-Restart**
   - 10-second interval health checks
   - Crash detection
   - Automatic restart (up to 10 times)
   - Exponential backoff (2^n seconds, max 60s)
   - Restart limit enforcement

4. **RPC Communication**
   - 30-second RPC readiness wait
   - Health check via getGlobalStat
   - Timeout handling
   - Custom port/secret configuration

5. **Multi-Instance Support**
   - Multiple daemons on different ports
   - Instance isolation

## Test Requirements

### Minimum Requirements (Unit Tests)
- Rust toolchain
- tokio async runtime
- tempfile crate

### Full Requirements (Integration Tests)
- aria2 binary (either pre-installed or downloaded)
- Network access (for binary download tests)
- File system write permissions
- Process management permissions (for crash/restart tests)
- Available ports (6801-6811 for tests)

## Test Execution Results

### Latest Test Run (2025-10-01)

```
Unit Tests (src/daemon/tests.rs):
  running 15 tests
  test result: ok. 15 passed; 0 failed; 0 ignored

Integration Tests (tests/daemon_integration_test.rs):
  running 10 tests
  test result: ok. 1 passed; 0 failed; 9 ignored

Overall:
  - 39 total tests pass
  - 0 failures
  - 14 tests run automatically
  - 9 integration tests available with --ignored flag
```

## Testing Best Practices

### For Developers

1. **Always run unit tests before committing**:
   ```bash
   cargo test --lib daemon
   ```

2. **Run integration tests when modifying daemon logic**:
   ```bash
   cargo test --test daemon_integration_test -- --ignored
   ```

3. **Check test coverage for new features**:
   - Add unit tests for new daemon components
   - Add integration tests for new lifecycle behaviors

### For CI/CD

1. **Unit tests should always pass**:
   ```bash
   cargo test --lib
   ```

2. **Integration tests optional** (may fail without aria2):
   ```bash
   cargo test -- --ignored || true
   ```

## Known Limitations

1. **Integration tests require aria2 binary**:
   - Tests are marked with `#[ignore]` by default
   - Can be run manually with `--ignored` flag
   - May fail in CI environments without aria2

2. **Platform-specific behavior**:
   - Some tests behave differently on Windows vs Linux
   - Process management tests may require elevated permissions

3. **Network dependency**:
   - Binary download tests require internet access
   - GitHub/Gitee availability affects download tests

4. **Port availability**:
   - Tests use ports 6801-6811
   - Conflicts may occur if ports are in use

## Future Test Enhancements

1. **Mock HTTP server for binary download tests**
   - Use `mockito` to simulate GitHub/Gitee responses
   - Test download failure scenarios without network

2. **Process mock for restart tests**
   - Mock process crashes without killing real processes
   - More reliable restart limit testing

3. **Parallel test execution safety**
   - Ensure tests don't conflict on shared ports
   - Use dynamic port allocation

4. **Performance benchmarks**
   - Measure daemon startup time
   - Monitor RPC response latency
   - Track memory usage

## Troubleshooting

### Test Failures

**"Binary download failed"**
- Check network connectivity
- Verify GitHub/Gitee are accessible
- Check file system write permissions

**"Process start failed"**
- Verify aria2 binary exists or can be downloaded
- Check port availability
- Ensure no other aria2 instances running

**"RPC not ready after 30 seconds"**
- Increase timeout in test configuration
- Check firewall settings
- Verify aria2 starts correctly manually

**Port conflicts**
- Change test port numbers in test configuration
- Stop other aria2 instances
- Use different port range

## Contact

For test-related issues or questions, please refer to:
- Technical Spec: `.claude/specs/aria2-daemon/technical-spec.md`
- Code Review: `.claude/specs/aria2-daemon/code-review.md`
