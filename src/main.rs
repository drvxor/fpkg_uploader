use inquire::{Select, Text};
use reqwest::blocking::multipart;
use reqwest::blocking::Client;
use tar::Builder;
use fpkg_uploader::models::Repository;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = vec!["Set Target Repository", "Upload new", "Exit"];

    let mut target_repository: Option<Repository> = None;

    loop {
        match Select::new("What do you want to do?", options.clone()).prompt()? {
            "Set Target Repository" => {
                let url: String = Text::new("Enter the url for your repository (e.g., http://localhost:8080)").prompt()?;
                let auth_token: String = Text::new("Enter the authentication token for your repository (e.g., 12345678)").prompt()?;

                target_repository = Some(Repository {
                    url,
                    auth_token
                });

                Text::new("Press Enter to continue...")
                    .prompt()?;
            }

            "Upload new" => {
                let repo = match &target_repository {
                    Some(r) => r,
                    None => {
                        println!("Error: You must set a target repository first!");
                        Text::new("Press Enter to continue...").prompt()?;
                        continue;
                    }
                };

                println!("Uploading to repository: {}", repo.url);

                let name: String = Text::new("Enter name of your package").prompt()?;
                let package_dir: String = Text::new("Enter the path to the directory you want to pack (e.g., ./my_package)").prompt()?;
                let version: String = Text::new("Enter version of your package").prompt()?;
                let description: String = Text::new("Enter description for your package").prompt()?;

                let mut source_based = false;
                let mut binary_based = false;

                let source_options = vec!["Source only", "Binary only", "Both"];
                match Select::new("Is your package source based?", source_options).prompt()? {
                    "Source only" => source_based = true,
                    "Binary only" => binary_based = true,
                    "Both" => {
                        source_based = true;
                        binary_based = true
                    },
                    _ => unreachable!(),
                }

                let mut build_cmd: String = String::from("");
                if source_based {
                    build_cmd = Text::new("Enter build command for your package").prompt()?;
                }

                let deps_raw: String = Text::new("Dependencies? (e.g., tokio, serde) [none]").prompt()?;

                let deps: Vec<String> = deps_raw
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                let mut file_bytes = Vec::new();

                {
                    let mut encoder = zstd::Encoder::new(&mut file_bytes, 19)?;
                    encoder.multithread(num_cpus::get() as u32)?;

                    let mut tar_builder = Builder::new(encoder);

                    tar_builder.append_dir_all(".", &package_dir)?;

                    let zstd_encoder = tar_builder.into_inner()?;

                    zstd_encoder.finish()?;
                }

                let upload_filename = format!("{}.tar.zst", name);

                let mut form = multipart::Form::new()
                    .text("name", name)
                    .text("version", version)
                    .text("description", description)
                    .text("source_based", source_based.to_string())
                    .text("binary_based", binary_based.to_string())
                    .text("build_cmd", build_cmd)
                    .part("file", multipart::Part::bytes(file_bytes).file_name(upload_filename));

                for dep in deps {
                    form = form.text("dependencies", dep);
                }

                let client = Client::new();
                let response = client
                    .post(format!("{}/upload", repo.url))
                    .header("x-fpkg-upload-token", &repo.auth_token)
                    .multipart(form)
                    .send()?;

                println!("{}", response.text()?);
            }

            "Exit" => {
                break;
            }

            _ => unreachable!(),
        }
    }

    Text::new("Press Enter to exit...")
        .prompt()?;

    Ok(())
}