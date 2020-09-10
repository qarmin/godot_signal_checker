use std::collections::HashMap;
use std::fs::File;
use std::fs::Metadata;
use std::io::{self, BufRead};
use std::path::Path;
use std::{env, fs, process};

fn main() {
    let mut added_signals: HashMap<String, u32> = Default::default();
    let mut emitted_signals: HashMap<String, u32> = Default::default();
    let mut connected_signals: HashMap<String, u32> = Default::default();
    let mut compat_connected_signals: HashMap<String, u32> = Default::default();

    let all_arguments: Vec<String> = env::args().collect();
    if all_arguments.len() < 2 {
        println!("You must provide absolute path when Godot can be found");
        process::exit(1);
    }
    let godot_directory = all_arguments[1].trim_end_matches('/').to_string() + "/";

    if !Path::new(&godot_directory).is_dir() {
        println!("{} isn't proper directory", all_arguments[1]);
        process::exit(1);
    }
    if !Path::new(&(godot_directory.to_string() + "/icon.svg")).exists() {
        println!("{} isn't proper Godot repository", all_arguments[1]);
        process::exit(1);
    }

    let mut folders_to_check: Vec<String> = Vec::new();
    let mut next_folder: String;
    let mut current_folder: String;

    folders_to_check.push(godot_directory);

    while !folders_to_check.is_empty() {
        current_folder = folders_to_check.pop().unwrap();
        println!("{}", current_folder);

        let read_dir = match fs::read_dir(&current_folder) {
            Ok(t) => t,
            _ => continue,
        };
        for entry in read_dir {
            let entry_data = match entry {
                Ok(t) => t,
                Err(_) => continue, //Permissions denied
            };
            let metadata: Metadata = match entry_data.metadata() {
                Ok(t) => t,
                Err(_) => continue, //Permissions denied
            };
            if metadata.is_dir() {
                let folder_name: String = match entry_data.file_name().into_string() {
                    Ok(t) => t,
                    Err(_) => continue, // Permission Denied
                };
                if folder_name == "thirdparty"
                    || folder_name.starts_with('.')
                    || folder_name == "__pycache__"
                    || folder_name == "misc"
                {
                    continue;
                }
                next_folder = "".to_string() + &*current_folder.to_string() + &*folder_name + "/";
                folders_to_check.push(next_folder);
            } else if metadata.is_file() {
                let file_name: String = match entry_data.file_name().into_string() {
                    Ok(t) => t,
                    Err(_) => continue, // Permission Denied
                };
                if (!file_name.ends_with(".cpp") && !file_name.ends_with(".h"))
                    || file_name.ends_with(".gen.h")
                {
                    continue;
                }

                if let Ok(file) = File::open(current_folder.to_string() + &file_name) {
                    // Consumes the iterator, returns an (Optional) String
                    for line in io::BufReader::new(file).lines() {
                        if let Ok(ip) = line {
                            if ip.contains("ADD_SIGNAL(MethodInfo(\"") {
                                let vector: Vec<&str> = ip.split('\"').collect();
                                let signal_name = vector.get(1).unwrap();

                                let current_value =
                                    match added_signals.get(&*signal_name.to_string()) {
                                        Some(t) => *t,
                                        None => 0,
                                    };
                                added_signals.insert(signal_name.to_string(), current_value + 1);
                            } else if ip.contains("emit_signal(SceneStringNames::get_singleton()->")
                                || ip.contains("emit_signal(CoreStringNames::get_singleton()->")
                            {
                                let vector: Vec<&str> = ip.split("::get_singleton()->").collect();
                                let second_vector: Vec<&str> =
                                    vector.get(1).unwrap().split(',').collect();
                                let third_vector: Vec<&str> =
                                    second_vector.get(0).unwrap().split(')').collect();
                                let signal_name = third_vector.get(0).unwrap();

                                let current_value =
                                    match emitted_signals.get(&*signal_name.to_string()) {
                                        Some(t) => *t,
                                        None => 0,
                                    };
                                emitted_signals.insert(signal_name.to_string(), current_value + 1);
                            } else if ip.contains("emit_signal(\"") {
                                let vector: Vec<&str> = ip.split('\"').collect();
                                let signal_name = vector.get(1).unwrap();

                                let current_value =
                                    match emitted_signals.get(&*signal_name.to_string()) {
                                        Some(t) => *t,
                                        None => 0,
                                    };
                                emitted_signals.insert(signal_name.to_string(), current_value + 1);
                            } else if ip.contains("->connect(\"") {
                                let vector: Vec<&str> = ip.split('\"').collect();
                                let signal_name = vector.get(1).unwrap();

                                let current_value =
                                    match connected_signals.get(&*signal_name.to_string()) {
                                        Some(t) => *t,
                                        None => 0,
                                    };
                                connected_signals
                                    .insert(signal_name.to_string(), current_value + 1);
                            } else if ip.contains("connect_compat(\"") {
                                let vector: Vec<&str> = ip.split('\"').collect();
                                let signal_name = vector.get(1).unwrap();

                                let current_value =
                                    match compat_connected_signals.get(&*signal_name.to_string()) {
                                        Some(t) => *t,
                                        None => 0,
                                    };
                                compat_connected_signals
                                    .insert(signal_name.to_string(), current_value + 1);
                            }
                        }
                    }
                }
            }
        }
    }
    println!("Emitted signals - {:?}", emitted_signals);
    println!("Added signals - {:?}", added_signals);
    println!("Connected signals - {:?}", connected_signals);
    println!("Compat Connected signals - {:?}", compat_connected_signals);
}
