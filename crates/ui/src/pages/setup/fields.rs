//! 设置页的字段构建器：把受控值 + 控件组合成 `SettingItem`。
//!
//! 每个方法读 `SetupState` 上的受控值，写入后请求 [`SetupGroup`] 重绘；控件一律用 gpui-kit 的
//! `Settings` 字段或 PCL 自有的 `AppButton` / `AppCheckBox` / `AppRadio`。

use std::cell::RefCell;
use std::rc::Rc;

use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::setting::{AnySettingField, SettingField, SettingGroup, SettingItem};
use gpui_kit::component::slider::{Slider, SliderState};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::SetupGroup;
use super::state::SetupState;
use crate::components::{AppButton, AppCheckBox, AppRadio, ButtonColor};
use crate::i18n;

/// 受控值的写入闭包：由页面持有状态、字段只请求新值。
pub(super) type BoolSetter = Rc<dyn Fn(&mut SetupState, bool)>;
/// 下标型受控值的写入闭包（下拉框与单选组）：字段只回写下标，状态由页面持有。
pub(super) type IndexSetter = Rc<dyn Fn(&mut SetupState, usize)>;
/// 按钮点击：需要窗口与上下文，交给调用方决定做什么。
pub(super) type SetupAction = Rc<dyn Fn(&mut SetupState, &mut Window, &mut App)>;
/// 「该项当前是否可用」的判断。
pub(super) type EnableCheck = Rc<dyn Fn(&SetupState) -> bool>;
/// [`Fields::slider`] 的 `enabled` 占位：滑块没有启用条件时传入，
/// 免得每个调用点都写 `None::<fn(&SetupState) -> bool>` 的类型标注。
pub(super) const NO_ENABLE: Option<fn(&SetupState) -> bool> = None;

/// 受控值 + 控件的构建器；页面与各字段闭包克隆它共享同一份 [`SetupState`]，
/// 克隆只增加 `Rc` / `WeakEntity` 引用计数。
#[derive(Clone)]
pub(super) struct Fields {
    pub(super) state: Rc<RefCell<SetupState>>,
    pub(super) group: WeakEntity<SetupGroup>,
}

impl Fields {
    /// 读取受控值。
    pub(super) fn read<T>(&self, read: impl FnOnce(&SetupState) -> T) -> T {
        read(&self.state.borrow())
    }

    /// 写入受控值并请求重绘。
    pub(super) fn write(&self, cx: &mut App, write: impl FnOnce(&mut SetupState)) {
        write(&mut self.state.borrow_mut());
        self.notify(cx);
    }

    /// 只请求重绘（值由 gpui-kit 的实体承载时使用，例如滑块/多行文本）。
    pub(super) fn notify(&self, cx: &mut App) {
        self.group.update(cx, |_, cx| cx.notify()).ok();
    }

