# TUI 功能审计报告

生成时间：2026-03-01
审计范围：TUI 与后端链接正确性、功能有效性、与 GUI 一致性

## 执行摘要

**总体评估：⚠️ 部分功能不完整**

- ✅ 数据库操作：正确
- ✅ 表单验证：完整
- ⚠️ Provider 切换：**严重缺陷**
- ✅ MCP 管理：完整
- ✅ Proxy 控制：完整
- ⚠️ 统一供应商：部分缺失

---

## 1. Provider 管理功能

### 1.1 Provider 切换 - ⚠️ 严重缺陷

**TUI 当前实现** (`cc-switch-tui/src/app/actions.rs:5-16`):
```rust
pub async fn switch_provider(&mut self, provider_id: &str) -> Result<()> {
    log::info!("Switching to provider: {}", provider_id);

    // 更新数据库中的当前 provider
    self.db.set_current_provider(&self.current_app_type, provider_id)?;

    // 刷新缓存
    self.refresh_providers()?;

    Ok(())
}
```

**GUI 完整实现** (`cc-switch-core/src/services/provider/mod.rs:434-594`):
```rust
pub fn switch(state: &AppState, app_type: AppType, id: &str) -> Result<SwitchResult, AppError> {
    // 1. 验证 provider 存在
    // 2. 检查代理接管模式
    // 3. 如果接管模式：热切换（不写 Live 配置）
    // 4. 如果正常模式：
    //    a. Backfill：保存当前 live 配置到旧 provider
    //    b. 更新本地 settings (device-level)
    //    c. 更新数据库 is_current
    //    d. 写入新 provider 配置到 live 文件
    //    e. 同步 MCP 配置
}
```

**缺失的关键步骤：**

❌ **Backfill 机制**
- GUI: 切换前保存当前 live 配置到旧 provider
- TUI: 完全缺失
- 影响：用户对当前 provider 的修改会丢失

❌ **本地 settings 更新**
- GUI: `crate::settings::set_current_provider(&app_type, Some(id))`
- TUI: 完全缺失
- 影响：设备级别的 provider 选择不生效

❌ **Live 配置写入**
- GUI: `write_live_snapshot(&app_type, provider)`
- TUI: 完全缺失
- 影响：**Claude/Codex/Gemini 不会读取新 provider 的配置，切换无效**

❌ **MCP 同步**
- GUI: `McpService::sync_all_enabled(state)`
- TUI: 完全缺失
- 影响：MCP 服务器配置不会更新

❌ **代理接管模式处理**
- GUI: 检测接管模式，热切换或更新备份
- TUI: 完全缺失
- 影响：代理接管模式下切换行为不正确

**严重性：🔴 高危**
- Provider 切换是核心功能
- 当前实现**完全无法工作**（只更新数据库，不写配置文件）
- 用户会认为切换成功，但实际上应用仍在使用旧 provider

---

### 1.2 Provider CRUD - ✅ 正确

**添加 Provider**:
- ✅ 调用 `db.save_provider()` - 正确
- ✅ 表单验证 JSON 格式 - 正确
- ⚠️ 缺少 `write_live_snapshot()` - 如果是第一个 provider 应该写入 live 配置

**编辑 Provider**:
- ✅ 调用 `db.save_provider()` - 正确
- ⚠️ 缺少检查是否为当前 provider
- ⚠️ 如果是当前 provider，应该调用 `write_live_snapshot()`

**删除 Provider**:
- ✅ 调用 `db.delete_provider()` - 正确
- ⚠️ 缺少检查是否为当前 provider（应该阻止删除）

---

## 2. MCP 管理功能

### 2.1 MCP CRUD - ✅ 完整

**添加/编辑 MCP**:
- ✅ 调用 `db.save_mcp_server()` - 正确
- ✅ 保存应用启用状态 - 正确
- ✅ 表单验证 - 正确

**删除 MCP**:
- ✅ 调用 `db.delete_mcp_server()` - 正确
- ✅ 确认对话框 - 正确

**应用切换**:
- ✅ 调用 `db.save_mcp_server()` 更新状态 - 正确
- ⚠️ 缺少 `McpService::sync_all_enabled()` - 应该同步到 live 配置

---

## 3. Proxy 管理功能

### 3.1 Proxy 控制 - ✅ 完整

**启动/停止**:
- ✅ 调用 `proxy_service.start()` - 正确
- ✅ 调用 `proxy_service.stop()` - 正确
- ✅ 更新状态缓存 - 正确

**接管切换**:
- ✅ 调用 `proxy_service.set_takeover_for_app()` - 正确
- ✅ 备份和恢复 Live 配置 - 正确
- ✅ 更新本地状态 - 正确

---

## 4. 统一供应商管理

### 4.1 统一供应商 CRUD - ⚠️ 部分缺失

