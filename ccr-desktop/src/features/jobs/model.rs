//! Command workbench model: catalog classification, search and draft checks.
//!
//! 无 GPUI 依赖。视图只消费这里的结果，不自行判断命令能否运行。

use ccr_app::jobs::policy::CommandInfo;

/// Where a catalog command can be completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Availability {
    Runnable,
    PreviewOnly,
    DedicatedPage,
}

/// Catalog counts per filter, computed before search.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Counts {
    pub runnable: usize,
    pub all: usize,
    pub preview_only: usize,
    pub dedicated_page: usize,
}

/// The command catalog as the workbench reads it.
pub struct Catalog {
    entries: Vec<CommandInfo>,
}

impl Catalog {
    pub fn new(entries: Vec<CommandInfo>) -> Self {
        Self { entries }
    }

    pub fn counts(&self) -> Counts {
        let mut counts = Counts {
            all: self.entries.len(),
            ..Counts::default()
        };
        for entry in &self.entries {
            match availability(entry) {
                Availability::Runnable => counts.runnable += 1,
                Availability::PreviewOnly => counts.preview_only += 1,
                Availability::DedicatedPage => counts.dedicated_page += 1,
            }
        }
        counts
    }

    /// Entries in `filter` whose name or description contains `query`,
    /// ignoring case. Catalog order is kept.
    pub fn visible(&self, filter: Filter, query: &str) -> Vec<&CommandInfo> {
        let query = query.trim().to_lowercase();
        self.entries
            .iter()
            .filter(|entry| match filter {
                Filter::All => true,
                Filter::Runnable => availability(entry) == Availability::Runnable,
                Filter::PreviewOnly => availability(entry) == Availability::PreviewOnly,
                Filter::DedicatedPage => availability(entry) == Availability::DedicatedPage,
            })
            .filter(|entry| {
                query.is_empty()
                    || entry.name.to_lowercase().contains(&query)
                    || entry.description.to_lowercase().contains(&query)
            })
            .collect()
    }
}

/// Why an argument draft cannot be split into words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenizeError {
    UnclosedQuote,
}

/// Splits an argument draft on whitespace; double quotes group words.
pub fn tokenize(text: &str) -> Result<Vec<String>, TokenizeError> {
    let mut words = Vec::new();
    let mut word: Option<String> = None;
    let mut quoted = false;
    for ch in text.chars() {
        if ch == '"' {
            quoted = !quoted;
            word.get_or_insert_with(String::new);
        } else if ch.is_whitespace() && !quoted {
            words.extend(word.take());
        } else {
            word.get_or_insert_with(String::new).push(ch);
        }
    }
    if quoted {
        return Err(TokenizeError::UnclosedQuote);
    }
    words.extend(word);
    Ok(words)
}

/// The full command line shown in the preview well: `ccr <command> <args>`.
pub fn preview(command: &str, text: &str) -> String {
    let mut line = format!("ccr {command}");
    match tokenize(text) {
        Ok(words) => {
            for word in words {
                line.push(' ');
                if word.is_empty() || word.chars().any(char::is_whitespace) {
                    line.push('"');
                    line.push_str(&word);
                    line.push('"');
                } else {
                    line.push_str(&word);
                }
            }
        }
        Err(_) => {
            line.push(' ');
            line.push_str(text.trim());
        }
    }
    line
}

/// Why Run is disabled for the loaded command and argument draft.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Blocked {
    UnknownCommand,
    NotRunnable(Availability),
    Tokenize(TokenizeError),
    MissingRequired(Vec<&'static str>),
    Running,
}

impl Catalog {
    pub fn get(&self, name: &str) -> Option<&CommandInfo> {
        self.entries.iter().find(|entry| entry.name == name)
    }

    /// The arguments Run would send, or the first reason it is disabled.
    /// The backend allowlist still validates whatever is sent.
    pub fn readiness(
        &self,
        command: &str,
        text: &str,
        running: bool,
    ) -> Result<Vec<String>, Blocked> {
        let entry = self.get(command).ok_or(Blocked::UnknownCommand)?;
        match availability(entry) {
            Availability::Runnable => {}
            other => return Err(Blocked::NotRunnable(other)),
        }
        let words = tokenize(text).map_err(Blocked::Tokenize)?;
        let missing = missing_positionals(entry, &words);
        if !missing.is_empty() {
            return Err(Blocked::MissingRequired(missing));
        }
        if running {
            return Err(Blocked::Running);
        }
        Ok(words)
    }
}

