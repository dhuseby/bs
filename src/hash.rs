use hex;
use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Sha512Trunc256, Digest};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufReader, BufRead};
use std::path::PathBuf;

enum Jobs<'a> {
    Classify(Vec<PathBuf>),
    Scan(PathBuf),
    Digest(PathBuf, Box<dyn BufRead + 'a>),
}

pub struct Hash {
    pub path: PathBuf,
    pub hash: [u8; 32]
}

impl Hash {
    pub fn new(p: &PathBuf, h: &[u8]) -> Self {
        let mut hash = Hash { path: p.to_path_buf(), hash: [0u8; 32] };
        hash.hash.copy_from_slice(h);
        hash
    }
}

pub fn hash(paths: Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    // create the resutl vector
    let mut results = Vec::new();

    // create the queue
    let mut q = VecDeque::<Jobs>::with_capacity(1);

    // set up the progress bar
    let mut total: u64 = 0;
    let pb = ProgressBar::new(total);
    pb.set_style(ProgressStyle::default_bar()
        .template("[ETA: {eta_precise}] [{bar}] {pos:>}/{len:} {wide_msg}")
        .progress_chars("=>-"));

    // add the first job
    q.push_back(Jobs::Classify(paths));

    // process all jobs
    while !q.is_empty() {
        let job = q.pop_front().unwrap();
        match job {
            Jobs::Classify(paths) => {
                for p in paths {
                    if let Ok(meta) = p.symlink_metadata() {
                        if meta.is_file() {
							if let Ok(file) = File::open(&p) {
								q.push_back(Jobs::Digest(p.to_path_buf(), Box::new(BufReader::new(file))));
                                total += 1;
                            }
                        } else if meta.is_dir() {
                            q.push_back(Jobs::Scan(p.to_path_buf()));
                        }
                    } else if p.to_str().unwrap() == "-" {
                        q.push_back(Jobs::Digest(PathBuf::from("stdin"), Box::new(BufReader::new(io::stdin()))));
                        total += 1;
                    }
                }
                pb.set_length(total);
            },
            Jobs::Scan(dir) => {
                pb.set_message(&format!("Scan: {}", dir.to_str().unwrap()));
                let dir_iter = dir.read_dir().expect(&format!("read_dir failed: {:?}", dir));
                q.push_back(Jobs::Classify(dir_iter.map(|res| res.unwrap().path()).collect()));
            },
            Jobs::Digest(path, mut reader) => {
                pb.inc(1);
                pb.set_message(&format!("Hash: {}", path.to_str().unwrap()));
                let mut hasher = Sha512Trunc256::new();
                'digest: loop {
                    let len = {
                        let buf = reader.fill_buf().unwrap();
                        hasher.input(buf);
                        buf.len()
                    };
                    if len == 0 {
                        break 'digest;
                    }
                    reader.consume(len);
                }
                results.push(Hash::new(&path, hasher.result().as_slice()));
            }
        }
    }
    pb.set_message("Done...");
    pb.finish();

    for h in results {
        println!("{}: {}", h.path.to_str().unwrap(), hex::encode(&h.hash));
    }

    Ok(())
}

