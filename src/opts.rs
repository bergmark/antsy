use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
pub(crate) struct Opts {
    #[structopt(long, default_value = "save.json")]
    pub(crate) save_file: PathBuf,
    #[structopt(long, default_value = "0.25")]
    pub(crate) speed_base: f64,
}
