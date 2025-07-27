# XQPath 文档索引

本目录包含 XQPath 项目的所有设计文档、规划文档和发布记录，按照 RFC 标准进行组织。

## 📁 文档结构

### 📋 RFC 文档 (`rfcs/`)

正式的设计文档和技术规范，遵循 RFC 格式：

- **RFC-001**: [jq 语法兼容性分析](rfcs/RFC-001-jq-syntax-compatibility.md)
- **RFC-002**: [v1.1 表达式系统设计](rfcs/RFC-002-expression-system.md)
- **RFC-003**: [v1.2 内置函数系统](rfcs/RFC-003-builtin-functions.md)
- **RFC-004**: [v1.3 用户自定义函数](rfcs/RFC-004-user-defined-functions.md)

### 🎯 规划文档 (`planning/`)

项目发展规划和路线图：

- [v1.2 开发路线图](planning/roadmap-v1.2.md)
- [v1.3 开发路线图](planning/roadmap-v1.3.md)
- [发布后后续规划](planning/post-release-roadmap.md)

### 📦 发布文档 (`releases/`)

版本发布记录和报告：

- [v0.0.1 发布报告](releases/v0.0.1-release-report.md)
- [v1.1 进度报告](releases/v1.1-progress-report.md)
- [v1.2 发布说明](releases/v1.2-release-notes.md)

### 🏗️ 设计文档 (`design/`)

具体功能的设计文档：

- [用户函数实现计划](design/user-functions-implementation.md)
- [模块系统设计](design/module-system-design.md)
- [错误处理机制](design/error-handling-design.md)

### 📚 归档文档 (`archive/`)

历史文档和已完成的设计文档：

- [v1.1 实现计划](archive/implementation-plan-v1.1.md)

## 📖 阅读指南

### 🚀 快速开始

- 新用户：先阅读 [README.md](../README.md)
- 开发者：查看 [RFC 文档](rfcs/) 了解技术设计
- 贡献者：参考 [规划文档](planning/) 了解项目方向

### 🔍 查找信息

- **技术规范**: 查看 `rfcs/` 目录
- **开发进度**: 查看 `releases/` 目录
- **未来规划**: 查看 `planning/` 目录
- **实现细节**: 查看 `design/` 目录

### 📝 文档标准

#### RFC 文档格式

```markdown
# RFC-XXX: [标题]

## 摘要

简短描述提案内容...

## 动机

为什么需要这个功能...

## 详细设计

技术实现细节...

## 实现计划

具体的实现步骤...

## 未解决的问题

需要进一步讨论的问题...
```

#### 版本命名规则

- **RFC**: RFC-001, RFC-002, ...
- **发布**: v1.0.0, v1.1.0, v1.2.0, ...
- **规划**: roadmap-v1.x, planning-YYYY-MM, ...

## 🔄 维护指南

### 文档生命周期

1. **草案** (Draft) - 初始设计阶段
2. **审查** (Review) - 社区审查阶段
3. **接受** (Accepted) - 正式采纳
4. **实现** (Implemented) - 功能实现完成
5. **归档** (Archived) - 移入归档目录

### 更新流程

1. 新功能设计 → 创建 RFC 文档
2. 实现完成 → 更新发布文档
3. 版本发布 → 创建发布报告
4. 功能弃用 → 移入归档目录

---

**文档维护者**: XQPath 项目组
**最后更新**: 2025 年 7 月 27 日
**格式版本**: v1.0
