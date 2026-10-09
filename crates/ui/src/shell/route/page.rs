//! 页面目录：一页一条登记项，`nav_pages!` / `setting_pages!` 由同一张表生成
//! 「子路由枚举 + 有序目录 + 下标换算 + 内容分派」。
//!
//! 加 / 删子页只动该组的表文件（表一行 + 同文件的顺序快照）。
//!
//! 这一层不认识具体分组：元数据与分组无关，同一个 [`PageMeta`] 同时服务左栏页（下载 / 工具）
//! 与设置分类；渲染入口、标题与图标由各组的登记行给出。

use gpui_kit::{AnyElement, Context, SharedString, Window};

use crate::shell::route::Route;

/// 一页的目录信息。
pub struct PageMeta<R: Copy + 'static> {
    pub route: R,
    pub title_key: &'static str,
    pub icon: &'static str,
    /// 分组标题键名：非空即从这一条起另起一组（PCL 的 `SelectorSectionKey`）。
    pub section: Option<&'static str>,
}

/// 一组子路由的公共面：外壳与左栏只认这个 trait。
pub trait PageRoute: Copy + Eq + std::hash::Hash + std::fmt::Debug + 'static {
    /// 全部页面，顺序即显示顺序（宏生成）。
    const ALL: &'static [PageMeta<Self>];
    /// 表内下标。
    fn index(self) -> usize;
    /// 包回顶层路由：`Route::Download(self)`。
    fn route(self) -> Route;
    fn meta(self) -> &'static PageMeta<Self> {
        &Self::ALL[self.index()]
    }
    /// 该页标题：左栏与占位文案都取这里。
    fn title(self) -> SharedString {
        crate::i18n::lang(self.meta().title_key)
    }
}

/// 分组按路由渲染内容；实现由 `nav_pages!` 生成。
pub trait PageHost<R: PageRoute>: Sized {
    fn render_page(&mut self, route: R, window: &mut Window, cx: &mut Context<Self>) -> AnyElement;
}

/// 内部：可选的分组标题字面量 → [`PageMeta::section`]。
macro_rules! section_key {
    () => {
        None
    };
    ($key:literal) => {
        Some($key)
    };
}

/// 内容分派：有渲染入口就用它，否则渲染该页的占位。
macro_rules! page_content {
    ($this:expr, $window:expr, $cx:expr, $route:expr, $render:path) => {
        $render($this, $window, $cx)
    };
    ($this:expr, $window:expr, $cx:expr, $route:expr) => {
        $crate::pages::frame::page_placeholder($route)
    };
}

