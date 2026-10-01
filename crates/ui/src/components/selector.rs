//! 选择栏（对应 `PCL.Ui/Controls/MyListItem.axaml` 与 `Views/Shared/PageSelectorBuilder.cs`）。
//!
//! PCL 的选择栏条目是 36px 高的图标 + 文案行，悬停与选中都用同一块填充（ColorBrush7 底 +
//! ColorBrush6 边），选中时左侧多一条 20px 高的主题色短条、文字转为较深的主题色。
//! 这里按同一套视觉实现，交互（点击、悬停、禁用、右侧动作按钮）用 gpui-kit 的按钮完成。

use gpui_kit::base::{InteractiveElementExt as _, TestSupportExt as _, h_flex, v_flex};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use std::rc::Rc;

use super::icon_button::IconButton;
use super::lucide;
use crate::theme;

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// 选择栏分组标题（PCL 的 `SelectorSectionKey`）。
#[derive(IntoElement)]
pub struct SelectorSection {
    title: SharedString,
    top_gap: Pixels,
}

impl SelectorSection {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            top_gap: px(0.),
        }
    }

    /// 分组与上一组之间的间距（PCL 的 `SelectorSectionTopMargin`，第二个分组起为 10）。
    pub fn top_gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.top_gap = gap.into();
        self
    }
}

impl RenderOnce for SelectorSection {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .px_3()
            .pt(self.top_gap + px(10.))
            .pb_1()
            .text_xs()
            .text_color(theme::palette(cx).gray_level(2))
            .opacity(0.6)
            .child(self.title)
    }
}

/// 选择栏条目。
#[derive(IntoElement)]
pub struct SelectorItem {
    id: ElementId,
    label: SharedString,
    /// 副文本（PCL 的 `MyListItem.Info`）：条目第二行，12px 灰色。
    info: Option<SharedString>,
    icon: Option<SharedString>,
    selected: bool,
    disabled: bool,
    action: Option<(SharedString, SharedString, ClickHandler)>,
    on_click: Option<ClickHandler>,
    on_double_click: Option<ClickHandler>,
}

