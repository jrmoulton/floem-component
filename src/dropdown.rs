use floem::{
    reactive::{
        create_effect,
        create_rw_signal,
        use_context,
        ReadSignal,
        RwSignal,
        SignalGet,
        SignalUpdate,
        SignalWith,
    },
    style::{Display, JustifyContent, Position, Style, TextOverflow},
    view::View,
    views::*,
};

use crate::{
    components::scrollable_dropdown,
    icons::*,
    style::{BorderWidths, ColorPalette, LightDark, PopOver, ResponsiveColor, WindowClicked},
};

#[derive(Clone)]
pub struct DropDownStyles {
    pub main_box_back: LightDark,
    pub main_box_border: LightDark,
    pub arrow: ResponsiveColor,
    pub button_cont: ResponsiveColor,
    pub drop_box_back: LightDark,
    pub drop_box_border: LightDark,
    pub sel_cont: ResponsiveColor,
    pub scroll_height: Option<f32>,
    pub max_scroll_height: Option<f32>,
}
impl Default for DropDownStyles {
    fn default() -> Self {
        let accent = ResponsiveColor::from_lightdark(
            ColorPalette::ACCENT_LD
                .dark_decrease_cap_light(10)
                .light_increase_cap_light(10),
        )
        .reduce_active_to_hover();
        let arrow = ResponsiveColor::from_lightdark(LightDark::default().reverse());
        Self {
            main_box_back: LightDark::new(ColorPalette::TRANSPARENT, ColorPalette::TRANSPARENT),
            main_box_border: ColorPalette::BORDER_LD,
            button_cont: accent.clone().disable_base(),
            drop_box_back: LightDark::new(ColorPalette::LIGHT2, ColorPalette::DARK1),
            drop_box_border: LightDark::new(ColorPalette::LIGHT3, ColorPalette::DARK3).reverse(),
            sel_cont: accent,
            scroll_height: Some(20. * 10.),
            max_scroll_height: None,
            arrow,
        }
    }
}

#[derive(Clone, Copy)]
enum ArrowSelect {
    TopLeft,
    BottomLeft,
    Right,
}

pub fn dropdown<SAPF, MAPF, V, V2, V3>(
    data: ReadSignal<im::Vector<String>>, name_icon: impl Fn() -> V, main_apperance: MAPF,
    scroll_apperance: SAPF, on_select: impl Fn(String) + 'static + Copy,
    default_val: Option<String>, styles: ReadSignal<DropDownStyles>,
) -> impl View
where
    V: View + 'static,
    V2: View + 'static,
    V3: View + 'static,
    MAPF: (Fn(Box<dyn Fn() -> String>) -> V2) + 'static + Copy,
    SAPF: (Fn(String) -> V3) + 'static + Copy,
{
    let window_clicked = use_context::<WindowClicked>()
        .expect("Expected there to be a global window clicked getter");
    let pop_over =
        use_context::<PopOver>().expect("Expected a PopOver trigger to have been provided");

    let display_scroll = create_rw_signal(false);
    let inner_text_idx = create_rw_signal(0);
    let (box_func, box_size) = crate::style::lazy_size();

    create_effect(move |_| {
        window_clicked.track();
        pop_over.track();
        display_scroll.update(|val| *val = false);
    });

    if let Some(default_val) = default_val {
        inner_text_idx.update(|val| {
            *val = data.with(|list| {
                list.iter()
                    .position(|inner| inner == &default_val)
                    .unwrap_or(*val)
            })
        })
    }
    let (main_box_border_radius_fn, main_box_border_radius) = crate::style::lazy_border_rad();

    container(|| {
        stack(move || {
            (
                // everything but the dropdown
                stack(|| {
                    (
                        // icon
                        name_icon(),
                        // hstack - continuous box w/ buttons
                        stack(|| {
                            (
                                up_down_buttons(inner_text_idx, data, on_select, styles),
                                // Text font size input
                                main_apperance(Box::new(move || {
                                    data.get().get(inner_text_idx.get()).unwrap().to_string()
                                }))
                                .style(move || {
                                    Style::BASE
                                        .padding_left_px(2.0)
                                        .text_overflow(TextOverflow::Ellipsis)
                                }),
                                // Down arrow on far right
                                arrow_and_container(ArrowSelect::Right, styles, move || {
                                    let state = display_scroll.get();
                                    pop_over.notify();
                                    display_scroll.update(|val| *val = !state);
                                }),
                            )
                        })
                        .on_resize(main_box_border_radius_fn)
                        .style(move || {
                            Style::BASE
                                .flex_row()
                                .border(BorderWidths::SM)
                                .border_radius(main_box_border_radius())
                                .items_center()
                                .justify_content(Some(JustifyContent::SpaceBetween))
                                .size_pct(100.0, 100.0)
                                .border_color(styles.with(|val| val.main_box_border.color()))
                                .background(styles.with(|val| val.main_box_back.color()))
                        }),
                    )
                })
                .on_resize(box_func)
                .style(|| Style::BASE.flex_row().items_start().size_pct(100.0, 100.0)),
                scrollable_dropdown(
                    move || data.get(),
                    scroll_apperance,
                    move || styles.get().drop_box_back,
                    move || styles.get().sel_cont,
                    move |item| {
                        inner_text_idx.update(move |val| {
                            *val = data
                                .get()
                                .iter()
                                .position(|inner| inner == &item)
                                .unwrap_or(*val);
                            display_scroll.update(|val| *val = false);
                            on_select(item.clone());
                        });
                    },
                )
                .style(move || {
                    Style::BASE
                        .apply_if(!display_scroll.get(), |s| {
                            s.display(Display::None).height_px(0.).width_px(0.)
                        })
                        .z_index(1)
                        .position(Position::Absolute)
                        .width_pct(70.)
                        .apply_opt(styles.get().scroll_height, |style, val| {
                            style.height_px(val)
                        })
                        .apply_opt(styles.get().max_scroll_height, |style, val| {
                            style.max_height_px(val)
                        })
                        .inset_top_px(box_size().height as f32 + 3.)
                        .border(2.)
                        .border_color(styles.with(|val| val.drop_box_border.color()))
                        .border_radius(main_box_border_radius())
                }),
            )
        })
        .style(|| Style::BASE.flex_col().items_center().size_pct(100.0, 100.0))
    })
}