// 位置参数按出现顺序对应 catalog 的 args；取值 flag 的下一个词是它的值，不计入位置参数。
fn missing_positionals(entry: &CommandInfo, words: &[String]) -> Vec<&'static str> {
    let mut positionals = 0;
    let mut words = words.iter();
    while let Some(word) = words.next() {
        if word.starts_with('-') && word.len() > 1 {
            let takes_value = !word.contains('=')
                && entry.flags.iter().any(|flag| {
                    flag.takes_value
                        && (flag.name == word || flag.aliases.iter().any(|alias| alias == word))
                });
            if takes_value {
                words.next();
            }
        } else {
            positionals += 1;
        }
    }
    entry
        .args
        .iter()
        .skip(positionals)
        .filter(|arg| arg.required)
        .map(|arg| arg.name)
        .collect()
}

/// One argument the catalog declares, as shown under the argument input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hint {
    pub syntax: String,
    pub required: bool,
}

/// Declared positionals first (`<name>` required, `[name]` optional), then flags.
pub fn hints(entry: &CommandInfo) -> Vec<Hint> {
    let positionals = entry.args.iter().map(|arg| Hint {
        syntax: if arg.required {
            format!("<{}>", arg.name)
        } else {
            format!("[{}]", arg.name)
        },
        required: arg.required,
    });
    let flags = entry.flags.iter().map(|flag| {
        let mut syntax = std::iter::once(flag.name)
            .chain(flag.aliases.iter().copied())
            .collect::<Vec<_>>()
            .join(", ");
        if flag.takes_value {
            syntax.push_str(&format!(" <{}>", flag.flag_type));
        }
        Hint {
            syntax,
            required: false,
        }
    });
    positionals.chain(flags).collect()
}

/// Catalog filter chips; the workbench opens on `Runnable`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Filter {
    #[default]
    Runnable,
    All,
    PreviewOnly,
    DedicatedPage,
}

