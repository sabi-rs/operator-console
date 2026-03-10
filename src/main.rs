use std::env;

use color_eyre::eyre::Result;
use operator_console::app::App;

fn main() -> Result<()> {
    color_eyre::install()?;

    if env::args()
        .skip(1)
        .any(|arg| arg == "--help" || arg == "-h")
    {
        println!("{}", help_text());
        return Ok(());
    }

    let mut app = App::default();
    let mut terminal = ratatui::init();
    let result = app.run(&mut terminal);
    ratatui::restore();
    result
}

fn help_text() -> &'static str {
    "operator-console\n\nUsage:\n  operator-console\n\nOptions:\n  -h, --help    Show this help\n"
}
