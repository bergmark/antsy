use structopt::StructOpt;

use crate::controls::UiStates;

#[derive(StructOpt)]
pub(crate) struct Opts {
    #[structopt(long)]
    pub(crate) new_save: bool,
    #[structopt(long, default_value = "save.json")]
    pub(crate) save_file: String,
    #[structopt(long, default_value = "0.25")]
    pub(crate) speed_base: f64,
    #[structopt(long)]
    pub(crate) start_state: Option<UiStates>,
}
