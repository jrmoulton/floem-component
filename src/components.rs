use std::{ops::RangeInclusive, sync::atomic::AtomicU16};

use floem::{
    event::Event,
    reactive::{create_rw_signal, create_signal, ReadSignal, RwSignal},
    responsive::ScreenSize,
    style::{CursorStyle, Display, Position, Style, TextOverflow},
    view::View,
    views::*,
};

use crate::{
    dropdown::{self, DPStyles},
    icons::*,
    style::{
        self,
        dropdown_cont,
        follow_popover,
        scroll_bar_color,
        BorderWidths,
        ColorPalette,
        EventHandlers,
        ExtAnyEvent,
        ExtDynamicStyle,
        LightDark,
        ResponsiveColor,
    },
};

#[macro_export]
macro_rules! stack {
    ($($expr:expr),*) => {
        floem::views::stack(move || ($($expr),*))
    };
}

/// SAPF - A function that takes a String and returns a View. It is 'static and
/// copyable. V - A View type, which is 'static.
/// OCF - A function that takes a String and is 'static and copyable.
pub fn scrollable_dropdown<
    SV,
    V,
    OCF,
    D: IntoIterator<Item = T>,
    T: 'static + Clone,
    DF: Fn() -> D + 'static,
>(
    data: DF,
    scroll_view: SV, // Scroll appearance function
    background_color: impl Fn() -> LightDark + 'static + Copy,
    selection_container_color: impl Fn() -> ResponsiveColor + 'static + Copy,
    on_click: OCF, // On click function
) -> impl View
where
    SV: Fn(T) -> V + 'static + Copy,
    V: View + 'static,
    OCF: Fn(T) + 'static + Copy,
{
    // Style closure for list
    let list_style = move || {
        Style::BASE
            .background(background_color().color())
            .flex_col()
            .width_pct(100.)
            .justify_center()
    };

    let scroll_container_style = || {
        Style::BASE
            .padding_left_px(DPStyles::SCROLL_CONTAINER_PADDING)
            .padding_vert_px(1.)
            .width_pct(100.)
            .items_center()
    };

    // Hover style closure for container
    let hover_style = move || {
        let color = selection_container_color().hover;
        scroll_container_style()
            .background(color.color())
            .color(color.fg_color().color())
    };

    let count = AtomicU16::new(0);
    // Creating a scroll container
    container(
        move || {
            scroll(move || {
                // Creating a list from data
                list(
                    data,
                    move |_| count.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                    move |item| {
                        let item_clone = item.clone();

                        // Creating a container for each item with the scroll appearance and setting
                        // its style and hover style
                        container(move || scroll_view(item.clone()))
                            .on_click(move |_| {
                                on_click(item_clone.clone());
                                true
                            })
                            .style(scroll_container_style)
                            .hover_style(hover_style)
                    },
                )
                .style(list_style) // Setting style for the list
            })
            .style(|| Style::BASE.width_pct(100.))
            .scroll_bar_color(scroll_bar_color)
        }, // Setting scroll bar color
    )
}

pub fn pop_over<V: View + 'static>(child: impl FnOnce() -> V + 'static) -> impl View {
    stack(|| {
        (
            icon_caret_up().style(|| Style::BASE.position(Position::Absolute).size_px(15., 15.)),
            container(child).style(|| {
                Style::BASE
                    .position(Position::Absolute)
                    .inset_top_px(15.)
                    .color(ColorPalette::BORDER_LD.reverse().color())
            }),
        )
    })
}

pub fn freq_dropdown(
    data: ReadSignal<im::Vector<String>>, on_select: impl Fn(String) + Copy + 'static,
) -> impl View {
    let dropdown_styles = dropdown::DropDownStyles::default();
    dropdown::dropdown(
        data,
        empty,
        |data_val| label(move || format!("Freq: {} Hz", data_val())),
        |data_val| label(move || format!("{data_val} Hz")),
        on_select,
        None,
        create_signal(dropdown_styles).0,
    )
    .style(dropdown_cont)
}

