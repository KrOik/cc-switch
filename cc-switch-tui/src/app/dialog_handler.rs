use super::{App, PendingAction};
use crate::ui::dialog::{ConfirmDialog, ConfirmButton};
use anyhow::Result;
use crossterm::event::KeyCode;

impl App {
    /// 处理确认对话框的键盘输入
    pub fn handle_dialog_key(&mut self, key: KeyCode) -> Result<bool> {
        if let Some(dialog) = &mut self.confirm_dialog {
            match key {
                KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                    dialog.toggle_selection();
                    Ok(true) // 已处理
                }
                KeyCode::Enter => {
                    let confirmed = dialog.is_confirm_selected();
                    let action = self.pending_action.take();
                    self.confirm_dialog = None;

                    if confirmed {
                        if let Some(action) = action {
                            // 返回 false 表示需要执行操作
                            self.pending_action = Some(action);
                            Ok(false)
                        } else {
                            Ok(true)
                        }
                    } else {
                        // 用户取消
                        Ok(true)
                    }
                }
                KeyCode::Esc => {
                    self.confirm_dialog = None;
                    self.pending_action = None;
                    Ok(true) // 已处理
                }
                _ => Ok(true), // 忽略其他按键
            }
        } else {
            Ok(false) // 没有对话框，未处理
        }
    }

    /// 显示删除 Provider 的确认对话框
    pub fn show_delete_provider_confirm(&mut self, provider_id: String, provider_name: &str) {
        self.confirm_dialog = Some(
            ConfirmDialog::new(
                "删除 Provider",
                format!("确定要删除 Provider \"{}\" 吗？\n\n此操作无法撤销。", provider_name)
            )
            .with_confirm_text("删除")
            .with_cancel_text("取消")
        );
        self.pending_action = Some(PendingAction::DeleteProvider(provider_id));
    }

    /// 显示删除 MCP 服务器的确认对话框
    pub fn show_delete_mcp_confirm(&mut self, server_id: String, server_name: &str) {
        self.confirm_dialog = Some(
            ConfirmDialog::new(
                "删除 MCP 服务器",
                format!("确定要删除 MCP 服务器 \"{}\" 吗？\n\n此操作无法撤销。", server_name)
            )
            .with_confirm_text("删除")
            .with_cancel_text("取消")
        );
        self.pending_action = Some(PendingAction::DeleteMcpServer(server_id));
    }

    /// 显示停止代理的确认对话框
    pub fn show_stop_proxy_confirm(&mut self) {
        self.confirm_dialog = Some(
            ConfirmDialog::new(
                "停止代理服务",
                "确定要停止代理服务吗？\n\n所有活跃连接将被中断。"
            )
            .with_confirm_text("停止")
            .with_cancel_text("取消")
        );
        self.pending_action = Some(PendingAction::StopProxy);
    }

    /// 显示重启代理的确认对话框
    pub fn show_restart_proxy_confirm(&mut self) {
        self.confirm_dialog = Some(
            ConfirmDialog::new(
                "重启代理服务",
                "确定要重启代理服务吗？\n\n所有活跃连接将被中断。"
            )
            .with_confirm_text("重启")
            .with_cancel_text("取消")
        );
        self.pending_action = Some(PendingAction::RestartProxy);
    }

    /// 检查是否有待处理的操作需要执行
    pub fn take_pending_action(&mut self) -> Option<PendingAction> {
        self.pending_action.take()
    }

    /// 检查是否正在显示对话框
    pub fn has_dialog(&self) -> bool {
        self.confirm_dialog.is_some()
    }

    /// 获取当前对话框的引用
    pub fn get_dialog(&self) -> Option<&ConfirmDialog> {
        self.confirm_dialog.as_ref()
    }
}
