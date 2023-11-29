use cfg_if::cfg_if;
use std::path::PathBuf;
use thiserror::Error;

cfg_if! {
    if #[cfg(target_os = "windows")] {
        #[doc(hidden)]
        pub mod windows;
        use windows::*;
        type ErrorType = WindowsShortcutError;
    } else if #[cfg(target_os = "linux")] {
        #[doc(hidden)]
        pub mod linux;
        use linux::*;
        type ErrorType = LinuxShortcutError;
    } else if #[cfg(target_os = "macos")] {
        compile_error!("MacOS is not supported yet.");
    }else {
        compile_error!("Unsupported OS");
    }
}
#[derive(Debug, Error)]
pub enum FileShortcutError {
    /// Error creating the shortcut file.
    ///
    /// Caused by something within the native implementation.
    #[error(transparent)]
    NativeError(#[from] ErrorType),
    #[error("The target path does not exist.")]
    TargetPathDoesNotExist(PathBuf),
    #[error("ICON path does not exist.")]
    IconPathDoesNotExist(PathBuf),
    #[error("Working Directory path does not exist.")]
    WorkingDirectoryPathDoesNotExist(PathBuf),
}

/// A builder for creating shortcut files.
///
/// # Example
/// ```
/// use shortcut_rs::shortcut_files::ShortcutFile;
/// let shortcut = ShortcutFile::new("My Shortcut", "C:\\Program Files\\My Program.exe")
/// .description("This is a shortcut to my program.")
/// .arg("--my-argument");
///
/// shortcut.save("C:\\Users\\Me\\Desktop\\My Shortcut.lnk");
///
/// ```

#[derive(Debug, Clone, PartialEq, Hash)]
#[non_exhaustive]
pub struct ShortcutFile {
    /// Name of the shortcut. Ignored on Windows.
    pub name: String,
    /// Description of the shortcut.
    pub description: Option<String>,
    /// Path to executable.
    pub path: PathBuf,
    /// Arguments to pass to the executable.
    pub arguments: Vec<String>,
    /// Path to icon.
    pub icon: Option<PathBuf>,
    /// Working directory of the shortcut.
    pub working_directory: Option<PathBuf>,
    /// Whether to show the terminal or command prompt when running the shortcut
    ///
    /// Defaults to false.
    pub show_terminal: bool,
    /// Categories of the shortcut.
    ///
    /// On Windows, this is ignored.
    pub categories: Vec<String>,
    // TODO: Add support for hotkeys
}

impl Default for ShortcutFile {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: None,
            path: PathBuf::new(),
            arguments: vec![],
            icon: None,
            working_directory: None,
            show_terminal: false,
            categories: vec![],
        }
    }
}
impl ShortcutFile {
    /// Creates a new shortcut file.
    pub fn new(name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            description: None,
            path: path.into(),
            arguments: vec![],
            icon: None,
            show_terminal: false,
            categories: vec![],
            working_directory: None,
        }
    }
    /// Sets the description of the shortcut.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    /// Sets the working directory of the shortcut.
    pub fn working_directory(mut self, working_directory: impl Into<PathBuf>) -> Self {
        self.working_directory = Some(working_directory.into());
        self
    }
    /// Adds an argument to the shortcut.
    pub fn arg(mut self, argument: impl Into<String>) -> Self {
        self.arguments.push(argument.into());
        self
    }
    /// Adds multiple arguments to the shortcut.
    /// # Warning
    /// This will overwrite any existing arguments.
    pub fn arguments(mut self, arguments: Vec<String>) -> Self {
        self.arguments = arguments;
        self
    }
    /// Sets the icon of the shortcut.
    pub fn icon(mut self, icon: impl Into<PathBuf>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    /// Sets the show command of the shortcut.
    pub fn show_terminal(mut self) -> Self {
        self.show_terminal = true;
        self
    }
    /// Adds a category to the shortcut.
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.categories.push(category.into());
        self
    }
    /// Adds multiple categories to the shortcut.
    ///
    /// # Warning
    /// This will overwrite any existing categories.
    pub fn categories(mut self, categories: Vec<String>) -> Self {
        self.categories = categories;
        self
    }
    /// Saves the shortcut to the given path.
    pub fn save(self, to: impl Into<PathBuf>) -> Result<(), FileShortcutError> {
        if !self.path.exists() {
            return Err(FileShortcutError::TargetPathDoesNotExist(self.path));
        }
        if let Some(icon) = &self.icon {
            if !icon.exists() {
                return Err(FileShortcutError::IconPathDoesNotExist(icon.clone()));
            }
        }
        if let Some(working_directory) = &self.working_directory {
            if !working_directory.exists() {
                return Err(FileShortcutError::WorkingDirectoryPathDoesNotExist(
                    working_directory.clone(),
                ));
            }
        }

        save_shortcut_file(self, to.into()).map_err(FileShortcutError::from)
    }
    pub fn read(path: impl Into<PathBuf>) -> Result<Self, FileShortcutError> {
        read_shortcut_file(path.into()).map_err(FileShortcutError::from)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_api() {
        let shortcut = super::ShortcutFile::new("My Shortcut", "C:\\Program Files\\My Program.exe")
            .description("This is a shortcut to my program.")
            .arg("--my-argument")
            .category("My Category");
        assert_eq!(
            shortcut,
            super::ShortcutFile {
                name: "My Shortcut".to_string(),
                description: Some("This is a shortcut to my program.".to_string()),
                path: "C:\\Program Files\\My Program.exe".into(),
                arguments: vec!["--my-argument".to_string()],
                icon: None,
                show_terminal: false,
                categories: vec!["My Category".to_string()],
                working_directory: None,
            }
        );
    }
}
