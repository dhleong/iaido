use backtrace::{Backtrace, BacktraceFmt, BacktraceFrame, PrintFmt};
use std::fmt::{self, Formatter};

const ABBRIATABLE_FRAME_PREFIXES: &[&'static str] = &[
    "backtrace::",
    "std::sys_common::backtrace::",
    "iaido::prepare_panic_capture",
    "std::panicking",
    "core::panicking",
    "rust_begin_unwind",
];

pub struct PanicData {
    pub info: String,
    pub trace: Backtrace,
}

impl std::fmt::Display for PanicData {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut print_path =
            move |fmt: &mut Formatter<'_>, path: backtrace::BytesOrWideString<'_>| path.fmt(fmt);
        let mut f = BacktraceFmt::new(fmt, PrintFmt::Short, &mut print_path);
        let mut ignoring = false;
        for frame in self.trace.frames() {
            let symbols = frame.symbols();
            if !symbols.is_empty() {
                let should_ignore = symbols.iter().any(|s| {
                    if let Some(name) = s.name() {
                        let name_str = name.to_string();
                        ABBRIATABLE_FRAME_PREFIXES
                            .iter()
                            .any(|prefix| name_str.starts_with(prefix))
                    } else {
                        false
                    }
                });

                if should_ignore && !ignoring {
                    print_abbreviated(&mut f, frame)?;
                }
                ignoring = should_ignore;

                if should_ignore {
                    continue;
                }
            }

            f.frame().backtrace_frame(frame)?;

            if symbols.iter().any(|s| {
                if let Some(name) = s.name() {
                    name.to_string().starts_with("iaido::main_loop")
                } else {
                    false
                }
            }) {
                print_abbreviated(&mut f, frame)?;
                break;
            }
        }
        Ok(())
    }
}

fn print_abbreviated(f: &mut BacktraceFmt, frame: &BacktraceFrame) -> fmt::Result {
    f.frame().print_raw(
        frame.ip(),
        Some(backtrace::SymbolName::new(
            " ... abbreviated ... ".as_bytes(),
        )),
        None,
        None,
    )
}
