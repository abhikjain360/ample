use std::path::PathBuf;

/// A nice sensible music player which uses vim-like bindings.
#[derive(Debug, Default, argh::FromArgs)]
pub(crate) struct Opts {
    /// path to the configuration file to use.
    #[argh(option, short = 'c')]
    #[expect(dead_code)]
    pub(crate) config_path: Option<PathBuf>,

    /// path to the settings file to use.
    #[argh(option, short = 's')]
    pub(crate) settings_path: Option<PathBuf>,

    /// path of the log file to use.
    #[argh(option)]
    pub(crate) log_file: Option<PathBuf>,
}