/// Takes a function that takes a boolean that is set to true when active and
/// false when inactive
pub fn toggle_button(
    initial_state: bool, action: impl Fn(bool) + 'static,
    toggle_height: impl Fn() -> f32 + 'static + Copy,
) -> impl View {
    let toggle_active = create_rw_signal(initial_state);
    let (size_fn, toggle_size) = style::lazy_size();

    let toggle_color = move || {
        if toggle_active.get() {
            ResponsiveColor::from_lightdark(ColorPalette::ACCENT_LD)
        } else {
            ResponsiveColor::from_lightdark(LightDark::new(
                ColorPalette::LIGHT3,
                ColorPalette::LIGHT4,
            ))
        }
    };
    let toggle_style = move || {
        let height = toggle_height();
        Style::BASE
            // the height should be 42 percent of the width
            .size_px(height * 2.333, height)
    };
    stack(|| {
        (
            icon_toggle_oval()
                .style(move || toggle_style().color(toggle_color().base.color()))
                .hover_style(move || toggle_style().color(toggle_color().hover.color()))
                .on_resize(size_fn),
            empty().style(move || {
                let size = toggle_height() * 0.90;
                let inset = toggle_height() * 0.05;
                Style::BASE
                    .position(Position::Absolute)
                    .inset_top_px(toggle_height() / 2. - size / 2.)
                    .inset_left_px(if toggle_active.get() {
                        toggle_size().width as f32 - inset - size
                    } else {
                        inset
                    })
                    .size_px(size, size)
                    .border_radius(size)
                    .background(ColorPalette::LIGHT1.color())
            }),
        )
    })
    .on_click(move |_| {
        toggle_active.update(|val| *val = !*val);
        action(toggle_active.get());
        true
    })
    .hover_style(|| Style::BASE.cursor(CursorStyle::Pointer))
}

pub fn slider<RF: Fn() -> RangeInclusive<f32> + 'static + Copy>(
    actual_value: RwSignal<f32>, range: RF,
) -> impl View {
    let range_diff = move || range().end() - range().start();
    let percent_over = move || (actual_value.get() - range().start()) / range_diff();
    let (bar_size_fn, bar_size) = style::lazy_size();
    let (background_size_fn, background_size) = style::lazy_size();
    let px_over = move || bar_size().width as f32 * percent_over();

    let background =
        ResponsiveColor::from_lightdark(LightDark::new(ColorPalette::LIGHT3, ColorPalette::LIGHT4));
    let (br_fn, br) = style::lazy_border_rad();
    let bar = move || {
        stack(|| {
            (
                empty()
                    .on_resize(bar_size_fn)
                    .style(|| {
                        Style::BASE
                            .border(BorderWidths::SM)
                            .width_pct(100.)
                            .border_color(ColorPalette::BORDER_LD.color())
                            // just make sure it's big
                            .border_radius(10.)
                    })
                    .on_event(
                        floem::event::EventListener::PointerDown,
                        move |pointer_event| {
                            if let Event::PointerDown(pointer_event) = pointer_event {
                                actual_value.update(|val| {
                                    *val = (pointer_event.pos.x / bar_size().width) as f32
                                        * range_diff()
                                        - range().start()
                                });
                            }
                            true
                        },
                    ),
                empty().style(move || {
                    Style::BASE
                        .position(Position::Absolute)
                        .border(BorderWidths::SM)
                        // This is a bug in taffy I'm pretty sure (it doesn't take into account the right padding but it does take into account left padding)
                        .width_px(px_over())
                        .border_color(ColorPalette::ACCENT_LD.color())
                        // just make sure it's big
                        .border_radius(10.)
                }),
            )
        })
        .style(move || {
            Style::BASE
                .width_pct(100.)
                .padding_px(10.)
                .border_radius(br())
                .background(background.base.color())
        })
        .on_resize(br_fn)
    };
    let controller = move || {
        let size = 15.;
        let initial_mouse_pos = create_rw_signal(None);

        empty()
            .style(move || {
                Style::BASE
                    .size_px(size, size)
                    .position(Position::Absolute)
                    .inset_top_px(background_size().height as f32 / 2. - size / 2.)
                    .background(ColorPalette::LIGHT1.color())
                    .border_radius(size)
                    .inset_left_px(px_over())
            })
            .on_event(floem::event::EventListener::PointerDown, move |event| {
                initial_mouse_pos.update(|val| *val = event.point());
                true
            })
            .on_event(floem::event::EventListener::PointerUp, move |_| {
                initial_mouse_pos.update(|val| *val = None);
                true
            })
            .on_event(floem::event::EventListener::PointerMove, move |event| {
                if let Some(point) = initial_mouse_pos.get() {
                    let delta = event.point().unwrap() - point;
                    actual_value.update(|val| {
                        let new = *val
                            + (delta.x / bar_size().width) as f32
                                * (range().end() - range().start());
                        if new > *range().start() && new < *range().end() {
                            *val = new;
                        }
                    });
                    return true;
                }
                false
            })
            .keyboard_navigatable()
    };
    stack(|| (bar(), controller()))
        .on_resize(background_size_fn)
        .style(|| Style::BASE.width_pct(100.))
}

