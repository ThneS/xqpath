# XQPath 配置文件说明

## 目录结构

```
config/
├── examples/           # 配置示例
│   ├── default.yaml   # 默认配置
│   ├── development.yaml # 开发环境配置
│   ├── production.yaml  # 生产环境配置
│   └── performance.yaml # 高性能配置
├── templates/          # 配置模板
│   ├── base.yaml      # 基础模板
│   └── debug.yaml     # 调试模板
└── profiles/          # 用户配置文件存储目录
```

## 配置文件格式

XQPath 使用 YAML 格式的配置文件，包含以下主要部分：

### debug 调试配置

- `level`: 日志级别 (trace, debug, info, warn, error)
- `output`: 输出目标 (stdout, stderr)
- `file`: 日志文件路径 (可选)
- `timing`: 是否启用计时统计

### performance 性能配置

- `memory_limit`: 内存限制 (支持 KB, MB, GB 单位)
- `timeout`: 操作超时时间 (支持 s, m, h 单位)
- `cache_size`: 缓存大小 (条目数)
- `parallel_jobs`: 并行任务数

### paths 路径配置

- `cache_dir`: 缓存目录
- `log_dir`: 日志目录
- `config_dir`: 配置目录

### features 功能配置

- `colored_output`: 是否启用彩色输出
- `interactive_mode`: 是否启用交互模式
- `auto_backup`: 是否启用自动备份

## 使用方式

### 查看当前配置

```bash
xqpath config show
```

### 设置配置值

```bash
xqpath config set debug.level warn
xqpath config set performance.cache_size 2000
```

### 使用配置模板

```bash
# 从模板创建配置
cp config/templates/base.yaml ~/.xqpath/config.yaml

# 或使用命令创建
xqpath config template create my-template
```

### 配置文件管理

```bash
# 创建配置文件
xqpath config profile create development

# 切换配置文件
xqpath config profile switch production

# 列出所有配置文件
xqpath config profile list
```

## 配置文件位置

- 默认配置文件: `~/.xqpath/config.yaml`
- 用户配置文件: `~/.xqpath/profiles/`
- 配置模板: `~/.xqpath/templates/`

## 环境变量覆盖

可以使用环境变量覆盖配置文件中的设置：

- `XQPATH_DEBUG_LEVEL`: 覆盖 debug.level
- `XQPATH_CACHE_SIZE`: 覆盖 performance.cache_size
- `XQPATH_LOG_FILE`: 覆盖 debug.file

## 配置验证

配置文件会在加载时进行验证，确保：

- 日志级别有效
- 内存限制格式正确
- 路径可访问
- 数值范围合理
