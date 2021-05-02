use std::path::PathBuf;

use clap::{self, crate_version, Arg};
use indoc::indoc;

pub enum CliInit {
    Uri(String),
    ScriptFile(String),
}

pub struct Args {
    pub demo: bool,
    pub init: Option<CliInit>,
}

pub fn args<'a>() -> Args {
    let matches = clap::App::new("iaido")
        .version(crate_version!())
        .about("the sharper mu* client")
        .long_about(indoc! {"
            the sharper mu* client

            iaido is a vim-inspired modal, CLI MU* client
        "})
        .arg(
            Arg::with_name("TARGET")
                .help("The hostname or URI to connect to, or a script file to load"),
        )
        .arg(
            Arg::with_name("PORT")
                .requires("TARGET")
                .validator(validate_port)
                .help("If providing a hostname to TARGET, the port may be provided separately"),
        )
        .arg(
            Arg::with_name("demo")
                .long("demo")
                .hidden(true)
                .help("Fills the UI with some state"),
        )
        .get_matches();

    let init = if let Some(target) = matches.value_of("TARGET") {
        Some(parse_target(target, matches.value_of("PORT")).unwrap_or_else(|e| e.exit()))
    } else {
        None
    };

    Args {
        demo: matches.is_present("demo"),
        init,
    }
}

fn parse_target(target: &str, port: Option<&str>) -> Result<CliInit, clap::Error> {
    if let Some(port_str) = port {
        if target.find(":").is_some() {
            return Err(clap::Error::value_validation_auto(format!(
                "Unexpected PORT argument ({}) with URI-formatted target ({})",
                port_str, target
            )));
        }

        return Ok(CliInit::Uri(format!("{}:{}", target, port_str)));
    }

    if target.find(":").is_some() && !target.starts_with("file:") {
        return Ok(CliInit::Uri(target.to_string()));
    }

    let path_buf = PathBuf::from(target);
    if path_buf.exists() {
        return Ok(CliInit::ScriptFile(
            path_buf
                .canonicalize()
                .expect("Unable to canonicalize path")
                .to_string_lossy()
                .to_string(),
        ));
    }

    Err(clap::Error::value_validation_auto(format!(
        "Unexpected TARGET format: {} (file may not exist)",
        target
    )))
}

fn validate_port(target: String) -> Result<(), String> {
    if let Err(_) = target.parse::<u32>() {
        Err("Port must be an integer".to_string())
    } else {
        Ok(())
    }
}
