# XQPath 测试优化配置

#

# 本文件定义了测试分组策略和优化建议

## 🎯 测试分组策略

### 1. 核心测试组 (core)

- **文件**: `integration.rs`, `lib.rs`
- **特点**: 不依赖额外 features，测试基础 API
- **运行时间**: ~2 秒
- **用途**: 快速验证核心功能

### 2. 配置管理测试组 (config)

- **文件**: `config_debug_features.rs` (配置部分)
- **Features**: `config-management`
- **运行时间**: ~1 秒
- **用途**: 验证配置系统

### 3. 调试功能测试组 (debug)

- **文件**: `config_debug_features.rs` (调试部分)
- **Features**: `interactive-debug`
- **运行时间**: ~1 秒
- **用途**: 验证交互式调试器

### 4. 高级功能测试组 (advanced)

- **文件**: `advanced_functions.rs`, `builtin_functions.rs`
- **Features**: 根据需要
- **运行时间**: ~3 秒
- **用途**: 验证扩展功能

## 📊 当前测试状态分析

### 测试文件统计

```
文件名                    | 行数 | 测试数 | 主要功能
-------------------------|------|--------|----------
integration.rs           | 323  | ~20    | 基础集成测试
advanced_functions.rs    | 357  | 11     | 高级函数测试
config_debug_features.rs | 167  | 9      | 配置和调试测试
error_handling.rs        | 174  | ~15    | 错误处理测试
builtin_functions.rs     | 102  | 3      | 内置函数测试
conditional_expressions.rs| 147 | 5      | 条件表达式测试
```

**总计**: ~1488 行代码，~85 个测试用例

## 🚀 优化建议

### 1. 清理重复测试

```bash
# 删除重复的测试文件
rm tests/v1_4_3_features.rs
```

### 2. 测试分层执行

```bash
# 开发时快速测试
./scripts/test-runner.sh quick

# 功能特定测试
./scripts/test-runner.sh config
./scripts/test-runner.sh debug

# 完整测试
./scripts/test-runner.sh all
```

### 3. 并行优化

```bash
# 启用并行测试
./scripts/test-runner.sh quick --fast

# 静默模式
./scripts/test-runner.sh all --quiet
```

### 4. CI/CD 分阶段

```yaml
# .github/workflows/test.yml 建议配置
stages:
  - quick_test # 核心功能 (~3秒)
  - feature_test # 功能特性 (~5秒)
  - full_test # 完整测试 (~10秒)
```

## 📋 测试用例优化

### 减少测试重复度

1. **合并相似测试**: 将 config 和 debug 测试合并到一个文件
2. **提取公共 setup**: 创建测试工具函数
3. **减少 IO 操作**: 使用内存临时文件
4. **并行安全**: 避免全局状态依赖

### 提高测试效率

1. **条件编译**: 使用`#[cfg(feature = "...")]`精确控制
2. **测试分组**: 用`#[ignore]`标记慢速测试
3. **mock 依赖**: 减少外部依赖
4. **缓存结果**: 重用测试数据

## 🛠️ 使用指南

### 日常开发

```bash
# 快速验证更改
./scripts/test-runner.sh core

# 测试特定功能
./scripts/test-runner.sh config --verbose
```

### 发布前

```bash
# 完整测试套件
./scripts/test-runner.sh all --fast

# 性能回归测试
./scripts/test-runner.sh performance
```

### 调试测试

```bash
# 详细输出
./scripts/test-runner.sh debug --verbose

# 单个测试
cargo test test_config_manager_basic --features config-management -- --nocapture
```

## 🎯 推荐的测试流程

### 1. 开发阶段

- 使用 `./scripts/test-runner.sh quick` 进行快速验证
- 针对修改的功能运行对应的测试组

### 2. 功能完成

- 运行 `./scripts/test-runner.sh integration` 确保集成正常
- 检查相关的 feature 测试

### 3. 发布准备

- 运行 `./scripts/test-runner.sh all` 进行完整测试
- 执行性能基准测试

这样可以将测试时间从 ~15 秒 优化到 ~3 秒（日常开发），同时保持完整的测试覆盖率。
