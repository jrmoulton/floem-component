use std::{marker::PhantomData, ops::Deref};

use colorsys::ColorAlpha;
use floem::{
    event::{Event, EventListener},
    kurbo,
    peniko::{self, Color},
    reactive::{create_effect, create_rw_signal, use_context, RwSignal, Trigger},
    style::{Style, StyleProp, Transition},
    view::View,
    views::Decorators,
};
use paste::paste;

macro_rules! generate_hsl_methods {
    ($($field:ident),*) => {
        $(
            paste! {
                #[doc = "Add a percentage to the `" $field "` value.\n"]
                #[doc = "The maximum value of this will be 100%, and adding more will still yield 100%.\n"]
                 pub const fn [<increase_cap_ $field>](mut self, percent: u8) -> Self {
                    self.$field = if self.$field + percent > 100 {
                        100
                    } else {
                        self.$field + percent
                    };
                    self
                }

                #[doc = "Subtract a percentage from the `" $field "` value.\n"]
                #[doc = "The minimum value of this will be 0%, and subtracting more will still yield 0%.\n"]
                pub const fn [<decrease_cap_ $field>](mut self, percent: u8) -> Self {
                    self.$field = self.$field.saturating_sub(percent);
                    self
                }

                #[doc = "Add a percentage to the `" $field "` value.\n"]
                #[doc = "The value will be capped at 100%, and after reaching 100%, it will wrap around to 0%.\n"]
                pub const fn [<increase_cycle_ $field>](mut self, percent: u8) -> Self {
                    self.$field = self.$field.wrapping_add(percent).rem_euclid(101);
                    self
                }

                #[doc = "Subtract a percentage from the `" $field "` value.\n"]
                #[doc = "The value will be capped at 0%, and after reaching 0%, it will wrap around to 100%.\n"]
                pub const fn [<decrease_cycle_ $field>](mut self, percent: u8) -> Self {
                    self.$field = self.$field.wrapping_sub(percent).rem_euclid(101);
                    self
                }
            }
        )*
    };
}

#[derive(Clone, Copy, Default, Debug)]
pub struct HSLColor {
    hue: u8,
    sat: u8,
    light: u8,
    alpha: u8,
}
impl HSLColor {
    generate_hsl_methods!(light, alpha, sat);

    /// All percentages out of 100
    /// hue: 0-100
    /// sat: 0-100
    /// light: 0-100
    /// alpha: 0-100
    pub const fn new(hue: u8, sat: u8, light: u8, alpha: u8) -> Self {
        Self {
            hue,
            sat,
            light,
            alpha,
        }
    }

    /// increse the hue py a percentage value. The value will be capped at 100%.
    /// This is a continuous value and after 100% it will wrap around to 0%
    pub const fn increase_hue(mut self, percent: u8) -> Self {
        self.hue = self.hue.wrapping_add(percent).rem_euclid(101);
        self
    }

    /// Increase the hue by a percentage value. The value will be capped at
    /// 100%. This is a continuous value, and after 100% it will wrap around
    /// to 0%.
    pub const fn decrease_hue(mut self, percent: u8) -> Self {
        self.hue = self.hue.wrapping_sub(percent).rem_euclid(101);
        self
    }

    pub const fn max_contrast(mut self) -> Self {
        // Calculate the opposite hue (180 degrees away)
        self.increase_hue(50);

        // Calculate the opposite lightness (relative to the middle lightness value)
        let middle_lightness: u8 = 50; // Assuming a middle lightness value of 50%
        self.light = middle_lightness
            .wrapping_sub(self.light)
            .wrapping_add(middle_lightness);

        self
    }

    pub const fn desat_opposite(self) -> Self {
        self.decrease_cap_sat(100)
            .increase_cycle_light(100)
            .increase_cap_alpha(50)
    }

    pub const fn set_alpha(mut self, val: u8) -> Self {
        self.alpha = val;
        self
    }

    // pub const fn fg_color(self) -> Self {
    //     if self.light < 50 {
    //         ColorPalette::LIGHT1
    //     } else {
    //         ColorPalette::DARK1
    //     }
    // }

    pub fn color(self) -> Color {
        let color = colorsys::Rgb::from(colorsys::Hsl::from(self));

        peniko::Color::rgba(
            color.red() / 255.0,
            color.green() / 255.0,
            color.blue() / 255.0,
            color.alpha(),
        )
    }
}

