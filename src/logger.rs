extern crate log;
extern crate env_logger;
extern crate pretty_env_logger;


pub fn init_logger() {
    #[cfg(not(feature = "github"))]
    {
        pretty_env_logger::init();
    }
    #[cfg(feature = "github")]
    {
        use std::io::Write;
        use env_logger::{Env, Builder};

        let env = Env::default().default_filter_or("warn");
        Builder::from_env(env).format(|buf, rec| {
                                  let level = rec.level();
                                  let l = format!("{}", level).chars().next().unwrap();
                                  let mp = rec.module_path()
                                            //   .map(|s| format!(" ({}) ", s))
                                            //   .unwrap_or_default();
                                              .unwrap_or(rec.target());

                                  use log::Level::*;
                                  match level {
                                      // just std-like log
                                      Trace => writeln!(buf,
                                                        "{level} {mp} : {msg}",
                                                        level = l,
                                                        mp = mp,
                                                        msg = rec.args()),
                                      // GH-specific
                                      Error => writeln!(buf, "::error {}", format_record(rec)),
                                      Warn => writeln!(buf, "::warning {}", format_record(rec)),
                                      Info => writeln!(buf, "::notice {}", format_record(rec)),
                                      // without details ¯\_(ツ)_/¯
                                      Debug => writeln!(buf, "::debug::{}", rec.args()),
                                  }
                              })
                              .init();
    }
}

/// Format log-record to github format, but without level prefix.
/// ```text
/// format:
/// ::debug::{message}
/// ::error file={name},line={line},endLine={endLine},title={title}::{message}
/// ::warning file={name},line={line},endLine={endLine},title={title}::{message}
/// ::notice file={name},line={line},endLine={endLine},title={title}::{message}
/// e.g.: "::notice file=main.rs,line=1,col=5,endColumn=7::Missing semicolon"
/// ```
#[cfg(feature = "github")]
fn format_record(rec: &log::Record) -> String {
    let mut parts = Vec::new();
    if let Some(path) = rec.file() {
        parts.push(format!("file={name}", name = path));
    }
    if let Some(n) = rec.line() {
        parts.push(format!("line={line}", line = n));
    }
    if let Some(mp) = rec.module_path() {
        let mp = mp.replace("::", ".");
        parts.push(format!("title={title}", title = mp));
    }

    format!("{}::{}", parts.join(","), rec.args())
}


pub use macros::*;
pub mod macros {
    #![macro_use]
    /*!
        Implements grouping for log-lines in GH-format:
        - `::group::{title}`
        - `::endgroup::`

        [format doc](https://docs.github.com/en/actions/learn-github-actions/workflow-commands-for-github-actions#grouping-log-lines)
    */

    #[macro_export]
    macro_rules! group {
        () => {{
            let mp = ::core::module_path!();
            $crate::group!(mp)
        }};
        // title is literal
        ($title:expr) => {{
            println!("::group::{title}", title = $title);

            struct GroupEnd();
            impl GroupEnd {
                #[inline(always)]
                #[must_use = "this value should be stored"]
                fn new() -> Self { Self() }
            }
            impl Drop for GroupEnd {
                fn drop(&mut self) {
                    println!("::endgroup::");
                }
            }

            GroupEnd::new()
        }};
    }
}
