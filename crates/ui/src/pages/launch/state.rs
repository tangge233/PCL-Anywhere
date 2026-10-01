//! 启动页的示例数据与登录面板形态。
//!
//! 账号服务与日志服务尚未接入，[`SAMPLE_PROFILE_NAME`] 等常量只是把界面填成合理内容；
//! 接入后应替换为视图模型提供的数据。

use gpui_kit::*;
use std::rc::Rc;

use crate::shell::Route;

/// 示例数据：账号服务与日志服务尚未接入，先把界面填充成合理内容。
pub(super) const SAMPLE_PROFILE_NAME: &str = "Steve";
pub(super) const SAMPLE_DEVICE_CODE: &str = "ABCD-EFGH";
pub(super) const MICROSOFT_LINK_URL: &str = "microsoft.com/link";
pub(super) const SAMPLE_AUTH_SERVER: &str = "LittleSkin";
pub(super) const SAMPLE_LOG: &str = "[12:00:00] [Init] PCL-Anywhere 已就绪\n\
[12:00:00] [Init] 未选择游戏实例";

/// 进入实例副页面的回调（外壳统一处理导航）。
pub(super) type InstanceHandler = Rc<dyn Fn(&Route, &mut Window, &mut App)>;

/// 登录面板当前显示的形态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LoginPanel {
    /// 已登录：显示账号名与两个按钮。
    Profile,
    /// 登录方式选择。
    Select,
    /// 微软设备代码登录。
    Microsoft,
    /// 第三方验证。
    Authlib,
    /// 离线登录。
    Offline,
}
