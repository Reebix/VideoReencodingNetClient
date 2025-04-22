use reqwest::Client;
use std::fs::File;
use std::io::{Cursor, Read};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = "http://192.168.1.102:8000";

    let req = reqwest::get(format!("{base_url}/request")).await?;
    let full_file_name = req.text().await?;
    let file_name = full_file_name
        .replace("\\", "/")
        .split('/')
        .last()
        .unwrap()
        .to_string();

    println!("{} {}", full_file_name, file_name);

    let resp = reqwest::get(format!("{base_url}/files/{full_file_name}")).await?;
    let mut file = File::create(format!("./{file_name}"))?;
    let mut content = Cursor::new(resp.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    println!("File downloaded successfully");

    let client = Client::new();

    // execute ffmpeg command
    /*
         let output = Command::new("ffmpeg")
            .arg("-i")
            .arg(format!("./{file_name}"))
             .arg("-c:v")
            .arg("av1_nvenc")
            .arg("-preset")
            .arg("p4")
             .arg("-cq")
            .arg("40")
             .arg(format!("./{file_name}"))
            .output()
            .expect("Failed to execute command");

        println!("status: {}", output.status);
        io::stdout().write_all(&output.stdout).expect("TODO: panic message");
        io::stderr().write_all(&output.stderr).expect("TODO: panic message");
    */

    // POST-Anfrage
    let mut file = File::open(format!("./{file_name}"))?;
    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content)?;

    let res = client
        .post(format!("{base_url}/converted/{full_file_name}"))
        .body(file_content)
        .send()
        .await?;

    println!("POST Response: {}", res.status());

    std::fs::remove_file(format!("./{file_name}"))?;
    Ok(())
}