impl From<(u8, u8, u8, u8)> for HSLColor {
    fn from(value: (u8, u8, u8, u8)) -> Self {
        Self {
            hue: value.0,
            sat: value.1,
            light: value.2,
            alpha: value.3,
        }
    }
}
impl From<HSLColor> for colorsys::Hsl {
    fn from(value: HSLColor) -> Self {
        colorsys::Hsl::new(
            (value.hue as f64) * 3.6,
            value.sat as f64,
            value.light as f64,
            Some((value.alpha as f64) / 100.),
        )
    }
}

#[derive(Clone, Copy)]
pub struct WindowClicked(pub Trigger);
impl Deref for WindowClicked {
    type Target = Trigger;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
pub fn windowclicked_notify() {
    let window_clicked = use_context::<WindowClicked>().unwrap();
    window_clicked.notify();
}

pub type DarkMode = bool;

pub type BorderRadiusPercent = f32;

pub type PopOver = Trigger;
pub fn follow_popover(visible_state: RwSignal<bool>) {
    let pop_over = use_context::<PopOver>().unwrap();
    create_effect(move |_| {
        pop_over.track();
        visible_state.set(false);
    });
}
pub fn popover_notify() {
    let pop_over = use_context::<PopOver>().unwrap();
    pop_over.notify();
}

pub trait DesignSystem {
    const TEXT: ResponsiveColor;
    const BACKGROUND: ResponsiveColor;
    const PRIMARY: ResponsiveColor;
    const SECONDARY: ResponsiveColor;
    const ACCENT: ResponsiveColor;
    const FONT_FAMILY: &'static str;
    const BASE_FONT_SIZE: f32;
    const BORDER_NORMAL: f32;
    const PADDING: f32;
    const SCROLL_BAR_WIDTH: f32 = 10.;
    const BORDER_RADIUS: f64 = 7.;

    const TRANSPARENT: Color = Color::TRANSPARENT;

    // computed colors
    const BACKGROUND_2: ResponsiveColor = ResponsiveColor::from_lightdark(
        Self::BACKGROUND
            .base
            .light_decrease_cap_light(10)
            .dark_increase_cap_light(10),
    );
    const BACKGROUND_3: ResponsiveColor = ResponsiveColor::from_lightdark(
        Self::BACKGROUND_2
            .base
            .light_decrease_cap_light(10)
            .dark_increase_cap_light(10),
    );
    const BACKGROUND_EXT: ResponsiveColor = ResponsiveColor::from_lightdark(
        Self::BACKGROUND
            .base
            .light_increase_cap_light(10)
            .dark_decrease_cap_light(10),
    );

    const HOVER_BACKGROUND: ResponsiveColor = ResponsiveColor {
        base: LightDark::transparent(),
        hover: Self::BACKGROUND.base,
        active: Self::BACKGROUND.hover,
    };

    const HOVER_ACCENT: ResponsiveColor = ResponsiveColor {
        base: Self::BACKGROUND.base,
        hover: Self::ACCENT.base,
        active: Self::ACCENT.hover,
    };
    const HOVER_ACCENT_2: ResponsiveColor = ResponsiveColor {
        base: Self::BACKGROUND_2.base,
        hover: Self::ACCENT.base,
        active: Self::ACCENT.hover,
    };
    const HOVER_ACCENT_3: ResponsiveColor = ResponsiveColor {
        base: Self::BACKGROUND_3.base,
        hover: Self::ACCENT.base,
        active: Self::ACCENT.hover,
    };

    // computed borders
    const BORDER_BIG: f32 = Self::BORDER_NORMAL * 2.;
    const BORDER_SMALL: f32 = Self::BORDER_NORMAL / 2.;
}

macro_rules! generate_lightdark_methods {
    ($field1:ident, $field2:ident; $($property:ident),*) => {
        $(
            generate_lightdark_property_methods!($field1, $property);
            generate_lightdark_property_methods!($field2, $property);
        )*
    };
}

macro_rules! generate_lightdark_property_methods {
    ($field:ident, $property:ident) => {
        generate_lightdark_op_methods!($field, $property, increase, cap);
        generate_lightdark_op_methods!($field, $property, decrease, cap);
        generate_lightdark_op_methods!($field, $property, increase, cycle);
        generate_lightdark_op_methods!($field, $property, decrease, cycle);
    };
}

macro_rules! generate_lightdark_op_methods {
    ($field:ident, $property:ident, $op:ident, $type:ident) => {
        paste! {
            #[doc = "Modifies the " $property " property of the " $field " field of the LightDark by " $op "ing until " $type ".
            \n\n# Arguments\n\n* `percent` - The percent by which to " $op " the HSL " $property "." ]
            pub const fn [<$field _ $op _ $type _ $property>](mut self, percent: u8) -> LightDark {
                self.[<$field>] = self.[<$field>].[<$op _ $type _ $property>](percent);
                self
            }
        }
    };
}

#[derive(Clone, Copy, Debug)]
enum LightModeDefault {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug)]
pub struct LightDark {
    pub light: HSLColor,
    pub dark: HSLColor,
    light_mode_default: LightModeDefault,
}

