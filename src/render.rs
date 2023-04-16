use std::io::prelude::*;
use std::io::BufReader;

use crate::cmd::SmCommand;
use crate::vapoursynth::output::{output, OutputParameters};
use regex::Regex;
use rustsynth::{
    api::{CoreCreationFlags, API},
    core::CoreRef,
    map::OwnedMap,
    node::Node,
};
use std::io::Cursor;
use std::path::PathBuf;
use std::process::{ChildStderr, Command, Stdio};

use crate::{ffpb2, verb};
use std::env;

//use crate::ffpb::ffmpeg;
use indicatif::{ProgressBar, ProgressStyle};
// use crate::ffpb::ffmpeg;

pub fn _teres_render(commands: Vec<SmCommand>) {
    for cmd in commands {
        let vspipe = Command::new(cmd.vs_path)
            .args(cmd.vs_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start vspipe process");

        let ffmpeg = Command::new(cmd.ff_path)
            .args(cmd.ff_args)
            .stdin(Stdio::from(
                vspipe.stdout.expect("Failed to open vspipe stdout"),
            ))
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start ffmpeg process");

        // dbg!("Spawned subprocesses");

        let progress = ProgressBar::new(100);
        progress.set_style(
            ProgressStyle::default_bar()
                .template(
                    format!(
                        " [{}] {{wide_bar:.cyan/blue}} {{percent}}% | ETA: {{eta_precise}}",
                        cmd.payload.basename
                    )
                    .as_str(),
                )
                .unwrap(),
        );

        _teres_progress(vspipe.stderr.unwrap(), progress);

        dbg!(ffmpeg.wait_with_output().unwrap().status);
    }
}

fn _teres_progress(stderr: ChildStderr, progress: ProgressBar) {
    let mut read_frames = false;
    let frame_regex = Regex::new(r"Frame: (?P<current>\d+)/(?P<total>\d+)").unwrap();
    let output_regex = Regex::new(r"Output").unwrap();
    let mut buf = BufReader::new(stderr);

    loop {
        let mut byte_vec = vec![];
        buf.read_until(b'\r', &mut byte_vec).expect("stderr Error");
        let string = String::from_utf8_lossy(&byte_vec);
        if output_regex.is_match(&string) {
            break;
        }
        let caps;
        if frame_regex.is_match(&string) {
            caps = frame_regex.captures(&string).unwrap();
            if !read_frames {
                progress.set_length(caps["total"].parse::<u64>().unwrap());
                read_frames = true
            }
            progress.set_position(caps["current"].parse::<u64>().unwrap())
        }
    }
}

pub fn vspipe_render(commands: Vec<SmCommand>) {
    for cmd in commands {
        let previewing: bool =
            cmd.recipe.get_bool("preview window", "enabled") && cmd.ffplay_args.is_some();

        verb!("FF args: {}", cmd.ff_args.join(" "));

        if previewing {
            verb!(
                "FFplay args: {}",
                &cmd.ffplay_args.clone().unwrap().join(" ")
            );
        }

        let mut vs = Command::new(cmd.vs_path)
            .args(cmd.vs_args)
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed in spawning FFmpeg child");

        let pipe = vs.stdout.take().expect("Failed piping out of VSPipe");

        // if ffmpeg(cmd.ff_args, pipe).is_ok() {
        //     println!("okie we good");
        // } else {
        //     panic!("Failed rendering");
        // }

        let mut ffmpeg = Command::new(cmd.ff_path)
            .args(cmd.ff_args)
            .stdin(pipe)
            .stdout(if previewing {
                Stdio::piped()
            } else {
                Stdio::inherit()
            })
            .spawn()
            .expect("Failed in spawning FFmpeg child");

        if previewing {
            let ffplay_pipe = ffmpeg.stdout.take().expect("Failed piping out of FFmpeg");

            let ffplay = Command::new(cmd.ffplay_path.unwrap())
                .args(cmd.ffplay_args.unwrap())
                .stdin(ffplay_pipe)
                .spawn()
                .expect("Failed in spawning ffplay child");

            ffplay.wait_with_output().unwrap();
        }

        vs.wait_with_output().unwrap();
        ffmpeg.wait_with_output().unwrap();
    }
}

pub fn _vpipe_render2(commands: Vec<SmCommand>) {
    for cmd in commands {
        let previewing: bool =
            cmd.recipe.get_bool("preview window", "enabled") && cmd.ffplay_args.is_some();

        verb!("FF args: {}", cmd.ff_args.join(" "));

        if previewing {
            verb!(
                "FFplay args: {}",
                &cmd.ffplay_args.clone().unwrap().join(" ")
            );
        }

        if true {
            let mut vs = Command::new(cmd.vs_path)
                .args(cmd.vs_args)
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed in spawning FFmpeg child");

            let pipe = vs.stdout.take().expect("Failed piping out of VSPipe");

            let mut ffmpeg = Command::new(cmd.ff_path)
                .args(cmd.ff_args)
                .stdin(pipe)
                .stdout(if previewing {
                    Stdio::piped()
                } else {
                    Stdio::inherit()
                })
                .spawn()
                .expect("Failed in spawning FFmpeg child");

            if previewing {
                let ffplay_pipe = ffmpeg.stdout.take().expect("Failed piping out of FFmpeg");
                let ffplay = Command::new(cmd.ffplay_path.unwrap())
                    .args(cmd.ffplay_args.unwrap())
                    .stdin(ffplay_pipe)
                    .spawn()
                    .expect("Failed in spawning ffplay child");
                ffplay.wait_with_output().unwrap();
            }

            vs.wait_with_output().unwrap();
            ffmpeg.wait_with_output().unwrap();
        } else {
            let mut vs = Command::new(cmd.vs_path)
                .args(cmd.vs_args)
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed in spawning FFmpeg child");

            let pipe = vs.stdout.take().expect("Failed piping out of VSPipe");

            let mut ffmpeg = Command::new(cmd.ff_path)
                .args(cmd.ff_args)
                .stdin(pipe)
                .stdout(if previewing {
                    Stdio::piped()
                } else {
                    Stdio::inherit()
                })
                .stderr(std::process::Stdio::piped())
                .spawn()
                .expect("Failed in spawning FFmpeg child");

            let ff_stats = ffmpeg.stderr.take().expect("Failed capturing FFmpeg");

            ffpb2::ffmpeg2(ff_stats).expect("Failed rendering ffmpeg");

            vs.wait_with_output().expect("failed waiting VapourSynth");
            ffmpeg.wait_with_output().expect("failed waiting ffmpeg");

            if previewing {
                // let ffplay_pipe = ffmpeg.stdout.take().expect("Failed piping out of FFmpeg");
                // let ffplay = Command::new(cmd.ffplay_path.unwrap())
                //     .args(cmd.ffplay_args.unwrap())
                //     .stdin(ffplay_pipe)
                //     .spawn()
                //     .expect("Failed in spawning ffplay child");
                // ffplay.wait_with_output().unwrap();
            }
        }
    }
}

fn libav_smashsource(filepath: PathBuf, core: CoreRef, api: API) -> Node {
    let lsmas = core.plugin_by_namespace("lsmas").unwrap();

    let mut in_args = OwnedMap::new(api);
    in_args
        .set_data(
            "source",
            filepath
                .display()
                .to_string()
                .replace("\\\\?\\", "")
                .as_bytes(),
        )
        .expect("Failed setting input source parameter");
    let map = lsmas.invoke("LWLibavSource", &in_args);

    map.get("clip")
        .expect("Failed getting clip from LWLibavSource")
}

pub fn api_render(commands: Vec<SmCommand>) {
    let api = API::get().unwrap();
    let core = api.create_core(CoreCreationFlags::NONE);

    for cmd in commands {
        let clip = libav_smashsource(cmd.payload.in_path, core, api);

        let num_frames = clip.video_info().unwrap().num_frames as usize;

        let mut buf: Cursor<Vec<u8>> = Cursor::new(Vec::new());

        output(
            &mut buf,
            None,
            OutputParameters {
                y4m: true,
                node: clip,
                start_frame: 0,
                end_frame: num_frames - 1,
                requests: core.info().num_threads,
            },
        )
        .expect("Failed outputting with output");
    }
}
