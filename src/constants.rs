// gui update time for commhandler (ns)
pub const GUI_UPDATE_TIME: u64 = 5000000;

// theme setup constants
#[cfg(windows)]
pub const DEFAULT_GTK_CSS_CONFIG: &'static str = "..\\gtk.css";

#[cfg(not(windows))]
pub const DEFAULT_GTK_CSS_CONFIG: &'static str = "../gtk.css";

pub const SECONDARY_GTK_CSS_CONFIG: &'static str = "gtk.css";

pub const GTK_THEME_CFG: &'static str = "theme.txt";

// determines the amount of time to keep recent byte amount before resetting it to 0
// (recent bytes downloaded in this time)
// in seconds
pub const DOWNLOAD_SPEED_UPDATE_TIME: f64 = 1.0;

// milliseconds before giving up on a connection
pub const CONNECT_MILLI_TIMEMOUT: u64 = 5000;
