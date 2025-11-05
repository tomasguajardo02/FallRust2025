use serde::Deserialize;
use std::fs::File;
use std::io::copy;

#[derive(Debug, Deserialize)]
struct DogImage {
    message: String,
}

#[derive(Debug)]
enum DogApiResult {
    Ok(String), // Contains the image URL
    Error(String),
}

fn fetch_dog_image_url() -> DogApiResult {
    let url = "https://dog.ceo/api/breeds/image/random";
    let response = ureq::get(url).call();

    match response {
        Ok(resp) => match resp.into_json::<DogImage>() {
            Ok(json) => DogApiResult::Ok(json.message),
            Err(e) => DogApiResult::Error(format!("JSON error: {}", e)),
        },
        Err(e) => DogApiResult::Error(format!("Request error: {}", e)),
    }
}

fn download_image(url: &str, file_name: &str) -> Result<(), String> {
    let response = ureq::get(url).call();

    match response {
        Ok(resp) => {
            let mut reader = resp.into_reader();
            let mut file = File::create(file_name).map_err(|e| e.to_string())?;
            copy(&mut reader, &mut file).map_err(|e| e.to_string())?;
            Ok(())
        }
        Err(e) => Err(format!("Download failed: {}", e)),
    }
}

fn main() {
    println!("Downloading 5 random dog images...\n");

    for i in 1..=5 {
        println!("Fetching image {}", i);

        match fetch_dog_image_url() {
            DogApiResult::Ok(url) => {
                let file_name = format!("dog_{}.jpg", i);
                match download_image(&url, &file_name) {
                    Ok(_) => println!("✅ Saved as {}\n", file_name),
                    Err(e) => println!("❌ Download error: {}\n", e),
                }
            }
            DogApiResult::Error(e) => println!("❌ API Error: {}\n", e),
        }
    }

    println!("Done!");
}
