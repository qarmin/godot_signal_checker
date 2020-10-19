use std::collections::BTreeMap;
use std::fs::File;
use std::fs::Metadata;
use std::io::{self, BufRead};
use std::path::Path;
use std::{env, fs, process};

fn main() {
    let mut added_signals: BTreeMap<String, u32> = Default::default();
    let mut emitted_signals: BTreeMap<String, u32> = Default::default();
    let mut connected_signals: BTreeMap<String, u32> = Default::default();
    let mut compat_connected_signals: BTreeMap<String, u32> = Default::default();

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
                if (!file_name.ends_with(".cpp") && !file_name.ends_with(".h")) || file_name.ends_with(".gen.h") {
                    continue;
                }

                if let Ok(file) = File::open(current_folder.to_string() + &file_name) {
                    // Consumes the iterator, returns an (Optional) String
                    for line in io::BufReader::new(file).lines() {
                        if let Ok(ip) = line {
                            if ip.trim().starts_with("//"){
                                continue;
                            }
                            if ip.contains("ADD_SIGNAL(MethodInfo(\"") {
                                let vector: Vec<&str> = ip.split("ADD_SIGNAL(MethodInfo(").collect();
                                let second_vector: Vec<&str> = vector.get(1).unwrap().split('\"').collect();
                                let signal_name = second_vector.get(1).unwrap();

                                let current_value = match added_signals.get(&*signal_name.to_string()) {
                                    Some(t) => *t,
                                    None => 0,
                                };
                                added_signals.insert(signal_name.to_string(), current_value + 1);
                            } else if ip.contains("emit_signal(CoreStringNames::get_singleton()->") {
                                let vector: Vec<&str> =
                                    ip.split("emit_signal(CoreStringNames::get_singleton()->").collect();
                                let second_vector: Vec<&str> = vector.get(1).unwrap().split(',').collect();
                                let third_vector: Vec<&str> = second_vector.get(0).unwrap().split(')').collect();
                                let signal_name = third_vector.get(0).unwrap();

                                let current_value = match emitted_signals.get(&*signal_name.to_string()) {
                                    Some(t) => *t,
                                    None => 0,
                                };
                                emitted_signals.insert(signal_name.to_string(), current_value + 1);
                            } else if ip.contains("emit_signal(SceneStringNames::get_singleton()->") {
                                let vector: Vec<&str> =
                                    ip.split("emit_signal(SceneStringNames::get_singleton()->").collect();
                                let second_vector: Vec<&str> = vector.get(1).unwrap().split(',').collect();
                                let third_vector: Vec<&str> = second_vector.get(0).unwrap().split(')').collect();
                                let signal_name = third_vector.get(0).unwrap();

                                let current_value = match emitted_signals.get(&*signal_name.to_string()) {
                                    Some(t) => *t,
                                    None => 0,
                                };
                                emitted_signals.insert(signal_name.to_string(), current_value + 1);
                            } else if ip.contains("emit_signal(\"") {
                                let vector: Vec<&str> = ip.split('\"').collect();
                                let signal_name = vector.get(1).unwrap();

                                let current_value = match emitted_signals.get(&*signal_name.to_string()) {
                                    Some(t) => *t,
                                    None => 0,
                                };
                                emitted_signals.insert(signal_name.to_string(), current_value + 1);
                            } else if ip.contains("->connect(\"") {
                                let vector: Vec<&str> = ip.split("->connect").collect();
                                let second_vector: Vec<&str> = vector.get(1).unwrap().split('\"').collect();
                                let signal_name = second_vector.get(1).unwrap();

                                let current_value = match connected_signals.get(&*signal_name.to_string()) {
                                    Some(t) => *t,
                                    None => 0,
                                };
                                connected_signals.insert(signal_name.to_string(), current_value + 1);
                            } else if ip.contains("connect_compat(\"") {
                                let vector: Vec<&str> = ip.split('\"').collect();
                                let signal_name = vector.get(1).unwrap();

                                let current_value = match compat_connected_signals.get(&*signal_name.to_string()) {
                                    Some(t) => *t,
                                    None => 0,
                                };
                                compat_connected_signals.insert(signal_name.to_string(), current_value + 1);
                            }
                        }
                    }
                }
            }
        }
    }

    let mut emitted: Vec<String> = Vec::new();
    let mut emitted_connected: Vec<String> = Vec::new();
    let mut emitted_added: Vec<String> = Vec::new();
    let mut added: Vec<String> = Vec::new();
    let mut added_connected: Vec<String> = Vec::new();
    let mut connected: Vec<String> = Vec::new();

    // Checking for unused signals
    for signal in &emitted_signals {
        if !added_signals.contains_key(signal.0) {
            if !connected_signals.contains_key(signal.0) && !connected_signals.contains_key(signal.0) {
                emitted.push(signal.0.clone());
            } else {
                emitted_connected.push(signal.0.clone());
            }
        } else if !connected_signals.contains_key(signal.0) && !connected_signals.contains_key(signal.0) {
            emitted_added.push(signal.0.clone());
        }
    }

    for signal in &added_signals {
        if !emitted_signals.contains_key(signal.0) {
            if !connected_signals.contains_key(signal.0) && !connected_signals.contains_key(signal.0) {
                added.push(signal.0.clone());
            } else {
                added_connected.push(signal.0.clone());
            }
        } else if !connected_signals.contains_key(signal.0) && !connected_signals.contains_key(signal.0) {
            continue; // This was checked above
        }
    }
    let mut new_connected_signals: BTreeMap<String, u32> = Default::default();
    new_connected_signals.extend(connected_signals);
    new_connected_signals.extend(compat_connected_signals);

    for signal in &new_connected_signals {
        if !added_signals.contains_key(signal.0) {
            if !emitted_signals.contains_key(signal.0) {
                connected.push(signal.0.clone());
            } else {
                continue; // This was checked above
            }
        } else if !emitted_signals.contains_key(signal.0) {
            continue; // This was checked above
        }
    }

    println!();
    for i in &emitted {
        println!("Signal {} is emitted but never added or connected", i);
    }
    println!();
    for i in &emitted_connected {
        println!("Signal {} is emitted and connected but never added", i);
    }
    println!();
    for i in &emitted_added {
        println!("Signal {} is emitted and added but never connected, this is just information message which you can ignore.", i);
    }
    println!();
    for i in &added {
        println!("Signal {} is added but never emitted or connected", i);
    }
    println!();
    for i in &added_connected {
        println!("Signal {} is added and connected but never emitted", i);
    }
    println!();
    for i in &connected {
        println!("Signal {} is connected but never added or emitted", i);
    }

    if emitted.len()
        + emitted_connected.len()
        + added.len()
        + added_connected.len()
        + connected.len()
        > 0
    {
        println!("\nFound unused signal, exiting with code 1.");
        process::exit(1);
    }

}
