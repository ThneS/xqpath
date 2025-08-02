# XQPath 性能测试和基准测试结构

## 目录结构

```
├── benches/
│   ├── performance.rs           # 基础 Criterion 基准测试
│   └── advanced_benchmarks.rs   # 高级基准测试套件 (整合了 debug/benchmark.rs 功能)
├── examples/
│   └── performance_demo.rs      # 性能分析功能演示示例 (保持不变)
├── src/debug/
│   ├── benchmark.rs            # 基准测试框架和工具 (库代码)
│   └── profiler.rs             # 性能分析器 (库代码)
└── scripts/
    └── run-benchmarks.sh       # 基准测试运行脚本
```

## 各文件作用

### 1. benches/performance.rs

- **作用**: 基础 Criterion 基准测试
- **内容**: 使用 `evaluate_path_expression` 和 `parse_path_expression` 的传统基准测试
- **运行**: `cargo bench --bench performance`

### 2. benches/advanced_benchmarks.rs

- **作用**: 高级基准测试套件，整合了 debug/benchmark.rs 的功能
- **内容**:
  - 多种复杂度测试数据生成
  - 数据集大小对性能影响测试
  - 嵌套深度对性能影响测试
  - 性能分析功能开销测试
  - 自定义基准测试套件使用演示
- **功能**: 支持条件编译，根据可用功能选择测试
- **运行**: `cargo bench --bench advanced_benchmarks --features="profiling,benchmark"`

### 3. examples/performance_demo.rs

- **作用**: 性能分析功能演示示例 (保持不变)
- **内容**:
  - 展示 v1.4.2 性能监控功能的使用方法
  - 包含基础性能分析、内存分析、基准测试演示
  - 生成 HTML 报告的演示
- **运行**: `cargo run --features="profiling,benchmark" --example performance_demo`
- **注意**: 这是演示示例，不是基准测试

### 4. src/debug/benchmark.rs

- **作用**: 基准测试框架和工具的库实现
- **内容**:
  - `BenchmarkSuite`, `BenchmarkResult`, `BenchmarkConfig` 结构体
  - 基准测试执行逻辑
  - 报告生成功能
- **用途**: 被其他模块和示例使用的库代码

### 5. src/debug/profiler.rs

- **作用**: 性能分析器的库实现
- **内容**:
  - `PerformanceMonitor`, `MemoryProfiler`, `ProfileReport` 结构体
  - 性能监控逻辑
  - HTML 报告生成
- **用途**: 被其他模块和示例使用的库代码

### 6. scripts/run-benchmarks.sh

- **作用**: 统一的基准测试运行脚本
- **功能**:
  - 检查功能可用性
  - 运行所有基准测试
  - 运行性能演示
  - 生成完整报告
- **运行**: `./scripts/run-benchmarks.sh`

## 运行方式

### 运行所有测试

```bash
./scripts/run-benchmarks.sh
```

### 运行特定基准测试

```bash
# 基础基准测试
cargo bench --bench performance

# 高级基准测试 (需要 profiling 和 benchmark 功能)
cargo bench --bench advanced_benchmarks --features="profiling,benchmark"

# 仅基础功能的高级基准测试
cargo bench --bench advanced_benchmarks
```

### 运行性能演示

```bash
# 完整功能演示
cargo run --features="profiling,benchmark" --example performance_demo

# 基础功能演示
cargo run --example performance_demo
```

## 特性

1. **保持向后兼容**: 原有的 performance.rs 基准测试保持不变
2. **功能演示分离**: performance_demo.rs 作为演示示例保持在 examples/ 目录
3. **高级功能集成**: 新的 advanced_benchmarks.rs 整合了 debug/benchmark.rs 的高级功能
4. **条件编译支持**: 根据可用 feature 自动选择测试内容
5. **统一运行脚本**: 提供一键运行所有测试的脚本

## 报告文件

运行后会生成以下报告文件：

- `target/criterion/` - Criterion HTML 报告
- `benchmark_report.html` - 自定义基准测试报告
- `performance_report.html` - 性能监控报告
