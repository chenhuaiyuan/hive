use crate::error::Result;
use std::env;
use std::fs;
use std::io::copy;
use std::path::Path;

pub fn create_object(name: String) -> Result<()> {
    let mut current_dir = env::current_dir()?;
    current_dir = current_dir.join(name.clone());
    if current_dir.exists() {
        println!("当前目录已存在{name}项目");
        return Ok(());
    } else {
        println!("start...");
    }
    let file_name = name.clone() + ".zip";
    let mut dloader = downloader::Downloader::builder()
        .parallel_requests(1)
        .build()
        .unwrap();
    let dl = downloader::Download::new(
        "https://github.com/chenhuaiyuan/hive_template/archive/refs/heads/master.zip",
    )
    .file_name(Path::new(&file_name));

    let result = dloader.download(&[dl])?;

    for r in result {
        if r.is_ok() {
            archive(file_name.clone(), Path::new(&name))?;
        } else if let Err(e) = r {
            println!("Error: {e}");
        }
    }
    Ok(())
}

fn archive(file: String, target: &Path) -> Result<()> {
    let current_dir = env::current_dir()?;
    let zip_file = fs::File::open(file.clone())?;
    let mut zip = zip::ZipArchive::new(zip_file)?;

    if !target.exists() {
        fs::create_dir_all(target)?;
    }
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        if file.is_dir() {
            let f = Path::new(file.name());
            let f = f.strip_prefix("hive_template-master/")?;
            if f != Path::new("") {
                println!("create path: {}", f.to_str().unwrap_or(""));
                let target = current_dir.join(target).join(f);
                fs::create_dir_all(target)?;
            }
        } else {
            let f = Path::new(file.name());
            let f = f.strip_prefix("hive_template-master/")?;
            if f != Path::new("") {
                let file_path = current_dir.join(target).join(f);
                let mut target_file = if !file_path.exists() {
                    println!("create file path: {}", file_path.to_str().unwrap());
                    fs::File::create(file_path)?
                } else {
                    fs::File::open(file_path)?
                };
                copy(&mut file, &mut target_file)?;
            }
        }
    }
    fs::remove_file(file)?;
    Ok(())
}
