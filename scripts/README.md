# XQPath 脚本目录

这个目录包含用于 XQPath 项目开发和维护的各种实用脚本。

## 📋 脚本列表

### `verify.sh`

- **用途**: 快速验证脚本入口点
- **功能**: 调用 `verify-debug.sh` 进行完整的调试功能验证
- **使用方法**:
  ```bash
  ./scripts/verify.sh
  ```

### `verify-debug.sh`

- **用途**: 验证 XQPath v1.4.1 调试功能的完整性
- **功能**:
  - 检查基础库编译
  - 检查调试功能编译
  - 检查 CLI 编译
  - 运行基础测试
  - 验证调试宏语法
  - 运行调试功能示例
  - 验证 API 示例
- **使用方法**:
  ```bash
  chmod +x scripts/verify-debug.sh
  ./scripts/verify-debug.sh
  ```

## 🚀 使用指南

### 运行验证脚本

```bash
# 从项目根目录执行（推荐方式）
cd /path/to/xqpath
./scripts/verify.sh

# 或者直接运行具体的验证脚本
chmod +x scripts/verify-debug.sh
./scripts/verify-debug.sh
```

### 脚本开发规范

1. **命名规范**: 使用有意义的名称，如 `verify-功能名.sh`
2. **权限设置**: 确保脚本具有执行权限
3. **文档说明**: 在脚本顶部添加说明注释
4. **错误处理**: 包含适当的错误检查和退出码
5. **输出格式**: 使用统一的输出格式（✅ 成功, ❌ 失败, ⚠️ 警告）

## 🔮 计划中的脚本

- `setup-dev.sh` - 开发环境设置脚本
- `run-benchmarks.sh` - 性能基准测试脚本
- `verify-v1.4.2.sh` - v1.4.2 版本验证脚本
- `build-release.sh` - 发布构建脚本
- `update-docs.sh` - 文档更新脚本

## 📝 贡献指南

如果您要添加新的脚本：

1. 确保脚本有清晰的用途和文档
2. 遵循现有的命名和格式规范
3. 包含适当的错误处理
4. 更新此 README 文件