pub type DisplayEdit = bool;

#[allow(clippy::too_many_arguments)]
pub fn label_with_edit_dropdown(
    display_name: impl Fn() -> String + 'static, text_edit_signal: RwSignal<String>,
    label_style: impl Fn() -> Style + Copy + 'static,
    text_edit_style: impl Fn() -> Style + Copy + 'static,
    label_responsive_color: impl Fn() -> ResponsiveColor + Copy + 'static,
    label_handlers: impl Fn(RwSignal<DisplayEdit>) -> EventHandlers + 'static,
    on_edit_accept: impl Fn() -> bool + 'static,
) -> impl View {
    let (edit_br_fn, edit_br) = style::lazy_border_rad();
    let (main_device_label_size_func, main_device_label_size) = style::lazy_size();
    let display_edit = create_rw_signal(false);
    follow_popover(display_edit);
    container(|| {
        stack(move || {
            (
                // Main device label
                hover_background(
                    move || label(display_name).dynamic_style(label_style, label_responsive_color),
                    || ColorPalette::POPOVER_BG.base.color(),
                )
                .on_resize(main_device_label_size_func)
                .all_events(label_handlers(display_edit).handlers),
                // The view to rename the device
                text_edit(text_edit_signal, move || {
                    display_edit.update(|val| *val = false);
                    on_edit_accept()
                })
                .on_resize(edit_br_fn)
                .style(move || {
                    text_edit_style()
                        .border_radius(edit_br())
                        .position(Position::Absolute)
                        .inset_top_px(main_device_label_size().height as f32)
                        .apply_if(!display_edit.get(), |style| style.display(Display::None))
                }),
            )
        })
        .style(|| Style::BASE.flex_col().items_center())
    })
}

pub fn hover_background<V: View>(
    child: impl FnOnce() -> V + 'static,
    hover_backgound_color: impl Fn() -> floem::peniko::Color + 'static,
) -> impl View {
    let (size_fn, _size, border_rad) = style::lazy_size_and_rad();
    // let padding_percent = 0.1;
    let container_style = move || {
        Style::BASE
            .padding_horiz_px(8.)
            .padding_vert_px(3.)
            .border_radius(border_rad())
            .items_center()
            .justify_center()
    };
    container(|| {
        container(child)
            .on_resize(size_fn)
            .style(container_style)
            .hover_style(move || {
                container_style()
                    .background(hover_backgound_color())
                    .cursor(CursorStyle::Pointer)
            })
    })
}

pub fn text_edit(text: RwSignal<String>, on_accept: impl Fn() -> bool + 'static) -> impl View {
    let (br_fn, br) = style::lazy_border_rad();
    container(|| {
        container(|| {
            text_input(text).style(|| Style::BASE.cursor_color(ColorPalette::CURSOR_LD.color()))
        })
        .on_resize(br_fn)
        .style(move || {
            Style::BASE
                .border_radius(br())
                .padding_px(5.)
                .items_center()
                .justify_center()
        })
        .on_click(|_| {
            // This random true is here to stop the click from propagating down. I think
            // this is a bug in floem
            true
        })
        .on_event(floem::event::EventListener::KeyDown, move |event| {
            if let Event::KeyDown(key_event) = event {
                if key_event.key == floem::glazier::KbKey::Enter {
                    on_accept()
                } else {
                    false
                }
            } else {
                false
            }
        })
        .on_event(floem::event::EventListener::PointerMove, |_| true)
        .on_event(floem::event::EventListener::PointerDown, |_| true)
        .on_event(floem::event::EventListener::PointerUp, |_| true)
    })
}

