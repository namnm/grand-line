use crate::prelude::*;
use std::env;

pub fn debug_macro(name: &str, ts: &Ts2) {
    if env::var("DEBUG_MACRO").unwrap_or_default() != "1" {
        return;
    }

    #[cfg(feature = "debug_macro_cli")]
    {
        use colored::{Colorize as _, control::SHOULD_COLORIZE};
        use prettyplease::unparse;

        SHOULD_COLORIZE.set_override(true);
        println!("==============================================================================");
        println!("{}", name.bold());
        println!();

        let code = parse2::<File>(ts.clone())
            .map(|v| unparse(&v))
            .unwrap_or_else(|_| ts.to_string());
        println!("{}", code.bright_black());
    };

    #[cfg(feature = "debug_macro_file")]
    {
        use std::fs::{File, create_dir_all};
        use std::io::Write as _;
        use std::path::PathBuf;
        use std::process::Command;

        let code = ts.to_string();
        let path = format!("target/grand-line/{name}.rs");
        let path = PathBuf::from(path);

        let Some(parent) = path.parent() else {
            eprintln!("debug_macro: path.parent returned None for {path:?}");
            return;
        };

        if let Err(e) = create_dir_all(parent) {
            eprintln!("debug_macro: create_dir_all failed: {e}");
            return;
        }

        let mut file = match File::create(&path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("debug_macro: File::create failed: {e}");
                return;
            }
        };

        if let Err(e) = writeln!(file, "{code}") {
            eprintln!("debug_macro: writeln failed: {e}");
            return;
        }

        let cmd = Command::new("rustfmt").arg("--edition").arg("2024").arg(&path).status();
        if let Err(e) = cmd {
            eprintln!("debug_macro: rustfmt failed: {e}");
        }
    }
}