**添加/编辑**:
- ✅ 调用 `db.save_universal_provider()` - 正确
- ✅ 表单验证 - 正确
- ⚠️ 缺少同步到各应用的 live 配置

**删除**:
- ✅ 调用 `db.delete_universal_provider()` - 正确
- ✅ 确认对话框 - 正确

**同步功能**:
- ❌ `sync_universal_provider()` 是占位实现
- 应该调用 `UniversalProviderService` 的同步方法

---

## 5. 与 GUI 功能对比

### 5.1 核心功能覆盖率

| 功能 | GUI | TUI | 一致性 |
|------|-----|-----|--------|
| Provider 列表 | ✅ | ✅ | ✅ |
| Provider 切换 | ✅ 完整 | ⚠️ 不完整 | ❌ |
| Provider 添加 | ✅ | ✅ | ⚠️ 部分 |
| Provider 编辑 | ✅ | ✅ | ⚠️ 部分 |
| Provider 删除 | ✅ | ✅ | ⚠️ 部分 |
| MCP 管理 | ✅ | ✅ | ✅ |
| MCP 同步 | ✅ | ⚠️ 部分 | ⚠️ |
| Proxy 控制 | ✅ | ✅ | ✅ |
| Proxy 接管 | ✅ | ✅ | ✅ |
| 统一供应商 | ✅ | ✅ | ⚠️ 部分 |

### 5.2 缺失的关键服务调用

TUI 缺少以下关键服务调用：

1. **ProviderService::switch()** - 应该使用完整的切换逻辑
2. **write_live_snapshot()** - 写入 live 配置文件
3. **McpService::sync_all_enabled()** - 同步 MCP 配置
4. **crate::settings::set_current_provider()** - 更新本地 settings
5. **read_live_settings()** - 读取 live 配置用于 backfill

---

## 6. 修复建议

### 6.1 高优先级修复（必须）

**1. 修复 Provider 切换逻辑**

TUI 应该使用 `ProviderService::switch()` 而不是直接调用数据库：

```rust
// 错误的当前实现
self.db.set_current_provider(&self.current_app_type, provider_id)?;

// 正确的实现
use cc_switch_core::services::ProviderService;
let app_type = AppType::from_str(&self.current_app_type)?;
ProviderService::switch(&self.get_app_state(), app_type, provider_id)?;
```

**问题：** TUI 没有 `AppState`，需要重构架构。

**2. 添加 Live 配置写入**

在 Provider 添加/编辑时，如果是当前 provider，需要写入 live 配置。

**3. 添加 MCP 同步**

在 MCP 应用切换后，调用 `McpService::sync_all_enabled()`。

### 6.2 中优先级修复（建议）

**1. 添加当前 provider 检查**

删除/编辑 provider 前检查是否为当前使用的。

**2. 完善统一供应商同步**

实现真实的同步逻辑，将统一供应商配置写入各应用。

---

## 7. 架构问题

### 7.1 核心问题：缺少 AppState

TUI 直接使用 `Database` 和 `ProxyService`，但很多业务逻辑在 `ProviderService` 等服务层中，这些服务需要 `AppState`。

**GUI 架构**:
```
Tauri Commands → Services (ProviderService, McpService) → Database + Live Config
```

**TUI 架构**:
```
TUI Actions → Database (直接调用)
```

**问题**:
- TUI 绕过了服务层的业务逻辑
- 缺少 live 配置写入
- 缺少 MCP 同步
- 缺少 backfill 机制

### 7.2 建议的架构改进

**选项 A: 创建 AppState**
```rust
pub struct App {
    state: Arc<AppState>,  // 包含 db, proxy_service 等
    // ...
}
```

**选项 B: 将服务逻辑移到 cc-switch-core**
创建不依赖 AppState 的服务方法：
```rust
impl ProviderService {
    pub fn switch_simple(
        db: &Database,
        proxy_service: &ProxyService,
        app_type: &str,
        provider_id: &str
    ) -> Result<()>
}
```

---

## 8. 测试建议

### 8.1 关键测试场景

1. **Provider 切换测试**
   - 切换 provider 后，检查 live 配置文件是否更新
   - 启动 Claude/Codex，验证是否使用新 provider

2. **MCP 同步测试**
   - 切换 MCP 应用启用状态后，检查应用配置文件

3. **Proxy 接管测试**
   - 开启接管后切换 provider，验证热切换行为

---

## 9. 结论

**当前状态：⚠️ 部分功能不可用**

- ✅ 数据库操作正确
- ✅ UI 交互完整
- ❌ **Provider 切换功能无法工作**（只更新数据库，不写配置文件）
- ⚠️ 部分功能缺少配置同步

**建议：**
1. **立即修复** Provider 切换逻辑（高危问题）
2. 添加 live 配置写入和 MCP 同步
3. 考虑架构重构，引入服务层或 AppState

**预估工作量：**
- 快速修复（绕过服务层）：2-3 小时
- 完整重构（引入 AppState）：6-8 小时
