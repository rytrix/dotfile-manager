use std::{
    fs::symlink_metadata, io::ErrorKind, os::unix::fs::symlink, path::PathBuf, str::FromStr,
};

use toml::{from_str, Table};

pub struct Manager {
    table: Table,
    src_dir: PathBuf,
    dst_dir: PathBuf,
    dry_run: bool,
}

impl Manager {
    pub fn new(text: &str, dry_run: bool) -> std::io::Result<Manager> {
        let serialized: Table = match from_str(text) {
            Ok(serialized) => serialized,
            Err(error) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    error.to_string(),
                ));
            }
        };

        let src_dir = if let Some(src_dir) = &serialized["src_dir"].as_str() {
            match PathBuf::from_str(src_dir) {
                Ok(src_dir) => canonicalize(src_dir)?,
                Err(error) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        error.to_string(),
                    ));
                }
            }
        } else {
            PathBuf::new()
        };

        let dst_dir = if let Some(dst_dir) = &serialized["dst_dir"].as_str() {
            match PathBuf::from_str(dst_dir) {
                Ok(dst_dir) => canonicalize(dst_dir)?,
                Err(error) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        error.to_string(),
                    ));
                }
            }
        } else {
            PathBuf::new()
        };

        Ok(Manager {
            table: serialized,
            src_dir,
            dst_dir,
            dry_run,
        })
    }

    pub fn list_entries(&self) {
        println!("{}", self.table["title"]);
        for (configuration, value) in self.table.iter() {
            if value.is_table() {
                println!("{}", configuration);
            }
        }
    }

    pub fn list_full_config(&self) {
        println!("{}", self.table["title"]);

        for (configuration, value) in self.table.iter() {
            if let Some(table) = value.as_table() {
                println!("Config {}:", configuration);

                for (dst_org, src_org) in table {
                    let dst: String = match self.full_path(dst_org, false) {
                        Ok(dst) => dst,
                        Err(error) => {
                            println!(
                                "Failed to get full path of {}, error: {}",
                                dst_org,
                                error.to_string()
                            );
                            continue;
                        }
                    };

                    let src: String;
                    if let Some(src_org) = src_org.as_str() {
                        src = match self.full_path(src_org, true) {
                            Ok(src) => src,
                            Err(error) => {
                                println!(
                                    "Failed to get full path of {}, error: {}",
                                    src_org,
                                    error.to_string()
                                );
                                continue;
                            }
                        };
                    } else {
                        println!("Error: not a string");
                        continue;
                    }

                    println!("  original: {} to link: {}", src, dst);
                }
            }
        }
    }

    pub fn deploy_config(&self, config: &str) {
        let value = &self.table[config];

        if let Some(table) = value.as_table() {
            for (dst_org, src_org) in table {
                let dst: String = match self.full_path(dst_org, false) {
                    Ok(dst) => dst,
                    Err(error) => {
                        println!(
                            "Failed to get full path of {}, error: {}",
                            dst_org,
                            error.to_string()
                        );
                        continue;
                    }
                };

                let src: String;
                if let Some(src_org) = src_org.as_str() {
                    src = match self.full_path(src_org, true) {
                        Ok(src) => src,
                        Err(error) => {
                            println!(
                                "Failed to get full path of {}, error: {}",
                                src_org,
                                error.to_string()
                            );
                            continue;
                        }
                    };
                } else {
                    println!("Error: not a string");
                    return;
                }

                if self.dry_run {
                    println!("original: {} to link: {}", src, dst);
                } else {
                    match symlink_or_replace(src.as_str(), dst.as_str()) {
                        Ok(()) => (),
                        Err(error) => {
                            println!("Error: {}", error.to_string());
                            println!("failed original: {}, to link {}", src, dst);
                        }
                    };
                }
            }
        }
    }

    fn full_path(&self, dir: &str, is_src: bool) -> std::io::Result<String> {
        let path = match PathBuf::from_str(dir) {
            Ok(path) => path,
            Err(error) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    error.to_string(),
                ));
            }
        };

        let path = match path.strip_prefix("~") {
            Ok(path) => {
                let path = match std::env::var("HOME") {
                    Ok(home) => home,
                    Err(_error) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "HOME environment variable does not exist",
                        ));
                    }
                } + "/"
                    + path.display().to_string().as_str();

                path
            }
            Err(_error) => {
                if is_src {
                    let mut src = self.src_dir.clone();
                    src.push(dir);
                    src.display().to_string()
                } else {
                    let mut dst = self.dst_dir.clone();
                    dst.push(dir);
                    dst.display().to_string()
                }
            }
        };

        Ok(path)
    }
}

fn symlink_or_replace(original: &str, link: &str) -> std::io::Result<()> {
    match symlink(original, link) {
        Ok(()) => (),
        Err(error) => {
            if error.kind() == ErrorKind::AlreadyExists {
                let meta = symlink_metadata(link)?;
                if meta.is_symlink() {
                    std::fs::remove_file(link)?;
                    symlink(original, link)?;
                } else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::AlreadyExists,
                        "not a symlink",
                    ));
                }
            }
        }
    };

    Ok(())
}

fn canonicalize(path: std::path::PathBuf) -> std::io::Result<PathBuf> {
    let path = match path.strip_prefix("~") {
        Ok(path) => {
            let path = match std::env::var("HOME") {
                Ok(home) => home,
                Err(_error) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "HOME environment variable does not exist",
                    ));
                }
            } + "/"
                + path.display().to_string().as_str();

            std::path::PathBuf::from_str(path.as_str()).unwrap()
        }
        Err(_error) => path,
    };

    Ok(match path.canonicalize() {
        Ok(path) => path,
        Err(error) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "Failed to canonicalize path {} with error {}",
                    path.display().to_string(),
                    error.to_string()
                ),
            ));
        }
    })
}

#[cfg(test)]
mod tests {
    use std::{io::ErrorKind, str::FromStr};

    use super::{canonicalize, symlink_or_replace};

    #[test]
    fn test_symlink() {
        symlink_or_replace("tests/symlink_ex", "tests/symlink_link").unwrap();
    }

    #[test]
    fn test_symlink_not_a_symlink() {
        match symlink_or_replace("tests/symlink_ex", "tests/symlink_ex2") {
            Ok(_) => {
                panic!("expected not a symlink error");
            }
            Err(error) => {
                assert_eq!(error.kind(), ErrorKind::AlreadyExists);
            }
        };
    }

    #[test]
    fn test_canonicalize() {
        let path = std::path::PathBuf::from_str("~/.config").unwrap();
        let path = canonicalize(path).unwrap();

        let mut actual = std::env::var("HOME").unwrap();
        actual.push_str("/.config");
        assert_eq!(path.display().to_string(), actual);
    }

    #[test]
    fn test_canonicalize_2() {
        let path = std::path::PathBuf::from_str("tests/~").unwrap();
        let path = canonicalize(path).unwrap();

        let mut actual = std::env::var("PWD").unwrap();
        actual.push_str("/tests/~");
        assert_eq!(path.display().to_string(), actual);
    }

    #[test]
    fn test_canonicalize_3() {
        let _ = std::fs::create_dir("~tests/");

        let path = std::path::PathBuf::from_str("~tests/").unwrap();
        let path = canonicalize(path).unwrap();

        let mut actual = std::env::var("PWD").unwrap();
        actual.push_str("/~tests");
        assert_eq!(path.display().to_string(), actual);

        std::fs::remove_dir("~tests/").unwrap();
    }
}