#[allow(unused)]
impl LightDark {
    generate_lightdark_methods!(light, dark; sat, light, alpha);

    /// Increase the hue of the dark field. Will continously cycle
    pub const fn dark_increase_hue(mut self, change: u8) -> LightDark {
        self.dark = self.dark.increase_hue(change);
        self
    }

    /// Decrease the hue of the dark field. Will continously cycle
    pub const fn dark_decrease_hue(mut self, change: u8) -> LightDark {
        self.dark = self.dark.decrease_hue(change);
        self
    }

    /// Increase the hue of the dark field. Will continously cycle
    pub const fn light_increase_hue(mut self, change: u8) -> LightDark {
        self.light = self.light.increase_hue(change);
        self
    }

    /// Decrease the hue of the dark field. Will continously cycle
    pub const fn light_decrease_hue(mut self, change: u8) -> LightDark {
        self.light = self.light.decrease_hue(change);
        self
    }

    fn get_base(self) -> HSLColor {
        let dark_mode = use_context::<RwSignal<DarkMode>>().unwrap();
        match (dark_mode.get(), self.light_mode_default) {
            (true, LightModeDefault::Light) => self.dark,
            (true, LightModeDefault::Dark) => self.light,
            (false, LightModeDefault::Light) => self.light,
            (false, LightModeDefault::Dark) => self.dark,
        }
    }

    pub fn color(self) -> Color {
        self.get_base().color()
    }

    pub const fn new(light: HSLColor, dark: HSLColor) -> Self {
        Self {
            light,
            dark,
            light_mode_default: LightModeDefault::Light,
        }
    }

    pub const fn max_contrast(mut self) -> Self {
        self.light = self.light.max_contrast();
        self.dark = self.light.max_contrast();
        self
    }

    pub const fn desat_opposite(mut self) -> Self {
        self.light = self.light.desat_opposite();
        self.dark = self.dark.desat_opposite();
        self
    }

    pub const fn reverse(mut self) -> LightDark {
        self.light_mode_default = match self.light_mode_default {
            LightModeDefault::Light => LightModeDefault::Dark,
            LightModeDefault::Dark => LightModeDefault::Light,
        };
        self
    }

    pub const fn transparent() -> Self {
        Self {
            light: HSLColor::new(0, 0, 0, 0),
            dark: HSLColor::new(0, 0, 0, 0),
            light_mode_default: LightModeDefault::Light,
        }
    }

    pub const fn set_alpha(mut self, val: u8) -> Self {
        self.light = self.light.set_alpha(val);
        self.dark = self.dark.set_alpha(val);
        self
    }
}

pub trait ExtDynamicColor<T> {
    fn dynamic_color(self, prop: T, color: ResponsiveColor) -> Self;
    fn ld_color(self, prop: T, color: LightDark) -> Self;
}
impl<T: StyleProp<Type = Option<Color>>> ExtDynamicColor<T> for Style {
    fn dynamic_color(self, prop: T, color: ResponsiveColor) -> Self {
        self.set(prop, color.base.color())
            .hover(move |s| s.set(prop, color.hover.color()))
            .active(move |s| s.set(prop, color.active.color()))
            .transition(prop, Transition::linear(0.1))
    }

    fn ld_color(self, prop: T, color: LightDark) -> Self {
        self.set(prop, color.color())
            .transition(prop, Transition::linear(0.1))
    }
}

