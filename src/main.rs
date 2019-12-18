extern crate clap;
use rusqlite::{params, Connection, Result};
use time::Timespec;
use clap::{Arg, App, SubCommand};
use blake2::{Blake2b, Digest};
use std::fs;
use std::path::Path;
use std::io;

#[derive(Debug)]
struct FileInfo {
    id: i32,
    hash: String,
    size: String, // KB
    password: String,
    time_created: Timespec,
}

impl FileInfo {
    fn new(file_name: &str, password: &str) -> std::io::Result<FileInfo> {
        let path = Path::new(file_name);
        let metadata = fs::metadata(path)?;
        let file_bin_content = fs::read(path)?;
        let mut hasher = Blake2b::new();
        hasher.input(file_bin_content);
        let hash = hasher.result();
        let hash_str = format!("{:x}", hash);
        Ok(FileInfo {
            id: 0,
            hash: hash_str.to_string(),
            size: metadata.len().to_string(),
            password: password.to_string(),
            time_created: time::get_time(),
        })
    }
}

fn main() -> Result<()> {
    let conn = Connection::open("db.db3")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS file (
            id INTEGER PRIMARY KEY,
            hash TEXT NOT NULL,
            size TEXT NOT NULL,
            password TEXT NOT NULL,
            time_created TEXT NOT NULL
            )",
        params![],
    )?;
    let matcher = App::new("Compression File Password Recorder")
                        .version("1.0")
                        .author("GoldenMean58")
                        .about("Record a compression file's password")
                        .subcommand(SubCommand::with_name("query")
                            .about("Query the saved password of a compression file")
                            .version("1.0")
                            .author("GoldenMean58 <lishuxiang@cug.edu.cn>")
                            .arg(Arg::with_name("file")
                                .short("f")
                                .long("file")
                                .help("The path of the compression file")
                                .value_name("FILE")
                                .takes_value(true)))
                        .subcommand(SubCommand::with_name("save")
                            .about("Save the compression file's password")
                            .version("1.0")
                            .author("GoldenMean58 <lishuxiang@cug.edu.cn>")
                            .arg(Arg::with_name("file")
                                .short("f")
                                .long("file")
                                .help("The path of the compression file")
                                .value_name("FILE")
                                .takes_value(true)))
                        .get_matches();
    if let Some(matches) = matcher.subcommand_matches("query") {
        if matches.is_present("file") {
            let file_name = matches.value_of("file").unwrap();
            match FileInfo::new(file_name, "") {
                Ok(file) => {
                    let mut stmt = conn.prepare("SELECT id, hash, size, password, time_created FROM file WHERE hash = ? and size = ?")?;
                    let file_iter = stmt.query_map(params![file.hash, file.size], |row| {
                        Ok(FileInfo {
                            id: row.get(0).unwrap(),
                            hash: row.get(1).unwrap(),
                            size: row.get(2).unwrap(),
                            password: row.get(3).unwrap(),
                            time_created: row.get(4).unwrap(),
                        })
                    })?;
                    let mut is_exists = false;
                    for file in file_iter {
                        println!("Password:\n{}", file.unwrap().password);
                        is_exists = true;
                        break;
                    }
                    if !is_exists 
                    {
                        println!("No record for this file!");
                    }
                },
                _ => {
                    println!("Cannot get the information of the file!");
                }
            };
        }
    }
    if let Some(matches) = matcher.subcommand_matches("save") {
        if matches.is_present("file") {
            let file_name = matches.value_of("file").unwrap();
            let mut password = String::new();
            println!("Password: ");
            io::stdin().read_line(&mut password).unwrap();
            match FileInfo::new(file_name, &password){
                Ok(file) => {
                    conn.execute("INSERT INTO file (hash, size, password, time_created) VALUES (?1, ?2, ?3, ?4)",
                                params![file.hash, file.size, file.password, file.time_created],)?;
                },
                _ => {
                    println!("Cannot get the information of the file!");
                }
            };
        }
    }
    Ok(())
}
