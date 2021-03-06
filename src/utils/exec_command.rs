use super::argument_parsing::Args;
use super::path_utils::Format::{Audio, Video};
use anyhow::{bail, Context, Result};
use std::process::{Command, Stdio};

pub fn run_youtube_dl(args: &Args) -> Result<()> {
    let mut youtube_dl_command = Command::new("youtube-dl");
    let file = args.file();
    let filepath = file.filepath();
    let file_stem = filepath.file_stem().unwrap();
    let file_without_ext = filepath.parent().unwrap();
    let file_without_ext = file_without_ext.join(file_stem);

    let file_format;
    let best_format_string;
    match &file.format() {
        Video(format) => {
            youtube_dl_command.arg("--recode-video");
            best_format_string = "bestvideo+bestaudio";
            file_format = format;
        }
        Audio(format) => {
            youtube_dl_command.arg("-x"); // extract audio
            youtube_dl_command.arg("--audio-format");
            best_format_string = "bestaudio";
            file_format = format;
        }
    };

    let best_format_string = best_format_string.to_owned(); // either "bestaudio" or "bestvideo+bestaudio"
    youtube_dl_command.arg(file_format); // specify final format incase of conversiion. mp3, avi etc..

    // set the quality parameter
    let quality = match args.quality() {
        Some(qual) => qual, // quality is overriden by quality passed in by user
        None => &best_format_string,
    };
    youtube_dl_command.arg(format!("-f {}", quality));

    // specify output filepath in the form /path/to/filestem.%(ext)s
    youtube_dl_command.arg("-o");
    youtube_dl_command.arg(format!("{}.%(ext)s", file_without_ext.to_string_lossy()));

    // if youtube_dl_args were specified add them to command
    if let Some(youtube_dl_args) = args.youtube_dl_args() {
        youtube_dl_command.args(youtube_dl_args);
    }

    // add url to command
    youtube_dl_command.arg(args.url());

    // user specified output to be verbose
    if *args.is_verbose() {
        // each argument in quotes like "youtube-dl" "-f best" "-o \"/Users/Adhu/some.mp3\""...
        let command_as_string = format!("{:?}", youtube_dl_command);
        // removes all the escape charecters and double quotes
        let command_as_string = command_as_string.replace("\"", "").replace("\\'", "\'");
        println!("{}", command_as_string);
    }

    let youtube_dl_status = youtube_dl_command
        .stdout(Stdio::inherit()) // use same stdout as parent
        .stderr(Stdio::inherit()) // use same stderr as parent
        .status()
        .context("could not start youtube-dl")?;

    if !youtube_dl_status.success() {
        bail!("eyeD3 errored out!");
    }
    Ok(())
}

pub fn run_eye_d3(args: &Args) -> Result<()> {
    let metadata_args = args.metadata_args();
    let filepath = args.file().filepath();
    let file_stem = filepath.file_stem().unwrap().to_string_lossy();
    let mut eye_d3_command = Command::new("eyeD3");
    eye_d3_command.arg("-Q"); // enable quiet mode for less output
    eye_d3_command.arg(format!("-t {}", file_stem.replace("_", " "))); // title is constructed by removing all underscores in filestem
    if let Some(metadata_args) = metadata_args {
        if let Some(album) = metadata_args.album() {
            eye_d3_command.arg(format!("-A {}", album)); // user passed in Album to embed
        }
        if let Some(artist) = metadata_args.artist() {
            eye_d3_command.arg(format!("-a {}", artist)); // user passed in artist to embed
        }
    }

    eye_d3_command.arg(filepath); // specify audiofile filepath
    let id3_status = eye_d3_command
        .stderr(Stdio::inherit()) // use same stderr as parent
        .status()
        .context("could not start eye3D")?;

    if !id3_status.success() {
        bail!("eyeD3 errored out!");
    }
    Ok(())
}
