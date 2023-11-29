use super::ShortcutFile;
use std::{
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

use log::debug;
use thiserror::Error;
#[derive(Debug, Error)]
pub enum LinuxShortcutError {
    #[error(transparent)]
    IOErr(#[from] std::io::Error),
    #[error("Path was not valid UTF-8")]
    PathNotValidUTF8,
    #[error("Missing Value: {0}")]
    MissingValue(&'static str),
}

pub fn save_shortcut_file(
    shortcut: ShortcutFile,
    to: impl AsRef<Path>,
) -> Result<(), LinuxShortcutError> {
    debug!(
        "Creating Shortcut to {:?} at {:?}",
        shortcut.path,
        to.as_ref()
    );
    let ShortcutFile {
        name,
        path,
        icon,
        description,
        arguments,
        working_directory,
        show_terminal,
        categories,
    } = shortcut;
    let file = OpenOptions::new().write(true).create(true).open(to)?;
    let mut writer = std::io::BufWriter::new(file);
    let command = path.to_str().ok_or(LinuxShortcutError::PathNotValidUTF8)?;
    let exec = if !arguments.is_empty() {
        let args = arguments.join(" ");
        format!("Exec={} {}", command, args)
    } else {
        format!("Exec={}", command)
    };
    let working_directory = working_directory
        .map(|v| {
            v.to_str()
                .map(|v| format!("Path={}", v))
                .ok_or(LinuxShortcutError::PathNotValidUTF8)
        })
        .transpose()?;
    let icon = icon
        .map(|v| {
            v.to_str()
                .map(|v| format!("Icon={}", v))
                .ok_or(LinuxShortcutError::PathNotValidUTF8)
        })
        .transpose()?;
    let description = description.map(|v| format!("Comment={}", v));
    let show_terminal = if show_terminal {
        "Terminal=true"
    } else {
        "Terminal=false"
    };
    let categories = if !categories.is_empty() {
        let categories = categories.join(";");
        Some(format!("Categories={};", categories))
    } else {
        None
    };
    writeln!(writer, "[Desktop Entry]")?;
    writeln!(writer, "Type=Application")?;
    writeln!(writer, "Name={}", name)?;
    writeln!(writer, "{}", exec)?;
    if let Some(working_directory) = working_directory {
        writeln!(writer, "{}", working_directory)?;
    }
    if let Some(icon) = icon {
        writeln!(writer, "{}", icon)?;
    }
    if let Some(description) = description {
        writeln!(writer, "{}", description)?;
    }
    writeln!(writer, "{}", show_terminal)?;
    if let Some(categories) = categories {
        writeln!(writer, "{}", categories)?;
    }
    writer.flush()?;
    Ok(())
}
pub fn read_shortcut_file(path: impl AsRef<Path>) -> Result<ShortcutFile, LinuxShortcutError> {
    let read = std::fs::read_to_string(path)?;
    let mut name = None;
    let mut path = None;
    let mut icon = None;
    let mut description = None;
    let mut arguments = None;
    let mut working_directory = None;
    let mut show_terminal = false;
    let mut categories = None;

    for line in read.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        let mut split = line.splitn(2, '=');
        let key = split.next().unwrap();
        let value = split.next().unwrap();
        match key {
            "Name" => name = Some(value.to_string()),
            "Path" => {
                working_directory = Some(PathBuf::from(value));
            }
            "Icon" => {
                icon = Some(PathBuf::from(value));
            }
            "Comment" => {
                description = Some(value.to_string());
            }
            "Exec" => {
                let mut split = value.split(" ");
                let command = split.next().unwrap();
                path = Some(PathBuf::from(command));
                arguments = Some(split.map(|v| v.to_owned()).collect());
            }
            "Terminal" => {
                show_terminal = value == "true";
            }
            "Categories" => {
                categories = Some(value.split(';').map(|v| v.to_string()).collect());
            }
            _ => {}
        }
    }
    let shortcut = ShortcutFile {
        name: name.ok_or(LinuxShortcutError::MissingValue("Name"))?,
        path: path.ok_or(LinuxShortcutError::MissingValue("Path"))?,
        icon,
        description,
        arguments: arguments.unwrap_or_default(),
        working_directory,
        show_terminal,
        categories: categories.unwrap_or_default(),
    };
    Ok(shortcut)
}
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::shortcut_files::{linux::save_shortcut_file, ShortcutFile};

    use super::read_shortcut_file;

    #[test]
    fn test_save_shortcut_file() {
        let shortcut = ShortcutFile {
            name: "Test".to_string(),
            path: PathBuf::from("/usr/bin/ls"),
            icon: Some(PathBuf::from("/usr/share/icons/ls.png")),
            description: Some("This is a test shortcut".to_string()),
            arguments: vec!["-l".to_string()],
            working_directory: None,
            show_terminal: false,
            categories: vec!["Utility".to_string(), "System".to_string()],
        };
        let path = PathBuf::from("test.desktop");
        save_shortcut_file(shortcut.clone(), &path).unwrap();
        let content = read_shortcut_file(path).unwrap();
        assert_eq!(shortcut, content);
    }
}
