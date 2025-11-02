use clap::Parser;
use reqwest::Client;
use std::fs::File;
use std::io;
use std::io::{Cursor, Read};
use std::process::{Command, Stdio};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Arguments {
    #[arg(short, long, default_value_t = String::from(""))]
    base_url: String,
    #[arg(short, long, default_value_t = false)]
    loop_: bool,
    #[arg(short, long, default_value_t = 1)]
    count: u32,
    #[arg(long, default_value_t = false)]
    hw_accel: bool,
}

fn abspath(p: &str) -> Option<String> {
    let exp_path = shellexpand::full(p).ok()?;
    let can_path = std::fs::canonicalize(exp_path.as_ref()).ok()?;
    can_path.into_os_string().into_string().ok()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut count = 1;
    println!("base_url:port? (e.g. : http://192.168.1.102:8000, leave blank for default):");
    let mut base_url = "http://192.168.1.102:8000";

    let args = Arguments::parse();
    let mut input = String::new();
    if args.base_url != "" {
        base_url = &*args.base_url;
        println!("base_url: {}", base_url);
    } else {
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim();
        if input != "" {
            base_url = input;
            println!("base_url: {}", base_url);
        } else {
            println!("Using default base_url");
        }
    }

    let mut loop_ = true;
    if !Arguments::parse().loop_ {
        println!("loop? (y/N):");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim();
        if input.to_lowercase() != "y" {
            loop_ = false;
            println!("Not looping");
            // count = 1;
            if args.count != 1 {
                count = args.count;
            } else {
                println!("How many files to process? (default 1):");
                let mut input = String::new();

                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read line");
                if input.trim() != "" {
                    count = input.trim().parse::<u32>().unwrap();
                }
            }
        }
    }

    let temp_dir = std::env::temp_dir();
    let temp_dir = temp_dir.to_str().unwrap();

    println!("temp_dir: {}", temp_dir);

    loop {
        for i in 0..count {
            println!("started: {}/{}", i + 1, count);
            let req = reqwest::get(format!("{base_url}/request")).await?;
            let full_file_name = req.text().await?;
            let file_name = full_file_name
                .replace("\\", "/")
                .split('/')
                .last()
                .unwrap()
                .to_string();

            if full_file_name == "" {
                println!("No file to process");
                return Ok(());
            }

            println!("{} {}", full_file_name, file_name);

            let resp = reqwest::get(format!("{base_url}/files/{full_file_name}")).await?;
            let mut file = File::create(format!("{temp_dir}/{file_name}"))?;
            let mut content = Cursor::new(resp.bytes().await?);
            std::io::copy(&mut content, &mut file)?;
            println!("File downloaded successfully");

            let client = Client::new();

            let mut cmd = Command::new("ffmpeg");

            if args.hw_accel {
                cmd.arg("-hwaccel")
                    .arg("cuda")
                    .arg("-hwaccel_output_format")
                    .arg("cuda");
            }

            cmd.arg("-i")
                .arg(abspath(format!("{temp_dir}/{file_name}").as_str()).unwrap())
                .arg("-c:v")
                .arg("av1_nvenc")
                .arg("-preset")
                .arg("p4")
                .arg("-cq")
                .arg("40")
                .arg(
                    abspath(format!("{temp_dir}/{file_name}").as_str())
                        .unwrap()
                        .replace(".mp4", "av.mp4"),
                )
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?
                .wait()
                .expect("ffmpeg command failed");

            // println!("status: {}", output.status);
            // io::stdout()
            //     .write_all(&output.stdout)
            //     .expect("TODO: panic message");
            // io::stderr()
            //     .write_all(&output.stderr)
            //     .expect("TODO: panic message");

            // POST-Anfrage
            let mut file = File::open(format!("{temp_dir}/{file_name}").replace(".mp4", "av.mp4"))?;

            // check if file is not empty
            if file.metadata()?.len() < 1000 {
                println!("File is basically empty");
                return Ok(());
            }
            let mut file_content = Vec::new();
            file.read_to_end(&mut file_content)?;

            let res = client
                .post(format!("{base_url}/converted/{full_file_name}"))
                .body(file_content)
                .send()
                .await?;

            println!("POST Response: {}", res.status());

            std::fs::remove_file(format!("{temp_dir}/{file_name}"))?;
            std::fs::remove_file(format!("{temp_dir}/{file_name}").replace(".mp4", "av.mp4"))?;
        }
        if !loop_ {
            return Ok(());
        }
    }
}
