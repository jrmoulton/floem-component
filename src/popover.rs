use floem::{
    peniko::kurbo::Size,
    reactive::{create_effect, create_rw_signal, use_context, SignalGet, SignalUpdate},
    style::{Display, Position, Style},
};

use crate::style::{border_radius, ColorPalette, PopOver};

/// A function to provide components for building a popover.
///
/// This function returns a style for popovers and a function that can be
/// plugged into an event handler to handle popover state.
///
/// # Arguments
///
/// * `toggle_event`: A closure that returns an event listener representing the
///   event that triggers the popover toggle. The returned event listener should
///   be unique for this particular popover.
/// * `parent_size`: A closure that returns the size of the parent element to
///   which the popover is attached. The returned size should be used to
///   position the popover relative to its parent.
///
/// # Returns
///
/// A tuple containing two closures:
/// 1. A closure (`style`) that returns a `Style` object representing the visual
/// style of the popover. The style includes attributes such as background
/// color, border, and positioning. 2. A closure (`toggle_function`) that takes
/// a reference to a `floem::event::Event` and handles the popover's display
/// state. When this function is plugged into an event handler, it will toggle
/// the visibility of the popover based on the provided `toggle_event`.
///
/// # Example
///
/// ```rust
/// use some_event_library::EventListener;
///
/// use crate::lazy_popover;
///
/// // Define the toggle event for the popover
/// fn my_toggle_event() -> EventListener {
///     // Code to create and return the event listener
///     unimplemented!()
/// }
///
/// // Define the parent size function
/// fn my_parent_size() -> Size {
///     // Code to calculate and return the parent size
///     unimplemented!()
/// }
///
/// // Get the style and toggle function for the popover
/// let (popover_style, popover_toggle) = lazy_popover(my_toggle_event, my_parent_size);
///
/// // Use the popover_style closure to get the style and apply it to the popover view
/// let popover_view = View::new().style(popover_style);
///
/// // In your event handler, use the popover_toggle closure to handle the popover state
/// // For example, in a click event handler:
/// let event = some_event_library::get_event();
/// if popover_toggle(&event) {
///     // Handle popover visibility change if the toggle event occurred
/// }
/// ```
pub fn lazy_popover<
    TF: Fn() -> floem::event::EventListener + 'static,
    PSF: Fn() -> Size + 'static,
>(
    toggle_event: TF, parent_size: PSF,
) -> (
    impl Fn() -> Style + 'static,
    impl Fn(&floem::event::Event) -> bool,
) {
    let pop_over = use_context::<PopOver>().unwrap();
    let display = create_rw_signal(false);

    let style = move || {
        Style::BASE
            .apply_if(!display.get(), |s| {
                s.display(Display::None).height_px(0.).width_px(0.)
            })
            .width_pct(100.)
            .z_index(1)
            .padding_px(3.)
            .background(ColorPalette::DROPDOWN_BG.color())
            .position(Position::Absolute)
            .max_height_px(20. * 10.)
            .inset_top_px(parent_size().height as f32 + 3.)
            .border(2.)
            .border_color(ColorPalette::BORDER_LD.color())
            .border_radius(border_radius(parent_size().to_rect()))
    };

    create_effect(move |_| {
        pop_over.track();
        display.update(|val| *val = false);
    });
    let toggle_function = move |event: &floem::event::Event| {
        if let Some(listener) = event.listener() {
            if listener.eq(&toggle_event()) {
                let state = display.get();
                pop_over.notify();
                display.update(|val| *val = !state);
                true
            } else {
                false
            }
        } else {
            false
        }
    };
    (style, toggle_function)
}