pub trait ExtAnyEvent {
    fn all_events(
        self,
        handlers: std::collections::HashMap<
            EventListener,
            Box<dyn Fn(&Event) -> floem::EventPropagation + 'static>,
        >,
    ) -> Self;
}
impl<T: View + Decorators> ExtAnyEvent for T {
    fn all_events(
        mut self,
        handlers: std::collections::HashMap<
            EventListener,
            Box<dyn Fn(&Event) -> floem::EventPropagation + 'static>,
        >,
    ) -> Self {
        for handler in handlers {
            self = self.on_event(handler.0, handler.1);
        }
        self
    }
}
// Define a custom data structure to hold event handlers
pub struct EventHandlers {
    pub handlers: std::collections::HashMap<EventListener, Box<dyn Fn(&Event) -> bool + 'static>>,
    _x: PhantomData<()>,
}
macro_rules! add_handler {
    ($($name:ident),* $(,)?) => {
        $(
            paste::paste! {
                pub fn [<on_ $name:lower>](mut self, action: impl Fn(&Event) -> bool + 'static) -> Self {
                    self.handlers.insert(EventListener::$name, Box::new(action));
                    self
                }
            }
        )*
    };
}
impl EventHandlers {
    add_handler! {
        KeyDown,
        KeyUp,
        Click,
        DoubleClick,
        DragStart,
        DragEnd,
        DragOver,
        DragEnter,
        DragLeave,
        Drop,
        PointerDown,
        PointerMove,
        PointerUp,
        PointerEnter,
        PointerLeave,
        PointerWheel,
        FocusGained,
        FocusLost,
        WindowClosed,
        WindowResized,
        WindowMoved,
    }

    pub fn new() -> Self {
        Self {
            handlers: std::collections::HashMap::new(),
            _x: PhantomData,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ResponsiveColor {
    pub base: LightDark,
    pub hover: LightDark,
    pub active: LightDark,
}

#[allow(unused)]
impl ResponsiveColor {
    pub const fn from_lightdark(color: LightDark) -> Self {
        Self {
            base: color,
            hover: color
                .dark_increase_cap_light(10)
                .light_decrease_cap_light(10),
            active: color
                .dark_increase_cap_light(20)
                .light_decrease_cap_light(20),
        }
    }

    pub const fn reverse(mut self) -> Self {
        self.base = self.base.reverse();
        self.hover = self.hover.reverse();
        self.active = self.active.reverse();
        self
    }

    pub const fn disable_base(mut self) -> Self {
        self.base = LightDark::transparent();
        self
    }

    pub const fn disable_hover(mut self) -> Self {
        self.hover = LightDark::transparent();
        self
    }

    pub const fn disable_active(mut self) -> Self {
        self.active = LightDark::transparent();
        self
    }

    pub const fn set_hover_to_base(mut self) -> Self {
        self.hover = self.base;
        self
    }

    pub const fn reduce_active_to_hover(mut self) -> Self {
        self.active = self.hover;
        self
    }

    pub const fn set_alpha(mut self, val: u8) -> Self {
        self.base = self.base.set_alpha(val);
        self.hover = self.hover.set_alpha(val);
        self.active = self.active.set_alpha(val);
        self
    }
}
impl ResponsiveColor {
    pub const WHITE_BLACK: Self = Self::from_lightdark(LightDark::new(
        HSLColor::new(0, 100, 95, 100),
        HSLColor::new(0, 100, 5, 100),
    ));
}

// return the border radius in pixels
pub fn border_radius(rect: kurbo::Rect) -> f32 {
    let border_radius_percent = use_context::<RwSignal<BorderRadiusPercent>>()
        .unwrap()
        .get();

    ((rect.x1 - rect.x0).min(rect.y1 - rect.y0) as f32) * border_radius_percent
}

pub fn lazy_border_rad() -> (
    impl Fn(kurbo::Rect) + Copy + 'static,
    impl Fn() -> f32 + 'static + Copy,
) {
    let rect_sig = create_rw_signal(kurbo::Rect::ZERO);
    (
        move |rect| rect_sig.update(|val| *val = rect),
        move || border_radius(rect_sig.get()),
    )
}
pub fn lazy_size() -> (
    impl Fn(kurbo::Rect) + 'static,
    impl Fn() -> kurbo::Size + 'static + Copy,
) {
    let signal = create_rw_signal(kurbo::Size::default());
    let func = move |rect: kurbo::Rect| signal.update(|val| *val = rect.size());
    (func, move || signal.get())
}

pub fn lazy_size_and_rad() -> (
    impl Fn(kurbo::Rect) + 'static,
    impl Fn() -> kurbo::Size + 'static + Copy,
    impl Fn() -> f32 + 'static + Copy,
) {
    let rect_sig = create_rw_signal(kurbo::Rect::ZERO);
    let size_signal = create_rw_signal(kurbo::Size::default());
    let func = move |rect: kurbo::Rect| {
        rect_sig.update(|val| *val = rect);
        size_signal.update(|val| {
            val.width = rect.x1 - rect.x0;
            val.height = rect.y1 - rect.y0;
        })
    };
    (
        func,
        move || size_signal.get(),
        move || border_radius(rect_sig.get()),
    )
}