    /// 复选项（PCL 的 `MyCheckBox`）。
    ///
    /// `Settings` 的复选字段把标签放在右侧，与 PCL 的整行复选框不一致，
    /// 因此用自定义元素承载 PCL 样式的 [`AppCheckBox`]；搜索用标题作为关键词。
    pub(super) fn checkbox(
        &self,
        id: &'static str,
        title_key: &str,
        tip_key: Option<&str>,
        get: impl Fn(&SetupState) -> bool + 'static,
        set: impl Fn(&mut SetupState, bool) + 'static,
    ) -> SettingItem {
        let fields = self.clone();
        let set: BoolSetter = Rc::new(set);
        let title = i18n::lang(title_key);
        let tip = tip_key.map(i18n::lang);
        SettingItem::render(move |_, _, cx| {
            let checked = get(&fields.state.borrow());
            let set = set.clone();
            let fields = fields.clone();
            v_flex()
                .gap_1()
                .w_full()
                .child(
                    AppCheckBox::new(id)
                        .label(title.clone())
                        .checked(checked)
                        .on_change(move |value, _, cx| {
                            fields.write(cx, |state| set(state, *value));
                        }),
                )
                .when_some(tip.clone(), |this, tip| {
                    this.child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(tip),
                    )
                })
        })
        .keywords([i18n::lang(title_key)])
    }

    /// 单行文本框（PCL 的 `MyTextBox`）。
    pub(super) fn input(
        &self,
        title_key: &str,
        tip_key: Option<&str>,
        get: impl Fn(&SetupState) -> SharedString + 'static,
        set: impl Fn(&mut SetupState, SharedString) + 'static,
    ) -> SettingItem {
        let read = self.clone();
        let write = self.clone();
        tip(
            item(
                title_key,
                SettingField::<SharedString>::input(
                    move |_| get(&read.state.borrow()),
                    move |value, cx| write.write(cx, |state| set(state, value)),
                ),
            ),
            tip_key,
        )
    }

    /// 下拉框（PCL 的 `MyComboBox`）。状态里保存选中项下标。
    pub(super) fn dropdown(
        &self,
        title_key: &str,
        tip_key: Option<&str>,
        options: Vec<(SharedString, SharedString)>,
        get: impl Fn(&SetupState) -> usize + 'static,
        set: impl Fn(&mut SetupState, usize) + 'static,
    ) -> SettingItem {
        let read = self.clone();
        let write = self.clone();
        let set: IndexSetter = Rc::new(set);
        tip(
            item(
                title_key,
                SettingField::<SharedString>::dropdown(
                    options,
                    move |_| SharedString::from(get(&read.state.borrow()).to_string()),
                    move |value, cx| {
                        if let Ok(index) = value.parse::<usize>() {
                            let set = set.clone();
                            write.write(cx, |state| set(state, index));
                        }
                    },
                ),
            ),
            tip_key,
        )
    }

    /// 单选行（PCL 的 `MyRadioBox` 横排）。
    pub(super) fn radios(
        &self,
        id: &'static str,
        title_key: &str,
        tip_key: Option<&str>,
        labels: &[&str],
        get: impl Fn(&SetupState) -> usize + 'static,
        set: impl Fn(&mut SetupState, usize) + 'static,
    ) -> SettingItem {
        let fields = self.clone();
        let labels: Vec<SharedString> = labels.iter().map(|key| i18n::lang(key)).collect();
        let set: IndexSetter = Rc::new(set);
        let field = SettingField::<SharedString>::render(move |_, _, _| {
            let selected = get(&fields.state.borrow());
            let mut row = h_flex().gap_4().items_center().flex_wrap();
            for (index, label) in labels.iter().enumerate() {
                let set = set.clone();
                let fields = fields.clone();
                row = row.child(
                    // 元素 id 由固定前缀与下标组成，不由译文派生。
                    AppRadio::new(SharedString::from(format!("{id}-{index}")))
                        .label(label.clone())
                        .selected(selected == index)
                        .on_change(move |_, _, cx| {
                            fields.write(cx, |state| set(state, index));
                        }),
                );
            }
            row
        });
        tip(item(title_key, field), tip_key)
    }

    /// 单选项（PCL 的 `MyRadioBox`，一项一行）。
    pub(super) fn radio_option(
        &self,
        id: &'static str,
        title_key: &str,
        tip_key: Option<&str>,
        index: usize,
        get: impl Fn(&SetupState) -> usize + 'static,
        set: impl Fn(&mut SetupState, usize) + 'static,
    ) -> SettingItem {
        let fields = self.clone();
        let set: IndexSetter = Rc::new(set);
        let title = i18n::lang(title_key);
        let tip = tip_key.map(i18n::lang);
        SettingItem::render(move |_, _, cx| {
            let selected = get(&fields.state.borrow()) == index;
            let set = set.clone();
            let fields = fields.clone();
            v_flex()
                .gap_1()
                .w_full()
                .child(
                    AppRadio::new(id)
                        .label(title.clone())
                        .selected(selected)
                        .on_change(move |_, _, cx| {
                            fields.write(cx, |state| set(state, index));
                        }),
                )
                .when_some(tip.clone(), |this, tip| {
                    this.child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(tip),
                    )
                })
        })
        .keywords([i18n::lang(title_key)])
    }

    /// 滑块（PCL 的 `MySlider`）。
    ///
    /// 值由 `SliderState` 实体承载（拖动滑块时它会通知，本组读到它即会重绘）；
    /// `unit` 是数值后缀（单位，不是界面文案）。PCL 的滑块在拖动时用气泡显示数值，
    /// 这里改成滑块右侧的常驻数值。
    pub(super) fn slider(
        &self,
        title_key: &str,
        tip_key: Option<&str>,
        slider: &Entity<SliderState>,
        unit: &'static str,
        enabled: Option<impl Fn(&SetupState) -> bool + 'static>,
    ) -> SettingItem {
        let fields = self.clone();
        let slider = slider.clone();
        let enabled: Option<EnableCheck> = enabled.map(|check| Rc::new(check) as EnableCheck);
        let field = SettingField::<SharedString>::render(move |_, _, cx| {
            let value = slider.read(cx).value().end();
            let disabled = enabled
                .as_ref()
                .is_some_and(|check| !check(&fields.state.borrow()));
            h_flex()
                .gap_3()
                .items_center()
                .w_full()
                // 设置项右侧的控件槽按内容定宽，这里给滑块一个固定宽度（PCL 的滑块约 200px）。
                .child(
                    div()
                        .w_56()
                        .flex_1()
                        .child(Slider::new(&slider).disabled(disabled)),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(SharedString::from(format!("{value:.0}{unit}"))),
                )
        });
        tip(item(title_key, field), tip_key)
    }

    /// 页面内的按钮（PCL 的 `MyButton`）。
    pub(super) fn button(
        &self,
        id: &'static str,
        label_key: &str,
        color: ButtonColor,
        on_click: impl Fn(&mut SetupState, &mut Window, &mut App) + 'static,
    ) -> SettingItem {
        let fields = self.clone();
        let label = i18n::lang(label_key);
        let on_click: SetupAction = Rc::new(on_click);
        SettingItem::render(move |_, _, _| {
            let fields = fields.clone();
            let on_click = on_click.clone();
            AppButton::new(id, label.clone())
                .color(color)
                .on_click(move |_, window, cx| {
                    on_click(&mut fields.state.borrow_mut(), window, cx);
                    fields.notify(cx);
                })
        })
        .keywords([i18n::lang(label_key)])
    }

    /// 只保留排版、无行为的占位按钮（打开外部网页 / 文件夹、刷新、清理、更换图片等操作尚未接入）。
    ///
    /// `id` 是控件标识（许可类按钮由许可名派生），`label_key` 是按钮文案键，
    /// `tooltip_key` 是可选的悬浮提示文案键。
    pub(super) fn stub_button(
        &self,
        id: impl Into<SharedString>,
        label_key: &str,
        tooltip_key: Option<&str>,
        color: ButtonColor,
    ) -> SettingItem {
        let id: SharedString = id.into();
        let label = i18n::lang(label_key);
        let tooltip = tooltip_key.map(i18n::lang);
        self.element(&[label_key], move |_, _| {
            let mut button = AppButton::new(id.clone(), label.clone()).color(color);
            if let Some(tooltip) = tooltip.clone() {
                button = button.tooltip(tooltip);
            }
            button.into_any_element()
        })
    }

    /// 任意自绘内容（说明文字、横幅、列表等）。
    pub(super) fn element(
        &self,
        keywords: &[&str],
        render: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> SettingItem {
        SettingItem::render(move |_, window, cx| render(window, cx))
            .keywords(keywords.iter().map(|key| i18n::lang(key)))
    }

    /// 黄底提示行（PCL 的 `MyHint Theme="Yellow"`）。
    ///
    /// PCL 的提示色取自 `ColorBrushYellow`；PCL 档位色里没有黄色档，
    /// 这里改用主题浅色填充。
    pub(super) fn hint(&self, text_key: &str) -> SettingItem {
        let text = i18n::lang(text_key);
        SettingItem::render(move |_, _, cx| {
            let palette = crate::theme::palette(cx);
            div()
                .w_full()
                .rounded(px(4.))
                .p_3()
                .text_sm()
                .bg(palette.color_level(7))
                .child(text.clone())
        })
        .keywords([i18n::lang(text_key)])
    }

    /// 段落文字（`Setup.About.*` 的说明与法律条款）。
    pub(super) fn paragraph(&self, text_key: &str, bold: bool) -> SettingItem {
        let text = i18n::lang(text_key);
        SettingItem::render(move |_, _, _| {
            let mut paragraph = div().text_sm().child(text.clone());
            if bold {
                paragraph = paragraph.font_weight(FontWeight::BOLD);
            }
            paragraph
        })
        .keywords([i18n::lang(text_key)])
    }
}