impl SelectorItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            info: None,
            icon: None,
            selected: false,
            disabled: false,
            action: None,
            on_click: None,
            on_double_click: None,
        }
    }

    /// 条目副文本：与 PCL 的 `MyListItem.Info` 一致，显示在标题下方。
    pub fn info(mut self, info: impl Into<SharedString>) -> Self {
        self.info = Some(info.into());
        self
    }

    /// lucide 图标名（与 .NET 版本的 `lucide/xxx` 对应）。
    pub fn icon(mut self, name: &'static str) -> Self {
        self.icon = Some(SharedString::from(name));
        self
    }

    /// 受控选中态：填充 + 左侧主题色短条 + 更深的前景色。
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 条目右侧的图标动作（PCL 的刷新 / 初始化按钮）。
    pub fn action(
        mut self,
        icon: &'static str,
        tooltip: impl Into<SharedString>,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some((SharedString::from(icon), tooltip.into(), Rc::new(handler)));
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// 双击回调（GPUI 每次点击都会走 `on_click`，`click_count == 2` 时额外走这里）。
    pub fn on_double_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_double_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for SelectorItem {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let palette = theme::palette(cx);
        let selected = self.selected;
        let disabled = self.disabled;
        let on_click = self.on_click.clone();
        let on_double_click = self.on_double_click.clone();
        let action = self.action.clone();
        // 动作按钮的稳定标识：由条目标识派生，不依赖译文。
        let item_id = SharedString::from(format!("{}-action", self.id));

        let mut item = h_flex().id(self.id);
        if !disabled {
            // 点击处理要在 `test_support()` 之前注册：它返回观测包装类型，双击只在
            // `Stateful` 上提供。
            item = item
                .when_some(on_double_click, |this, handler| {
                    this.on_double_click(move |event, window, cx| handler(event, window, cx))
                })
                .when_some(on_click, |this, handler| {
                    this.on_click(move |event, window, cx| handler(event, window, cx))
                });
        }
        let mut item = item
            .test_support()
            // 选择栏条目在可访问性树里就是一组可选项（与 gpui-kit 边栏条目一致）。
            .role(Role::TreeItem)
            .aria_label(self.label.clone())
            .aria_selected(selected)
            // 36px 为 PCL 列表项常规高度；带副文本时按内容增高。
            .min_h(px(36.))
            .py_1()
            .mx_3()
            .px_2()
            .gap_2()
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(gpui::transparent_white())
            .text_base()
            .text_color(palette.gray_level(1))
            .child(
                // 选中条：与 PCL 的 CheckedBar 一致，短条居中于条目高度内。
                div()
                    .w(px(2.))
                    .h(px(20.))
                    .flex_shrink_0()
                    .rounded_full()
                    .bg(if selected {
                        palette.color_level(3)
                    } else {
                        gpui::transparent_white()
                    }),
            )
            .child(
                // 固定图标槽：图标宽度不同也不会带动文案位移。
                div()
                    .w(px(28.))
                    .flex_shrink_0()
                    .flex()
                    .justify_center()
                    .child(
                        self.icon
                            .as_ref()
                            // PCL 的列表项图标与文字同色（未选中取正文色、选中取主题色）。
                            .map(|name| {
                                lucide(name).size_4().text_color(if selected {
                                    palette.color_level(3)
                                } else {
                                    palette.gray_level(1)
                                })
                            })
                            .unwrap_or_default(),
                    ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap_0p5()
                    .child(
                        div()
                            .truncate()
                            .when(selected, |this| {
                                this.text_color(palette.color_level(2))
                                    .font_weight(FontWeight::MEDIUM)
                            })
                            .child(self.label.clone()),
                    )
                    .when_some(self.info.clone(), |this, info| {
                        this.child(
                            div()
                                .truncate()
                                .text_size(px(12.))
                                .text_color(palette.gray_level(2))
                                .child(info),
                        )
                    }),
            );

        if let Some((icon, tooltip, handler)) = action {
            item = item.child(
                IconButton::new(item_id.clone(), icon, tooltip.clone())
                    // PCL 的列表项按钮为 25px，图标名由目录给出。
                    .size(px(25.))
                    .tooltip(tooltip)
                    .on_click(move |event, window, cx| handler(event, window, cx)),
            );
        }

        if !disabled {
            item = item
                // 悬停与选中共用同一块填充，与 PCL 的 RectBack 一致。
                .hover(|this| {
                    this.bg(palette.color_level(7))
                        .border_color(palette.color_level(6))
                })
                .active(|this| {
                    this.bg(palette.color_level(6))
                        .border_color(palette.color_level(5))
                })
                .when(selected, |this| {
                    this.bg(palette.color_level(7))
                        .border_color(palette.color_level(6))
                });
        } else {
            item = item.text_color(palette.gray_level(4));
        }

        item
    }
}

/// 选择栏容器：整列可滚动，条目之间不留间距（与 PCL 的列表一致）。
#[derive(IntoElement)]
pub struct Selector {
    id: ElementId,
    width: Pixels,
    top_inset: Pixels,
    children: Vec<AnyElement>,
}

impl Selector {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            width: px(255.),
            top_inset: px(12.),
            children: Vec::new(),
        }
    }

    /// 选择栏宽度；PCL 中下载 / 设置 / 工具为内容宽度，实例与启动为 300。
    pub fn width(mut self, width: impl Into<Pixels>) -> Self {
        self.width = width.into();
        self
    }

    pub fn top_inset(mut self, inset: impl Into<Pixels>) -> Self {
        self.top_inset = inset.into();
        self
    }

    /// 追加分组标题（PCL 的选择栏分组）。
    pub fn section(mut self, title: impl Into<SharedString>, top_gap: impl Into<Pixels>) -> Self {
        self.children.push(
            SelectorSection::new(title)
                .top_gap(top_gap)
                .into_any_element(),
        );
        self
    }
}

impl ParentElement for Selector {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Selector {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        v_flex()
            .id(self.id)
            .h_full()
            .w(self.width)
            .flex_shrink_0()
            .min_h_0()
            .overflow_y_scrollbar()
            .child(
                v_flex()
                    .w_full()
                    .pt(self.top_inset)
                    .pb_4()
                    .children(self.children),
            )
    }
}
