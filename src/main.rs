use anyhow::{anyhow, bail, Result};
use reqwest::multipart::{Form, Part};
use reqwest::{Body, Client};
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

#[derive(Serialize, Deserialize)]
struct Payload {
    code: String,
    ext: String,
}

impl std::fmt::Display for Payload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "https://ptpimg.me/{}.{}", self.code, self.ext)
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    let api_key = std::env::var("PTPIMG_KEY").map_err(|_| anyhow!("no PTPIMG_KEY set"))?;
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        bail!("filename required (usage: ptpimg-cli <filepath>...)")
    }

    for arg in &args[1..] {
        let file_path = std::path::Path::new(arg);
        let file_name = file_path
            .file_name()
            .ok_or(anyhow!("could not get file name"))?
            .to_os_string()
            .into_string()
            .ok()
            .ok_or(anyhow!("could not convert file name osstr to string"))?;

        let mime = mime_guess::from_path(file_path)
            .first()
            .ok_or(anyhow!("could not guess mime type of file"))?;

        let file_handle = File::open(file_path).await?;
        let file_body = Body::wrap_stream(FramedRead::new(file_handle, BytesCodec::new()));
        let part = Part::stream(file_body)
            .file_name(file_name)
            .mime_str(&mime.to_string())?;

        let form = Form::new()
            .part("api_key", Part::text(api_key.clone()))
            .part("file-upload[]", part);

        let client = Client::new();
        let result = client
            .post("https://ptpimg.me/upload.php")
            .header("referer", "https://ptpimg.me/index.php")
            .multipart(form)
            .send()
            .await?
            .json::<Vec<Payload>>()
            .await?;

        result.iter().for_each(|url| {
            println!("{url}");
        })
    }

    Ok(())
}