/// 把 i18n 键列表变成下拉项：值为下标字符串，标签为文案。
///
/// `Settings` 的 `SettingField::dropdown` 以字符串保存选中项，这里统一用下标做值，
/// 状态里只存 `usize`。
pub(super) fn choices(keys: &[&str]) -> Vec<(SharedString, SharedString)> {
    keys.iter()
        .enumerate()
        .map(|(index, key)| (SharedString::from(index.to_string()), i18n::lang(key)))
        .collect()
}

/// 带标题的设置项。
pub(super) fn item(title_key: &str, field: impl AnySettingField + 'static) -> SettingItem {
    SettingItem::new(i18n::lang(title_key), field)
}

/// 附加说明（对应 PCL 控件的 `ToolTip.Tip`）。
pub(super) fn tip(item: SettingItem, tip_key: Option<&str>) -> SettingItem {
    match tip_key {
        Some(key) => item.description(i18n::lang(key)),
        None => item,
    }
}

/// 一个设置分组。
pub(super) fn group(title_key: &str, items: Vec<SettingItem>) -> SettingGroup {
    SettingGroup::new()
        .title(i18n::lang(title_key))
        .items(items)
}

/// 构造页面滑块的 [`SliderState`] 实体（值由实体承载，交给 [`Fields::slider`] 展示）。
///
/// `min` / `max` / `step` / `default` 分别是滑块的最小值、最大值、步长与初始值。
pub(super) fn slider_state(
    cx: &mut Context<SetupGroup>,
    min: f32,
    max: f32,
    step: f32,
    default: f32,
) -> Entity<SliderState> {
    cx.new(|_| {
        SliderState::new()
            .min(min)
            .max(max)
            .step(step)
            .default_value(default)
    })
}