/// 内部：由行列表生成子路由枚举与 [`PageRoute`] 实现（两个对外的宏共用）。
macro_rules! route_table {
    (
        $(#[$meta:meta])*
        $vis:vis enum $route:ident as Route::$variant:ident {
            $( $name:ident : $title:literal , $icon:literal $(, section $section:literal)? );* $(;)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        $vis enum $route {
            $( $name ),*
        }

        impl $crate::shell::route::page::PageRoute for $route {
            const ALL: &'static [$crate::shell::route::page::PageMeta<Self>] = &[
                $(
                    $crate::shell::route::page::PageMeta {
                        route: Self::$name,
                        title_key: $title,
                        icon: $icon,
                        section: $crate::shell::route::page::section_key!($($section)?),
                    }
                ),*
            ];

            fn index(self) -> usize {
                self as usize
            }

            fn route(self) -> $crate::shell::route::Route {
                $crate::shell::route::Route::$variant(self)
            }
        }
    };
}

/// 声明一组「左栏页」：一张表同时生成子路由枚举、目录与内容分派。
///
/// 每行是 `变体名: "文案键", "图标"`，后面可以跟（顺序固定）：
/// - `, section "分组标题键"` —— 从这一条起另起一组；
/// - `, render 渲染入口` —— 这一页的渲染函数（`fn(&mut 分组, &mut Window, &mut Context<分组>) -> AnyElement`）。
///
/// 不写 `render` 就是尚未迁移的页：外壳按该页标题自动渲染占位。
///
/// ```ignore
/// nav_pages! {
///     pub enum DownloadRoute for DownloadGroup as Route::Download {
///         Minecraft: "Download.Left.Minecraft", "boxes", render DownloadGroup::render_minecraft;
///         Mod: "Download.Left.Mod", "puzzle", section "Download.Left.CommunityResources";
///     }
/// }
/// ```
macro_rules! nav_pages {
    (
        $(#[$meta:meta])*
        $vis:vis enum $route:ident for $group:path as Route::$variant:ident {
            $(
                $name:ident : $title:literal , $icon:literal
                $( , section $section:literal )?
                $( , render $render:path )?
            );* $(;)?
        }
    ) => {
        $crate::shell::route::page::route_table! {
            $(#[$meta])*
            $vis enum $route as Route::$variant {
                $( $name : $title , $icon $(, section $section)? );*
            }
        }

        impl $crate::shell::route::page::PageHost<$route> for $group {
            fn render_page(
                &mut self,
                route: $route,
                window: &mut ::gpui_kit::Window,
                cx: &mut ::gpui_kit::Context<Self>,
            ) -> ::gpui_kit::AnyElement {
                match route {
                    $(
                        $route::$name => {
                            $crate::shell::route::page::page_content!(
                                self, window, cx, route $(, $render)?
                            )
                        }
                    )*
                }
            }
        }
    };
}

/// 声明一组「设置分类」：一张表生成子路由枚举、目录与按顺序构造的设置页。
///
/// 每行是 `变体名: "文案键", "图标"`，后面跟可选的 `, section "分组标题键"`，
/// 最后必须是 `, setting 内容入口`：该函数返回这一页的内容分组
/// （`fn(&mut 分组, &mut Window, &mut Context<分组>) -> Vec<SettingGroup>`）；
/// 页面的标题与图标取自本表，构造器不必再写一遍。
///
/// ```ignore
/// setting_pages! {
///     pub enum SetupRoute for SetupGroup as Route::Setup {
///         Launch: "Setup.Left.Item.Launch", "rocket", setting super::launch::page;
///     }
/// }
/// ```
macro_rules! setting_pages {
    (
        $(#[$meta:meta])*
        $vis:vis enum $route:ident for $group:path as Route::$variant:ident {
            $(
                $name:ident : $title:literal , $icon:literal
                $( , section $section:literal )?
                , setting $setting:path
            );* $(;)?
        }
    ) => {
        $crate::shell::route::page::route_table! {
            $(#[$meta])*
            $vis enum $route as Route::$variant {
                $( $name : $title , $icon $(, section $section)? );*
            }
        }

        impl $group {
            /// 按目录顺序构造设置页（顺序即 gpui-kit `Settings` 的页序）；
            /// 调用方缓存，每帧克隆。
            pub(crate) fn build_setting_pages(
                &mut self,
                window: &mut ::gpui_kit::Window,
                cx: &mut ::gpui_kit::Context<Self>,
            ) -> ::std::vec::Vec<::gpui_kit::component::setting::SettingPage> {
                ::std::vec![
                    $(
                        {
                            let mut page = ::gpui_kit::component::setting::SettingPage::new(
                                $crate::i18n::lang($title),
                            )
                            .icon($crate::components::lucide($icon));
                            for group in $setting(self, window, cx) {
                                page = page.group(group);
                            }
                            page
                        }
                    ),*
                ]
            }
        }
    };
}

/// 测试用：目录 → `(路由, 文案键, 图标, 分组标题)`。
#[cfg(test)]
pub(crate) fn snapshot_rows<R: PageRoute>(
    all: &[PageMeta<R>],
) -> Vec<(R, &'static str, &'static str, Option<&'static str>)> {
    all.iter()
        .map(|meta| (meta.route, meta.title_key, meta.icon, meta.section))
        .collect()
}

pub(crate) use {nav_pages, page_content, route_table, section_key, setting_pages};
