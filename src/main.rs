// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process;

use clap::Arg;
use clap::ArgAction;
use clap::Command;
use gettextrs::{bind_textdomain_codeset, bindtextdomain, gettext, setlocale, textdomain, LocaleCategory};

use ualias::{
    append_pattern, load_devices_dir, lookup_name, pattern_for_append, ucfirst, DevicePattern,
    UaliasError, DEFAULT_UALIAS_DIR,
};

const TEXT_DOMAIN: &str = "ualias";
const PROJECT_AUTHOR: &str = "Lenik";
const PROJECT_EMAIL: &str = "ualias@bodz.net";
const PROJECT_YEAR: i32 = 2026;
const UALIAS_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Resolve GNU gettext "localedir" (contains `<lang>/LC_MESSAGES/<domain>.mo`).
fn pick_localedir() -> String {
    if let Ok(s) = std::env::var("UALIAS_LOCALEDIR") {
        return s;
    }
    for candidate in ["/build/po", "build/po"] {
        if Path::new(candidate).is_dir() {
            return candidate.to_string();
        }
    }
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = format!("{manifest}/../build/po");
        if Path::new(&p).is_dir() {
            return p;
        }
    }
    "/usr/local/share/locale".to_string()
}

fn init_gettext() {
    let _ = setlocale(LocaleCategory::LcAll, "");
    let localedir = pick_localedir();
    if bindtextdomain(TEXT_DOMAIN, &localedir).is_err()
        || textdomain(TEXT_DOMAIN).is_err()
        || bind_textdomain_codeset(TEXT_DOMAIN, "UTF-8").is_err()
    {
        // Translations are optional: continue with msgids.
    }
}

fn c_format_replace(template: &str, pairs: &[(&str, &str)]) -> String {
    let mut s = template.to_string();
    for (a, b) in pairs {
        s = s.replace(a, b);
    }
    s
}

fn usage() {
    let msg = gettext(
        "Usage: ualias [OPTION]... [SERIAL [NAME]]\n\
Look up or append a USB device alias in /etc/udev/ualias.d/*.\n\
Without --name (or NAME), print the matching alias. With a name, append to ualias.d/local.\n",
    );
    let bugs = c_format_replace(
        &gettext("Report bugs to: <%s>\n"),
        &[("%s", PROJECT_EMAIL)],
    );
    eprintln!();
    eprintln!("{msg}");
    eprintln!();
    eprintln!("  -w, --vendor ID    {}", gettext("USB vendor id (hex)\n").trim_end());
    eprintln!("  -p, --product ID   {}", gettext("USB product id (hex)\n").trim_end());
    eprintln!("  -s, --serial SER   {}", gettext("device serial / MAC\n").trim_end());
    eprintln!("  -n, --name NAME    {}", gettext("alias name (append mode)\n").trim_end());
    eprintln!("  -u, --ucfirst      {}", gettext("uppercase the first letter of the name\n").trim_end());
    eprintln!("  -h, --help         {}", gettext("display this help and exit\n").trim_end());
    eprintln!("      --version      {}", gettext("output version information and exit\n").trim_end());
    eprintln!();
    eprint!("{bugs}");
}

fn print_version() {
    println!("ualias {UALIAS_VERSION}");
    let cpy = c_format_replace(
        &gettext("Copyright (C) %d %s\n"),
        &[("%d", &format!("{PROJECT_YEAR}")), ("%s", PROJECT_AUTHOR)],
    );
    print!("{cpy}");
    print!("{}", gettext("License AGPL-3.0-or-later: <https://www.gnu.org/licenses/agpl-3.0.html>\n"));
    print!("{}", gettext("This is free software: you are free to change and redistribute it.\n"));
    print!("{}", gettext("This project opposes AI exploitation and AI hegemony.\n"));
    print!("{}", gettext(
        "This project rejects mindless MIT-style licensing and politically naive BSD-style licensing.\n",
    ));
    print!("{}", gettext("There is NO WARRANTY, to the extent permitted by law.\n"));
}

fn self_exe() -> String {
    std::env::args()
        .next()
        .as_ref()
        .map(|s| {
            let p = Path::new(s);
            p.file_name()
                .map(OsStr::to_string_lossy)
                .map(|c| c.into_owned())
                .unwrap_or_else(|| s.clone())
        })
        .unwrap_or_else(|| "ualias".to_string())
}

fn ualias_dir() -> PathBuf {
    if let Ok(s) = std::env::var("UALIAS_DIR") {
        return PathBuf::from(s);
    }
    // Backward-compatible override name from the single-file era.
    if let Ok(s) = std::env::var("UALIAS_DEVICES") {
        return PathBuf::from(s);
    }
    PathBuf::from(DEFAULT_UALIAS_DIR)
}