/// `executable` wins; `typed_only` needs its page; everything else
/// (`preview_only`, `interactive`) is preview only.
pub fn availability(entry: &CommandInfo) -> Availability {
    if entry.executable {
        Availability::Runnable
    } else if entry.risk == "typed_only" {
        Availability::DedicatedPage
    } else {
        Availability::PreviewOnly
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ccr_app::jobs::policy::command_catalog;

    #[test]
    fn real_catalog_counts_six_runnable_fourteen_preview_four_dedicated_of_twenty_four() {
        let catalog = Catalog::new(command_catalog());
        assert_eq!(
            catalog.counts(),
            Counts {
                runnable: 6,
                all: 24,
                preview_only: 14,
                dedicated_page: 4,
            }
        );
    }

    fn names(entries: Vec<&CommandInfo>) -> Vec<&'static str> {
        entries.into_iter().map(|entry| entry.name).collect()
    }

    #[test]
    fn filters_split_the_catalog_and_search_matches_name_or_description_ignoring_case() {
        let catalog = Catalog::new(command_catalog());
        assert_eq!(
            names(catalog.visible(Filter::Runnable, "")),
            ["status", "validate", "history", "version", "help", "show"]
        );
        assert_eq!(
            names(catalog.visible(Filter::DedicatedPage, "")),
            ["switch", "delete", "import", "restore"]
        );
        let preview = names(catalog.visible(Filter::PreviewOnly, ""));
        assert_eq!(preview.len(), 14);
        assert!(preview.contains(&"add"), "interactive is preview only");
        assert_eq!(catalog.visible(Filter::All, "").len(), 24);
        // Name match, case-insensitive.
        assert_eq!(
            names(catalog.visible(Filter::Runnable, "HIST")),
            ["history"]
        );
        // Description match: only `history` says "configuration operations".
        assert_eq!(
            names(catalog.visible(Filter::All, "Configuration OPERATIONS")),
            ["history"]
        );
        // Search narrows within the filter; title alone does not match.
        assert!(
            catalog
                .visible(Filter::Runnable, "Audit history")
                .is_empty()
        );
        assert!(
            catalog
                .visible(Filter::DedicatedPage, "zzz-none")
                .is_empty()
        );
        // Counts stay computed before search.
        assert_eq!(catalog.counts().runnable, 6);
    }

    #[test]
    fn tokenize_splits_on_whitespace_groups_double_quotes_and_rejects_an_open_quote() {
        assert_eq!(
            tokenize("-l 20 -t switch"),
            Ok(vec!["-l".into(), "20".into(), "-t".into(), "switch".into()])
        );
        // Runs of spaces and tabs are one separator; edges are ignored.
        assert_eq!(
            tokenize("  -l   20 \t --json  "),
            Ok(vec!["-l".into(), "20".into(), "--json".into()])
        );
        assert_eq!(tokenize("   "), Ok(Vec::new()));
        assert_eq!(
            tokenize(r#"--type "two words" "C:\Program Files\x""#),
            Ok(vec![
                "--type".into(),
                "two words".into(),
                r"C:\Program Files\x".into()
            ])
        );
        // Quotes join with adjacent text; an empty pair is an empty word.
        assert_eq!(
            tokenize(r#"--name="a b" """#),
            Ok(vec!["--name=a b".into(), String::new()])
        );
        assert_eq!(tokenize(r#"-t "switch"#), Err(TokenizeError::UnclosedQuote));
    }

    #[test]
    fn preview_line_shows_ccr_command_and_normalized_arguments() {
        assert_eq!(preview("status", ""), "ccr status");
        assert_eq!(preview("status", "   "), "ccr status");
        assert_eq!(
            preview("history", "-l   20 -t switch"),
            "ccr history -l 20 -t switch"
        );
        // Words that contain spaces, or are empty, are shown quoted again.
        assert_eq!(
            preview("history", r#"-t  "two words" """#),
            r#"ccr history -t "two words" """#
        );
        // An open quote cannot be split; the draft is shown as typed.
        assert_eq!(
            preview("history", r#" -t "open "#),
            r#"ccr history -t "open"#
        );
    }

    #[test]
    fn readiness_sends_split_arguments_or_names_each_blocking_reason() {
        let catalog = Catalog::new(command_catalog());
        assert_eq!(
            catalog.readiness("history", "-l 20", false),
            Ok(vec!["-l".into(), "20".into()])
        );
        assert_eq!(catalog.readiness("status", "", false), Ok(Vec::new()));
        assert_eq!(
            catalog.readiness("missing", "", false),
            Err(Blocked::UnknownCommand)
        );
        assert_eq!(
            catalog.readiness("list", "", false),
            Err(Blocked::NotRunnable(Availability::PreviewOnly))
        );
        assert_eq!(
            catalog.readiness("add", "", false),
            Err(Blocked::NotRunnable(Availability::PreviewOnly))
        );
        assert_eq!(
            catalog.readiness("delete", "", false),
            Err(Blocked::NotRunnable(Availability::DedicatedPage))
        );
        assert_eq!(
            catalog.readiness("history", r#"-t "switch"#, false),
            Err(Blocked::Tokenize(TokenizeError::UnclosedQuote))
        );
        assert_eq!(
            catalog.readiness("history", "-l 20", true),
            Err(Blocked::Running)
        );
    }

    #[test]
    fn readiness_requires_declared_positionals_and_skips_flag_values() {
        let real = command_catalog();
        let mut need = real
            .iter()
            .find(|entry| entry.name == "help")
            .cloned()
            .unwrap_or_else(|| panic!("help entry"));
        need.name = "need";
        need.args[0].required = true;
        need.flags = real
            .iter()
            .find(|entry| entry.name == "history")
            .map(|entry| entry.flags.clone())
            .unwrap_or_else(|| panic!("history entry"));
        let catalog = Catalog::new(vec![need]);
        assert_eq!(
            catalog.readiness("need", "", false),
            Err(Blocked::MissingRequired(vec!["path"]))
        );
        // `20` is the value of `-l`, not the positional.
        assert_eq!(
            catalog.readiness("need", "-l 20", false),
            Err(Blocked::MissingRequired(vec!["path"]))
        );
        assert_eq!(
            catalog.readiness("need", "--limit=20 validate", false),
            Ok(vec!["--limit=20".into(), "validate".into()])
        );
        assert_eq!(
            catalog.readiness("need", "-l 20 validate", false),
            Ok(vec!["-l".into(), "20".into(), "validate".into()])
        );
    }

    fn hint(syntax: &str, required: bool) -> Hint {
        Hint {
            syntax: syntax.into(),
            required,
        }
    }

    #[test]
    fn hints_list_declared_positionals_then_flags_with_aliases_and_value_types() {
        let real = command_catalog();
        let entry = |name: &str| {
            real.iter()
                .find(|entry| entry.name == name)
                .cloned()
                .unwrap_or_else(|| panic!("{name} entry"))
        };
        assert_eq!(
            hints(&entry("history")),
            [
                hint("--limit, -l <number>", false),
                hint("--type, -t <text>", false)
            ]
        );
        assert_eq!(
            hints(&entry("status")),
            [hint("--json", false), hint("--verbose", false)]
        );
        assert_eq!(hints(&entry("help")), [hint("[path]", false)]);
        assert!(hints(&entry("version")).is_empty());
        let mut need = entry("help");
        need.args[0].required = true;
        assert_eq!(hints(&need), [hint("<path>", true)]);
    }
}