pub fn setting<VF: Fn() -> V + 'static, V: View + 'static, SF: Fn() -> String + 'static>(
    name: SF, view_fn: VF,
) -> impl View {
    // The base style to be applied on all screen sizes
    let base_style = || Style::BASE.width_pct(100.).justify_between();

    stack(|| {
        (
            label(name).style(|| Style::BASE.text_overflow(TextOverflow::Wrap)),
            view_fn().responsive_style(ScreenSize::XS, || {
                Style::BASE.align_self(Some(floem::style::AlignItems::End))
            }),
        )
    })
    .style(base_style)
    .responsive_style(ScreenSize::XS, move || base_style().flex_col())
}

#[derive(Clone, Copy)]
pub enum LeftRight {
    Left,
    Right,
}

/// Generate a container box that dynamically switches between two views based
/// on a click event.
///
/// This function creates a flexible container box that can toggle between two
/// views based on the result of the provided click function. It is designed to
/// work with types that implement the `View` trait, allowing it to accommodate
/// different types of views.
///
/// # Parameters
///
/// - `click`: A closure function that takes no arguments and returns a value of
///   type `LeftRight`. This function determines which view should be displayed:
///   `LeftRight::Left` or `LeftRight::Right`.
///
/// - `left`: A closure function that takes no arguments and returns a value of
///   a type that implements the trait `View`. This closure is used to generate
///   the left view that will be displayed when `click` returns
///   `LeftRight::Left`.
///
/// - `right`: A closure function that takes no arguments and returns a value of
///   a type that implements the trait `View`. This closure is used to generate
///   the right view that will be displayed when `click` returns
///   `LeftRight::Right`.
///
/// # Generics
///
/// This function uses generics to maintain flexibility and compatibility with
/// different types. The following generics are used:
///
/// - `LV`: A generic type parameter that represents the left view. It is
///   required to implement the trait `View` and must have a `'static` lifetime.
///
/// - `LVF`: A generic type parameter that represents the closure function for
///   generating the left view. It takes no arguments and returns a value of
///   type `LV`. This closure must implement the `Copy` trait and have a
///   `'static` lifetime.
///
/// - `RV`: A generic type parameter that represents the right view. It is
///   required to implement the trait `View` and must have a `'static` lifetime.
///
/// - `RVF`: A generic type parameter that represents the closure function for
///   generating the right view. It takes no arguments and returns a value of
///   type `RV`. This closure must implement the `Copy` trait and have a
///   `'static` lifetime.
///
/// - `CF`: A generic type parameter that represents the closure function for
///   handling the click event. It takes no arguments and returns a value of
///   type `LeftRight`. This function must implement the `Copy` trait and have a
///   `'static` lifetime.
///
/// # Return
///
/// The function returns a type that implements the `View` trait. The returned
/// view switches between the left and right views based on the result of the
/// `click` function.
///
/// # Examples
///
/// ```rust
/// // An enumeration representing left and right views.
/// enum LeftRight {
///     Left,
///     Right,
/// }
///
/// // Define a function that determines which view to display based on a signal.
/// fn get_signal_value() -> LeftRight {
///     // Your custom logic to determine the view (Left or Right) goes here.
///     // For demonstration purposes, we'll return Left in this example.
///     LeftRight::Left
/// }
///
/// // Define a function that generates the left view based on a signal.
/// fn create_left_view() -> impl View {
///     // Your implementation of creating the left view goes here.
///     // For demonstration purposes, we'll return a placeholder view.
///     PlaceholderLeftView
/// }
///
/// // Define a function that generates the right view based on a signal.
/// fn create_right_view() -> impl View {
///     // Your implementation of creating the right view goes here.
///     // For demonstration purposes, we'll return another placeholder view.
///     PlaceholderRightView
/// }
///
/// // Use the `left_right` function to create a container box that toggles between the views based on signals.
/// let container = left_right(get_signal_value, create_left_view, create_right_view);
/// ```
pub fn left_right<
    LV: View + 'static,
    LVF: Fn() -> LV + Copy + 'static,
    RV: View + 'static,
    RVF: Fn() -> RV + Copy + 'static,
    CF: Fn() -> LeftRight + Copy + 'static,
>(
    click: CF, left: LVF, right: RVF,
) -> impl View {
    container_box(move || match click() {
        LeftRight::Left => Box::new(left()),
        LeftRight::Right => Box::new(right()),
    })
}
