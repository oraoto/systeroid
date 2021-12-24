use getopts::Options;
use std::env;
use std::path::PathBuf;
use systeroid_core::regex::Regex;
use systeroid_core::sysctl::display::DisplayType;
use systeroid_core::sysctl::DEFAULT_PRELOAD;

/// Help message for the arguments.
const HELP_MESSAGE: &str = r#"
Usage:
    {bin} [options] [variable[=value] ...] --load[=<file>]

Options:
{usage}

For more details see {bin}(8)."#;

/// Command-line arguments.
#[derive(Debug, Default)]
pub struct Args {
    /// Whether if the verbose logging is enabled.
    pub verbose: bool,
    /// Whether if the quiet mode is enabled.
    pub quiet: bool,
    /// Whether if only the write mode is enabled.
    pub write: bool,
    /// Path of the Linux kernel documentation.
    pub kernel_docs: Option<PathBuf>,
    /// Display type of the variables.
    pub display_type: DisplayType,
    /// Whether if the deprecated variables should be included while listing.
    pub display_deprecated: bool,
    /// Whether if the unknown variable errors should be ignored.
    pub ignore_errors: bool,
    /// Do not pipe output into a pager.
    pub no_pager: bool,
    /// Whether if files are given to preload values.
    pub preload_files: bool,
    /// Whether if the values will be preloaded from system.
    pub preload_system_files: bool,
    /// Pattern for matching the variables.
    pub pattern: Option<Regex>,
    /// Whether if the documentation should be shown.
    pub explain: bool,
    /// Whether if the output should be in tree format.
    pub tree_output: bool,
    /// Free string fragments.
    pub values: Vec<String>,
}

impl Args {
    /// Returns the available options.
    fn get_options() -> Options {
        let mut opts = Options::new();
        opts.optflag("a", "all", "display all variables");
        opts.optflag("T", "tree", "display all variables in tree format");
        opts.optflag("A", "", "alias of -a");
        opts.optflag("X", "", "alias of -a");
        opts.optflag(
            "",
            "deprecated",
            "include deprecated variables while listing",
        );
        opts.optflag("e", "ignore", "ignore unknown variable errors");
        opts.optflag("N", "names", "print only variable names");
        opts.optflag("n", "values", "print only variable values");
        opts.optflag("b", "binary", "print only variable values without new line");
        opts.optflag("p", "load", "read values from file");
        opts.optflag("f", "", "alias of -p");
        opts.optflag("S", "system", "read values from all system directories");
        opts.optopt(
            "r",
            "pattern",
            "use a regex for matching variable names",
            "<expr>",
        );
        opts.optflag("q", "quiet", "do not print variable after the value is set");
        opts.optflag("w", "write", "only enable writing a value to variable");
        opts.optflag("o", "", "does nothing");
        opts.optflag("x", "", "does nothing");
        opts.optflag("d", "", "alias of -h");
        opts.optflag(
            "E",
            "explain",
            "provide a detailed explanation for variable",
        );
        opts.optopt(
            "D",
            "docs",
            "set the path of the kernel documentation",
            "<path>",
        );
        opts.optflag("P", "no-pager", "do not pipe output into a pager");
        opts.optflag("v", "verbose", "enable verbose logging");
        opts.optflag("h", "help", "display this help and exit");
        opts.optflag("V", "version", "output version information and exit");
        opts
    }

    /// Parses the command-line arguments.
    pub fn parse(env_args: Vec<String>) -> Option<Self> {
        let opts = Self::get_options();
        let mut matches = opts
            .parse(&env_args[1..])
            .map_err(|e| eprintln!("error: `{}`", e))
            .ok()?;

        let preload_files = matches.opt_present("p") || matches.opt_present("f");
        let show_help = matches.opt_present("h") || matches.opt_present("d");
        let display_all = matches.opt_present("a")
            || matches.opt_present("A")
            || matches.opt_present("X")
            || matches.opt_present("N")
            || matches.opt_present("n")
            || matches.opt_present("b");
        let required_args_present = !matches.free.is_empty()
            || display_all
            || preload_files
            || matches.opt_present("S")
            || matches.opt_present("r")
            || matches.opt_present("E")
            || matches.opt_present("T");

        if show_help || env_args.len() == 1 {
            let usage = opts.usage_with_format(|opts| {
                HELP_MESSAGE
                    .replace("{bin}", env!("CARGO_PKG_NAME"))
                    .replace(
                        "{usage}",
                        &opts
                            .filter(|msg| {
                                !(msg.contains("alias of") || msg.contains("does nothing"))
                            })
                            .collect::<Vec<String>>()
                            .join("\n"),
                    )
            });
            println!("{}", usage);
            None
        } else if matches.opt_present("V") {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            None
        } else if !required_args_present {
            eprintln!(
                "{}: no variables specified\n\
                Try `{} --help' for more information.",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_NAME")
            );
            None
        } else {
            let display_type = if matches.opt_present("N") {
                DisplayType::Name
            } else if matches.opt_present("n") {
                DisplayType::Value
            } else if matches.opt_present("b") {
                DisplayType::Binary
            } else {
                DisplayType::Default
            };
            if preload_files && matches.free.is_empty() {
                matches.free = vec![DEFAULT_PRELOAD.to_string()];
            }
            Some(Args {
                verbose: matches.opt_present("v"),
                quiet: matches.opt_present("q"),
                write: matches.opt_present("w"),
                kernel_docs: matches.opt_str("D").map(PathBuf::from),
                display_type,
                display_deprecated: matches.opt_present("deprecated"),
                ignore_errors: matches.opt_present("e"),
                tree_output: matches.opt_present("T"),
                no_pager: matches.opt_present("P"),
                preload_files,
                preload_system_files: matches.opt_present("S"),
                pattern: matches
                    .opt_str("r")
                    .map(|v| Regex::new(&v).expect("invalid regex")),
                explain: matches.opt_present("E"),
                values: matches.free,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args() {
        for env_args in [
            vec![String::new()],
            vec![String::new(), String::from("-r")],
            vec![String::new(), String::from("-V")],
        ] {
            assert!(Args::parse(env_args).is_none());
        }

        let args = Args::parse(vec![
            String::new(),
            String::from("-v"),
            String::from("-w"),
            String::from("-A"),
            String::from("-T"),
        ])
        .unwrap();
        assert!(args.verbose);
        assert!(args.write);
        assert!(args.tree_output);

        assert!(!Args::parse(vec![String::new(), String::from("-p")])
            .unwrap()
            .values
            .is_empty());

        assert_eq!(
            DisplayType::Binary,
            Args::parse(vec![String::new(), String::from("-A"), String::from("-b")])
                .unwrap()
                .display_type
        );
    }
}
