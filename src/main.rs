use std::collections::HashSet;
use std::fs;
use std::fmt;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use chrono::prelude::{DateTime, Local};
use fs_extra::dir::get_size;
use walkdir::WalkDir;
use walkdir::Error as WalkDirError;

struct Photo {
    path: PathBuf,
    size: u64,
    m_time: DateTime<Local>,
}

impl fmt::Display for Photo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.path.display(), self.m_time, self.size)
    }
}

fn get_photos(input_path: &Path) -> Result<Vec<Photo>, WalkDirError> {
    let mut photos = vec![];
    let mut caches = HashSet::new();
    for entry in WalkDir::new(input_path) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                return Err(e);
           }
        };
        if entry.file_type().is_dir() {
            continue;
        }
        let entry_extension = match entry.path().extension() {
            Some(e) => e,
            None => {
                continue;
            }
        };
        if entry_extension != "jpg" && entry_extension != "JPG" && 
            entry_extension != "dng" && entry_extension != "DNG" &&
            entry_extension != "nef" && entry_extension != "NEF" {
            continue;
        }
        let mut entry_path = entry.path();
        let parent_path = if let Some(parent_path) = entry_path.parent() {
            parent_path
        } else {
            panic!("Not found parnet path for {}", entry_path.display());
        };

        if parent_path.ends_with("jpg") || parent_path.ends_with("DxO") {
            let parent_path_buf = parent_path.to_path_buf();
            if caches.contains(&parent_path_buf) {
                continue;
            }
            entry_path = parent_path;
        }
        caches.insert(entry_path.to_path_buf());

        let entry_meta = if let Ok(entry_meta) = fs::metadata(&entry_path) {
            entry_meta
        } else {
            panic!("Get metadata for {}", entry_path.display());
        };

        let entry_size: u64;
        if entry_path.is_dir() {
            entry_size = if let Ok(entry_size) = get_size(&entry_path) {
                entry_size
            } else {
                panic!("fs_extra::dir::get_size for {}", entry_path.display());
            };
        } else {
            entry_size = entry_meta.len();
        }

        let entry_mtime = if let Ok(entry_mtime) = entry_meta.modified() {
            entry_mtime
        } else {
            panic!("Get modified time for {}", entry_path.display());
        };

        let photo = Photo {
            path: entry_path.to_path_buf(),
            size: entry_size,
            m_time: entry_mtime.into(),
        };
        photos.push(photo);
    }
    Ok(photos)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("使用法: {} <入力ファイルパス> <出力ファイルパス>", args[0]);
        return;
    }

    let input_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);

    if !input_path.exists() {
        println!("入力ファイルパスが存在しません。");
        return;
    }

    let photos = match get_photos(input_path) {
        Ok(m) => m,
        Err(e) => panic!("{:?}", e),
    };
    for photo in photos {
        let mk_dir = output_path.join(format!(
            "{}/{}-{}-{}",
            photo.m_time.format("%Y"),
            photo.m_time.format("%Y"),
            photo.m_time.format("%m"),
            photo.m_time.format("%d")
        ));
        if !mk_dir.exists() {
            let mk_status = Command::new("mkdir")
                .arg("-p")
                .arg(&mk_dir)
                .status()
                .expect("mkdirコマンドの実行に失敗しました。");
            if !mk_status.success() {
                panic!("mkdirコマンドの実行に失敗したため終了します");
            }
        }
        let cp_path = output_path.join(format!(
            "{}/{}-{}-{}/{}",
            photo.m_time.format("%Y"),
            photo.m_time.format("%Y"),
            photo.m_time.format("%m"),
            photo.m_time.format("%d"),

            // Option<&OsStr> to &str
            photo.path.file_name().unwrap().to_str().unwrap()
        ));
        println!("{} to {}", photo, cp_path.display());

        let status = Command::new("cp")
            .arg("-rp")
            .arg(&photo.path)
            .arg(&cp_path)
            .status()
            .expect("cp コマンドの実行に失敗しました。");

        if !status.success() {
            panic!("cpコマンドの実行に失敗したため終了します")
        }

        let cp_size = if let Ok(cp_size) = get_size(&cp_path) {
            cp_size
        } else {
            panic!("fs_extra::dir::get_size for {}", cp_path.display());
        };
        if cp_size != photo.size {
            panic!("source:{}, to:{} was not eq. cp_path:{}",
                photo.size, cp_size, cp_path.display()
            );
        }
        println!("ok! {} to {}", photo, cp_path.display());
    }
}
