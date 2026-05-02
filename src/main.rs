use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

const NOTES_FILE: &str = "notes.json";

#[derive(Debug, Serialize, Deserialize)]
struct Note {
    id: u64,
    title: String,
    content: String,
    created_at: u64,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    let result = match args[1].as_str() {
        "add" => add_note(&args),
        "list" => list_notes(),
        "view" => view_note(&args),
        "delete" => delete_note(&args),
        "update" => update_note(&args),
        "help" | "-h" | "--help" => {
            print_usage();
            Ok(())
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
            Err("invalid command".to_string())
        }
    };

    if let Err(error) = result {
        eprintln!("Error: {}", error);
        process::exit(1);
    }
}

fn print_usage() {
    println!("Basic Note Taking App");
    println!();
    println!("Usage:");
    println!("  cargo run -- add <title> <content>");
    println!("  cargo run -- list");
    println!("  cargo run -- view <id>");
    println!("  cargo run -- delete <id>");
    println!("  cargo run -- update <id> <title> <content>");
}

fn add_note(args: &[String]) -> Result<(), String> {
    if args.len() < 4 {
        return Err("add requires <title> <content>".to_string());
    }

    let title = args[2].trim().to_string();
    let content = args[3..].join(" ").trim().to_string();

    if title.is_empty() {
        return Err("title cannot be empty".to_string());
    }
    if content.is_empty() {
        return Err("content cannot be empty".to_string());
    }

    let mut notes = load_notes()?;
    let next_id = notes.iter().map(|n| n.id).max().unwrap_or(0) + 1;
    let created_at = current_timestamp()?;

    let note = Note {
        id: next_id,
        title,
        content,
        created_at,
    };
    notes.push(note);
    save_notes(&notes)?;

    println!("Added note with id {}", next_id);
    Ok(())
}

fn list_notes() -> Result<(), String> {
    let notes = load_notes()?;

    if notes.is_empty() {
        println!("No notes yet.");
        return Ok(());
    }

    println!("Notes:");
    for note in &notes {
        println!("  {}: {}", note.id, note.title);
    }
    Ok(())
}

fn view_note(args: &[String]) -> Result<(), String> {
    if args.len() != 3 {
        return Err("view requires <id>".to_string());
    }

    let id = parse_id(&args[2])?;
    let notes = load_notes()?;

    let note = notes
        .iter()
        .find(|n| n.id == id)
        .ok_or_else(|| format!("note {} not found", id))?;

    println!("Id: {}", note.id);
    println!("Title: {}", note.title);
    println!("Created: {}", note.created_at);
    println!("Content:");
    println!("{}", note.content);
    Ok(())
}

fn delete_note(args: &[String]) -> Result<(), String> {
    if args.len() != 3 {
        return Err("delete requires <id>".to_string());
    }

    let id = parse_id(&args[2])?;
    let mut notes = load_notes()?;
    let before = notes.len();

    notes.retain(|n| n.id != id);
    if notes.len() == before {
        return Err(format!("note {} not found", id));
    }

    save_notes(&notes)?;
    println!("Deleted note {}", id);
    Ok(())
}

fn update_note(args : &[String]) -> Result<(), String>{
    if args.len() < 5 {
        return Err("Update requires <id> <new name> <description>".to_string());
    }
    let id = parse_id(&args[2])?;
    let mut notes = load_notes()?;

    let note = notes.iter_mut().find(|n| n.id == id).ok_or_else(|| "Error id not found".to_string())?;

    let title : String = args[3].trim().to_string();

    let content : String = args[4..].join(" ").trim().to_string();

    note.title = title;
    note.content = content;
    note.created_at = current_timestamp()?;

    save_notes(&notes)?;

    println!("Note with Id {} was updated",id);

    Ok(())

}

fn parse_id(input: &str) -> Result<u64, String> {
    input
        .parse::<u64>()
        .map_err(|_| format!("invalid id '{}'", input))
}

fn load_notes() -> Result<Vec<Note>, String> {
    if !Path::new(NOTES_FILE).exists() {
        return Ok(Vec::new());
    }

    let content =
        fs::read_to_string(NOTES_FILE).map_err(|e| format!("failed to read notes file: {}", e))?;
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_str::<Vec<Note>>(&content)
        .map_err(|e| format!("failed to parse notes file: {}", e))
}

fn save_notes(notes: &[Note]) -> Result<(), String> {
    let json = serde_json::to_string_pretty(notes)
        .map_err(|e| format!("failed to serialize notes: {}", e))?;
    fs::write(NOTES_FILE, json).map_err(|e| format!("failed to write notes file: {}", e))
}

fn current_timestamp() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|e| format!("system time error: {}", e))
}
