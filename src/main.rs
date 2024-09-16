use indicatif::{ProgressBar, ProgressStyle};
use clap::{Arg, Command};
use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE};
use reqwest::Client;
use std::fs::File;
use std::io::{self, Write};
use std::error::Error;
use url::Url;
use console::style;
use tokio::runtime::Builder;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("Rget")
        .version("0.1.0")
        .author("Rohit Joshi <bhdream000@gmail.com>")
        .about("wget clone written in Rust")
        .arg(Arg::new("URL")
                 .required(true)
                 .index(1)
                 .help("URL to download"))
        .get_matches();
    let url = matches.get_one::<String>("URL").unwrap();

    // Create a new runtime to execute the async function
    let rt = Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(download(url, false))?;
    Ok(())
}

fn create_progress_bar(quiet_mode: bool, msg: &str, length: Option<u64>) -> ProgressBar {
    let bar = if quiet_mode {
        ProgressBar::hidden()
    } else {
        match length {
            Some(len) => ProgressBar::new(len),
            None => ProgressBar::new_spinner(),
        }
    };

    bar.set_message(msg.to_string());
    if length.is_some() {
        bar.set_style(ProgressStyle::default_bar()
            .template("{msg} {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} eta: {eta}")
            .unwrap()
            .progress_chars("=> "));
    } else {
        bar.set_style(ProgressStyle::default_spinner());
    }

    bar
}

async fn download(target: &str, quiet_mode: bool) -> Result<(), Box<dyn Error>> {
    let url = Url::parse(target)?;
    let client = Client::new();
    let mut resp = client.get(url).send().await?;

    println!("{}", format!("HTTP request sent... {}", style(format!("{}", resp.status())).green()));

    if resp.status().is_success() {
        let headers = resp.headers().clone();
        let ct_len = headers.get(CONTENT_LENGTH).and_then(|val| val.to_str().ok()?.parse().ok());
        let ct_type = headers.get(CONTENT_TYPE).and_then(|val| val.to_str().ok());

        match ct_len {
            Some(len) => {
                println!("Length: {} ({})", style(len).green(), style(format!("{}", len)).red());
            },
            None => {
                println!("Length: {}", style("unknown").red());
            },
        }

        if let Some(ct_type) = ct_type {
            println!("Type: {}", style(ct_type).green());
        }

        let fname = target.split('/').last().unwrap_or("downloaded_file");
        println!("Saving to: {}", style(fname).green());

        let mut buf = Vec::new();
        let bar = create_progress_bar(quiet_mode, fname, ct_len);

        while let Some(chunk) = resp.chunk().await? {
            buf.extend_from_slice(&chunk);
            bar.inc(chunk.len() as u64);
        }

        bar.finish();
        save_to_file(&buf, fname)?;
    }

    Ok(())
}

fn save_to_file(buf: &[u8], fname: &str) -> io::Result<()> {
    let mut file = File::create(fname)?;
    file.write_all(buf)?;
    Ok(())
}