fn arrow_and_container(
    selector: ArrowSelect, styles: ReadSignal<DropDownStyles>, on_click: impl Fn() + 'static,
) -> impl View {
    let active = create_rw_signal(false);
    container(move || {
        match selector {
            ArrowSelect::TopLeft => icon_chevron_up(),
            ArrowSelect::BottomLeft | ArrowSelect::Right => icon_chevron_down(),
        }
        .style(move || DPStyles::button_style(styles, active))
        .hover_style(move || {
            DPStyles::button_style(styles, active).color(styles.with(|val| val.arrow.hover.color()))
        })
        .active_style(move || {
            DPStyles::button_style(styles, active)
                .color(styles.with(|val| val.arrow.active.color()))
        })
    })
    .style(move || {
        DPStyles::button_cont_style(selector)
            .background(styles.with(|val| val.button_cont.base.color()))
    })
    .hover_style(move || {
        DPStyles::button_cont_style(selector)
            .background(styles.with(|val| val.button_cont.hover.color()))
    })
    .active_style(move || {
        DPStyles::button_cont_style(selector)
            .background(styles.with(|val| val.button_cont.active.color()))
    })
    .on_event(floem::event::EventListener::PointerDown, move |_| {
        active.update(|val| *val = true);
        true
    })
    .on_event(floem::event::EventListener::PointerUp, move |_| {
        active.update(|val| *val = false);
        on_click();
        true
    })
}

fn increment_and_select(
    selector: ArrowSelect, inner_text_idx: RwSignal<usize>, data: ReadSignal<im::Vector<String>>,
    on_select: impl Fn(String) + 'static + Copy,
) -> bool {
    inner_text_idx.update(|val| {
        let new_val = if let ArrowSelect::TopLeft = selector {
            val.checked_sub(1).unwrap_or(*val)
        } else {
            *val + 1
        };
        if new_val < data.get().len() {
            *val = new_val;
        }
        on_select(data.get().get(*val).unwrap().to_string());
    });
    true
}

fn up_down_buttons(
    inner_text_idx: RwSignal<usize>, data: ReadSignal<im::Vector<String>>,
    on_select: impl Fn(String) + 'static + Copy, styles: ReadSignal<DropDownStyles>,
) -> impl View {
    stack(|| {
        (
            // First Button
            arrow_and_container(ArrowSelect::TopLeft, styles, move || {
                increment_and_select(ArrowSelect::TopLeft, inner_text_idx, data, on_select);
            }),
            // Secong Button
            arrow_and_container(ArrowSelect::BottomLeft, styles, move || {
                increment_and_select(ArrowSelect::BottomLeft, inner_text_idx, data, on_select);
            }),
        )
    })
    .style(|| Style::BASE.flex_col().items_center())
}

pub struct DPStyles {}
impl DPStyles {
    const ICW: f32 = 15.0;
    pub const SCROLL_CONTAINER_PADDING: f32 = 6.;

    fn button_style(styles: ReadSignal<DropDownStyles>, _active: RwSignal<bool>) -> Style {
        let color = styles.with(|val| val.arrow.base.color());
        Style::BASE.size_px(Self::ICW, Self::ICW).color(color)
        // .apply_if(active.get(), |s| s.inset_top_px(0.2))
        // .apply_if(!active.get(), |s| s.inset_top(LengthPercentageAuto::Auto))
    }

    fn button_cont_style(selector: ArrowSelect) -> Style {
        let horiz_padding = 5.0;
        Style::BASE
            .height_pct(100.0)
            .width_px(Self::ICW + horiz_padding * 2.)
            .items_center()
            .justify_center()
            .padding_horiz_px(horiz_padding)
            .padding_top_px(match selector {
                ArrowSelect::TopLeft => 1.,
                _ => 0.,
            })
            .padding_bottom_px(match selector {
                ArrowSelect::BottomLeft => 1.,
                _ => 0.,
            })
    }
}
