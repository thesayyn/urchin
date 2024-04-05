use clap::Parser;
use mac_notification_sys::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    title: String,

    #[arg(long)]
    subtitle: Option<String>,

    #[arg(long)]
    message: String,
}

fn main() {
    let args = Args::parse();

    set_application("notify.com.bazel").unwrap();

    send_notification(
        &args.title,
        args.subtitle.as_ref().map(|t| &**t),
        &args.message,
        Some(&Notification::new().asynchronous(true).sound("Blow").close_button("Close")),
    )
    .unwrap();

    println!("success")
}
