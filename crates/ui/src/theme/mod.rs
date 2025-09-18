use crate::{highlighter::HighlightTheme, scroll::ScrollbarShow};
use gpui::{px, App, Global, Hsla, Pixels, SharedString, Window, WindowAppearance};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::Arc,
};

mod color;
mod registry;
mod schema;
mod theme_color;

pub use color::*;
pub use registry::*;
pub use schema::*;
pub use theme_color::*;

pub fn init(cx: &mut App) {
    registry::init(cx);

    Theme::sync_system_appearance(None, cx);
    Theme::sync_scrollbar_appearance(cx);
}

pub trait ActiveTheme {
    fn theme(&self) -> &Theme;
}

impl ActiveTheme for App {
    #[inline(always)]
    fn theme(&self) -> &Theme {
        Theme::global(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Theme {
    pub colors: ThemeColor,
    pub highlight_theme: Arc<HighlightTheme>,
    pub light_theme: Rc<ThemeConfig>,
    pub dark_theme: Rc<ThemeConfig>,

    pub mode: ThemeMode,
    pub font_family: SharedString,
    pub font_size: Pixels,
    /// Radius for the general elements.
    pub radius: Pixels,
    /// Radius for the large elements, e.g.: Modal, Notification border radius.
    pub radius_lg: Pixels,
    pub shadow: bool,
    pub transparent: Hsla,
    /// Show the scrollbar mode, default: Scrolling
    pub scrollbar_show: ScrollbarShow,
    /// Tile grid size, default is 4px.
    pub tile_grid_size: Pixels,
    /// The shadow of the tile panel.
    pub tile_shadow: bool,
}

impl Default for Theme {
    fn default() -> Self {
        Self::from(ThemeColor::default())
    }
}

impl Deref for Theme {
    type Target = ThemeColor;

    fn deref(&self) -> &Self::Target {
        &self.colors
    }
}

impl DerefMut for Theme {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.colors
    }
}

impl Global for Theme {}

impl Theme {
    /// Returns the global theme reference
    #[inline(always)]
    pub fn global(cx: &App) -> &Theme {
        cx.global::<Theme>()
    }

    /// Returns the global theme mutable reference
    #[inline(always)]
    pub fn global_mut(cx: &mut App) -> &mut Theme {
        cx.global_mut::<Theme>()
    }

    /// Returns true if the theme is dark.
    #[inline(always)]
    pub fn is_dark(&self) -> bool {
        self.mode.is_dark()
    }

    /// Returns the current theme name.
    pub fn theme_name(&self) -> &SharedString {
        if self.is_dark() {
            &self.dark_theme.name
        } else {
            &self.light_theme.name
        }
    }

    // /// Sets the theme to default light.
    // pub fn set_default_light(&mut self) {
    //     self.light_theme = ThemeColor::light();
    //     self.colors = ThemeColor::light();
    //     self.light_highlight_theme = Arc::new(HighlightTheme::default_light());
    //     self.highlight_theme = self.light_highlight_theme.clone();
    // }

    // /// Sets the theme to default dark.
    // pub fn set_default_dark(&mut self) {
    //     self.dark_theme = ThemeColor::dark();
    //     self.colors = ThemeColor::dark();
    //     self.dark_highlight_theme = Arc::new(HighlightTheme::default_dark());
    //     self.highlight_theme = self.dark_highlight_theme.clone();
    // }

    /// Sync the theme with the system appearance
    pub fn sync_system_appearance(window: Option<&mut Window>, cx: &mut App) {
        // Better use window.appearance() for avoid error on Linux.
        // https://github.com/longbridge/gpui-component/issues/104
        let appearance = window
            .as_ref()
            .map(|window| window.appearance())
            .unwrap_or_else(|| cx.window_appearance());

        Self::change(appearance, window, cx);
    }

    /// Sync the Scrollbar showing behavior with the system
    pub fn sync_scrollbar_appearance(cx: &mut App) {
        Theme::global_mut(cx).scrollbar_show = if cx.should_auto_hide_scrollbars() {
            ScrollbarShow::Scrolling
        } else {
            ScrollbarShow::Hover
        };
    }

    pub fn change(mode: impl Into<ThemeMode>, window: Option<&mut Window>, cx: &mut App) {
        let mode = mode.into();
        if !cx.has_global::<Theme>() {
            let mut theme = Theme::default();
            theme.light_theme = ThemeRegistry::global(cx).default_light_theme().clone();
            theme.dark_theme = ThemeRegistry::global(cx).default_dark_theme().clone();
            cx.set_global(theme);
        }

        let theme = cx.global_mut::<Theme>();
        theme.mode = mode;
        if mode.is_dark() {
            theme.apply_config(&theme.dark_theme.clone());
        } else {
            theme.apply_config(&theme.light_theme.clone());
        }

        if let Some(window) = window {
            window.refresh();
        }
    }
}

impl From<ThemeColor> for Theme {
    fn from(colors: ThemeColor) -> Self {
        Theme {
            mode: ThemeMode::default(),
            transparent: Hsla::transparent_black(),
            font_size: px(16.),
            font_family: if cfg!(target_os = "macos") {
                ".SystemUIFont".into()
            } else if cfg!(target_os = "windows") {
                "Segoe UI".into()
            } else {
                "FreeMono".into()
            },
            radius: px(6.),
            radius_lg: px(8.),
            shadow: true,
            scrollbar_show: ScrollbarShow::default(),
            tile_grid_size: px(8.),
            tile_shadow: true,
            colors,
            light_theme: Rc::new(ThemeConfig::default()),
            dark_theme: Rc::new(ThemeConfig::default()),
            highlight_theme: HighlightTheme::default_light(),
        }
    }
}

#[derive(
    Debug, Clone, Copy, Default, PartialEq, PartialOrd, Eq, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

impl ThemeMode {
    #[inline(always)]
    pub fn is_dark(&self) -> bool {
        matches!(self, Self::Dark)
    }

    /// Return lower_case theme name: `light`, `dark`.
    pub fn name(&self) -> &'static str {
        match self {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
        }
    }
}

impl From<WindowAppearance> for ThemeMode {
    fn from(appearance: WindowAppearance) -> Self {
        match appearance {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Self::Dark,
            WindowAppearance::Light | WindowAppearance::VibrantLight => Self::Light,
        }
    }
}
