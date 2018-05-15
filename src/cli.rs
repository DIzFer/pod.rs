use clap::{Arg, App};

pub fn build_cli() -> App<'static, 'static> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::with_name("pretend")
            .short("p")
            .long("pretend")
            .help(
                "Don't actually download podcasts (but log them as downloaded in poddb anyway)"
                )
            )
        .arg(Arg::with_name("list")
             .short("l")
             .long("list")
             .visible_aliases(&["config", "c"])
             .required(true)
             .takes_value(true)
             .help(
                 "File which contains the target directory and the list of subscriptions"
                 )
            )
        .arg(Arg::with_name("db")
             .short("d")
             .long("poddb")
             .visible_aliases(&["db", "cache"])
             .required(true)
             .takes_value(true)
             .help(
                 "Path to the file where we will store the list of downloaded episodes (so they're not downloaded again)"
                 )
             )
}