fn maybe_ucfirst(name: &str, enabled: bool) -> String {
    if enabled {
        ucfirst(name)
    } else {
        name.to_string()
    }
}

fn run() -> i32 {
    init_gettext();

    let cmd = Command::new("ualias")
        .disable_version_flag(true)
        .disable_help_flag(true)
        .arg(
            Arg::new("vendor")
                .short('w')
                .long("vendor")
                .value_name("ID")
                .help(
                    gettext("USB vendor id (hex)\n")
                        .trim_end_matches('\n')
                        .to_string(),
                ),
        )
        .arg(
            Arg::new("product")
                .short('p')
                .long("product")
                .value_name("ID")
                .help(
                    gettext("USB product id (hex)\n")
                        .trim_end_matches('\n')
                        .to_string(),
                ),
        )
        .arg(
            Arg::new("serial")
                .short('s')
                .long("serial")
                .value_name("SERIAL")
                .help(
                    gettext("device serial / MAC\n")
                        .trim_end_matches('\n')
                        .to_string(),
                ),
        )
        .arg(
            Arg::new("name")
                .short('n')
                .long("name")
                .value_name("NAME")
                .help(
                    gettext("alias name (append mode)\n")
                        .trim_end_matches('\n')
                        .to_string(),
                ),
        )
        .arg(
            Arg::new("ucfirst")
                .short('u')
                .long("ucfirst")
                .action(ArgAction::SetTrue)
                .help(
                    gettext("uppercase the first letter of the name\n")
                        .trim_end_matches('\n')
                        .to_string(),
                ),
        )
        .arg(
            Arg::new("help")
                .short('h')
                .long("help")
                .action(ArgAction::SetTrue)
                .help(
                    gettext("display this help and exit\n")
                        .trim_end_matches('\n')
                        .to_string(),
                ),
        )
        .arg(
            Arg::new("version")
                .long("version")
                .action(ArgAction::SetTrue)
                .help(
                    gettext("output version information and exit\n")
                        .trim_end_matches('\n')
                        .to_string(),
                ),
        )
        .arg(
            Arg::new("serial_pos")
                .value_name("SERIAL")
                .help(gettext("device serial / MAC").to_string()),
        )
        .arg(
            Arg::new("name_pos")
                .value_name("NAME")
                .help(gettext("alias name (append mode)").to_string())
                .requires("serial_pos"),
        );
    let m = match cmd.try_get_matches() {
        Ok(m) => m,
        Err(e) => {
            e.print().expect("write stderr");
            return 2;
        }
    };
    if m.get_flag("version") {
        print_version();
        return 0;
    }
    if m.get_flag("help") {
        usage();
        return 0;
    }

    let exe = self_exe();
    let do_ucfirst = m.get_flag("ucfirst");

    let vendor = m.get_one::<String>("vendor").map(String::as_str);
    let product = m.get_one::<String>("product").map(String::as_str);

    let serial = m
        .get_one::<String>("serial")
        .or_else(|| m.get_one::<String>("serial_pos"))
        .map(String::as_str);
    let name = m
        .get_one::<String>("name")
        .or_else(|| m.get_one::<String>("name_pos"))
        .map(String::as_str);

    if vendor.is_none() && product.is_none() && serial.is_none() && name.is_none() {
        eprintln!(
            "{}",
            c_format_replace(
                &gettext("%s: missing vendor, product, serial, or name\n"),
                &[("%s", &exe)],
            )
            .trim_end()
        );
        usage();
        return 1;
    }

    let dir = ualias_dir();

    if let Some(name) = name {
        let name = maybe_ucfirst(name, do_ucfirst);
        let pat: DevicePattern = pattern_for_append(vendor, product, serial, &name);
        if let Err(e) = append_pattern(&dir, &pat) {
            eprintln!("{exe}: {}: {e}", dir.display());
            return 1;
        }
        return 0;
    }

    // Lookup mode
    let patterns = match load_devices_dir(&dir) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{exe}: {}: {e}", dir.display());
            return 1;
        }
    };
    match lookup_name(&patterns, vendor, product, serial) {
        Ok(n) => {
            println!("{}", maybe_ucfirst(&n, do_ucfirst));
            0
        }
        Err(UaliasError::NoMatch) => {
            eprintln!("{}", gettext("no matching alias").trim_end());
            1
        }
        Err(e) => {
            eprintln!("{exe}: {e}");
            1
        }
    }
}

fn main() {
    let code = run();
    process::exit(code);
}
