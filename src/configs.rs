//! Common Unix/Linux CLI tool configuration file locations.
//!
//! Each entry is a *relative* path anchored to a `Base` (home dir, the XDG
//! config dir, the XDG data dir, or `/etc`). Resolving an entry combines the
//! base with the relative path to give you a concrete `PathBuf`.
//!
//! Dependency: add the `dirs` crate (`cargo add dirs`).
//!
//! Quick usage:
//! ```no_run
//! # use config_files::{CONFIG_FILES, Base};
//! // Iterate everything:
//! for cfg in CONFIG_FILES {
//!     println!("{:<12} {:?}  {}", cfg.tool, cfg.base, cfg.path);
//! }
//!
//! // Predicate "contains" (idiomatic form):
//! let known = CONFIG_FILES.iter().any(|c| c.path == "fish/config.fish");
//!
//! // Resolve to a real path and check if it exists on disk:
//! if let Some(p) = CONFIG_FILES.iter().find(|c| c.tool == "git" && c.base == Base::Home) {
//!     if let Some(path) = p.resolve() {
//!         println!("git config would be at: {}", path.display());
//!     }
//! }
//! ```

use std::env;
use std::path::PathBuf;
use strum::Display;

/// Where a config file's relative path is anchored.
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum Base {
    /// Relative to the user's home directory (`$HOME`). Legacy dotfiles.
    #[strum(serialize = "~")]
    Home,
    /// Relative to `$XDG_CONFIG_HOME` (defaults to `~/.config`).
    #[strum(serialize = "~/.config")]
    XdgConfig,
    /// Relative to `$XDG_DATA_HOME` (defaults to `~/.local/share`).
    #[strum(serialize = "~/.local/share")]
    XdgData,
    /// Relative to the system config dir (`/etc`).
    #[strum(serialize = "/etc")]
    Etc,
}

/// A single known configuration file for a CLI tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub struct ConfigFile {
    /// The tool this config belongs to (e.g. "git", "nvim").
    pub tool: &'static str,
    /// The root this path is relative to.
    pub base: Base,
    /// Path relative to `base` (e.g. ".gitconfig", "nvim/init.lua").
    pub path: &'static str,
}

impl ConfigFile {
    /// Resolve this entry to an absolute path. Returns `None` only if the home
    /// directory can't be determined (rare).
    ///
    /// Note: XDG dirs are resolved manually from env vars (rather than via
    /// `dirs::config_dir()`) so that on macOS this still points at `~/.config`
    /// where CLI tools actually keep their configs.
    pub fn resolve(&self) -> Option<PathBuf> {
        let root = match self.base {
            Base::Home => dirs::home_dir()?,
            Base::XdgConfig => xdg_dir("XDG_CONFIG_HOME", ".config")?,
            Base::XdgData => xdg_dir("XDG_DATA_HOME", ".local/share")?,
            Base::Etc => PathBuf::from("/etc"),
        };
        Some(root.join(self.path))
    }

    /// Resolve and check whether the file actually exists on disk.
    pub fn exists(&self) -> bool {
        self.resolve().map(|p| p.exists()).unwrap_or(false)
    }
}

/// Resolve an XDG base dir: honor the env var if set to an absolute path,
/// otherwise fall back to `$HOME/<default>`.
fn xdg_dir(var: &str, default_suffix: &str) -> Option<PathBuf> {
    match env::var_os(var) {
        Some(v) if !v.is_empty() && PathBuf::from(&v).is_absolute() => Some(PathBuf::from(v)),
        _ => dirs::home_dir().map(|h| h.join(default_suffix)),
    }
}

#[rustfmt::skip]
// TODO: Is there an easier way to group these? Perhaps a different file or format?
/// All known config files, grouped by base for readability.
/// This will be manually modified as PRs come in.
pub const CONFIG_FILES: &[ConfigFile] = &[
    // DISCLAIMER: Claude actually helped me put this long list below together as a start.
    // ---- Home dotfiles (`$HOME/<path>`) -----------------------------------
    ConfigFile { tool: "bash",       base: Base::Home, path: ".bashrc" },
    ConfigFile { tool: "bash",       base: Base::Home, path: ".bash_profile" },
    ConfigFile { tool: "bash",       base: Base::Home, path: ".bash_logout" },
    ConfigFile { tool: "sh",         base: Base::Home, path: ".profile" },
    ConfigFile { tool: "zsh",        base: Base::Home, path: ".zshrc" },
    ConfigFile { tool: "zsh",        base: Base::Home, path: ".zshenv" },
    ConfigFile { tool: "zsh",        base: Base::Home, path: ".zprofile" },
    ConfigFile { tool: "zsh",        base: Base::Home, path: ".zlogin" },
    ConfigFile { tool: "readline",   base: Base::Home, path: ".inputrc" },
    ConfigFile { tool: "vim",        base: Base::Home, path: ".vimrc" },
    ConfigFile { tool: "vim",        base: Base::Home, path: ".gvimrc" },
    ConfigFile { tool: "git",        base: Base::Home, path: ".gitconfig" },
    ConfigFile { tool: "git",        base: Base::Home, path: ".gitignore_global" },
    ConfigFile { tool: "tmux",       base: Base::Home, path: ".tmux.conf" },
    ConfigFile { tool: "screen",     base: Base::Home, path: ".screenrc" },
    ConfigFile { tool: "ssh",        base: Base::Home, path: ".ssh/config" },
    ConfigFile { tool: "ssh",        base: Base::Home, path: ".ssh/known_hosts" },
    ConfigFile { tool: "ssh",        base: Base::Home, path: ".ssh/authorized_keys" },
    ConfigFile { tool: "gnupg",      base: Base::Home, path: ".gnupg/gpg.conf" },
    ConfigFile { tool: "gnupg",      base: Base::Home, path: ".gnupg/gpg-agent.conf" },
    ConfigFile { tool: "curl",       base: Base::Home, path: ".curlrc" },
    ConfigFile { tool: "wget",       base: Base::Home, path: ".wgetrc" },
    ConfigFile { tool: "netrc",      base: Base::Home, path: ".netrc" },
    ConfigFile { tool: "aws",        base: Base::Home, path: ".aws/config" },
    ConfigFile { tool: "aws",        base: Base::Home, path: ".aws/credentials" },
    ConfigFile { tool: "docker",     base: Base::Home, path: ".docker/config.json" },
    ConfigFile { tool: "kubectl",    base: Base::Home, path: ".kube/config" },
    ConfigFile { tool: "npm",        base: Base::Home, path: ".npmrc" },
    ConfigFile { tool: "yarn",       base: Base::Home, path: ".yarnrc" },
    ConfigFile { tool: "cargo",      base: Base::Home, path: ".cargo/config.toml" },
    ConfigFile { tool: "gem",        base: Base::Home, path: ".gemrc" },
    ConfigFile { tool: "conda",      base: Base::Home, path: ".condarc" },
    ConfigFile { tool: "terraform",  base: Base::Home, path: ".terraformrc" },
    ConfigFile { tool: "mysql",      base: Base::Home, path: ".my.cnf" },
    ConfigFile { tool: "psql",       base: Base::Home, path: ".psqlrc" },
    ConfigFile { tool: "irb",        base: Base::Home, path: ".irbrc" },
    ConfigFile { tool: "nano",       base: Base::Home, path: ".nanorc" },
    ConfigFile { tool: "ack",        base: Base::Home, path: ".ackrc" },
    ConfigFile { tool: "ctags",      base: Base::Home, path: ".ctags" },
    ConfigFile { tool: "editorconfig", base: Base::Home, path: ".editorconfig" },
    ConfigFile { tool: "dircolors",  base: Base::Home, path: ".dir_colors" },
    ConfigFile { tool: "x11",        base: Base::Home, path: ".Xresources" },
    ConfigFile { tool: "x11",        base: Base::Home, path: ".xinitrc" },
    ConfigFile { tool: "x11",        base: Base::Home, path: ".xprofile" },
 
    // ---- XDG config (`$XDG_CONFIG_HOME` ~ `~/.config/<path>`) --------------
    ConfigFile { tool: "nvim",       base: Base::XdgConfig, path: "nvim/init.lua" },
    ConfigFile { tool: "nvim",       base: Base::XdgConfig, path: "nvim/init.vim" },
    ConfigFile { tool: "git",        base: Base::XdgConfig, path: "git/config" },
    ConfigFile { tool: "git",        base: Base::XdgConfig, path: "git/ignore" },
    ConfigFile { tool: "fish",       base: Base::XdgConfig, path: "fish/config.fish" },
    ConfigFile { tool: "nushell",    base: Base::XdgConfig, path: "nushell/config.nu" },
    ConfigFile { tool: "starship",   base: Base::XdgConfig, path: "starship.toml" },
    ConfigFile { tool: "alacritty",  base: Base::XdgConfig, path: "alacritty/alacritty.toml" },
    ConfigFile { tool: "kitty",      base: Base::XdgConfig, path: "kitty/kitty.conf" },
    ConfigFile { tool: "wezterm",    base: Base::XdgConfig, path: "wezterm/wezterm.lua" },
    ConfigFile { tool: "tmux",       base: Base::XdgConfig, path: "tmux/tmux.conf" },
    ConfigFile { tool: "htop",       base: Base::XdgConfig, path: "htop/htoprc" },
    ConfigFile { tool: "btop",       base: Base::XdgConfig, path: "btop/btop.conf" },
    ConfigFile { tool: "bat",        base: Base::XdgConfig, path: "bat/config" },
    ConfigFile { tool: "gh",         base: Base::XdgConfig, path: "gh/config.yml" },
    ConfigFile { tool: "gh",         base: Base::XdgConfig, path: "gh/hosts.yml" },
    ConfigFile { tool: "helix",      base: Base::XdgConfig, path: "helix/config.toml" },
    ConfigFile { tool: "zellij",     base: Base::XdgConfig, path: "zellij/config.kdl" },
    ConfigFile { tool: "lazygit",    base: Base::XdgConfig, path: "lazygit/config.yml" },
    ConfigFile { tool: "k9s",        base: Base::XdgConfig, path: "k9s/config.yaml" },
    ConfigFile { tool: "pip",        base: Base::XdgConfig, path: "pip/pip.conf" },
    ConfigFile { tool: "ranger",     base: Base::XdgConfig, path: "ranger/rc.conf" },
    ConfigFile { tool: "yazi",       base: Base::XdgConfig, path: "yazi/yazi.toml" },
    ConfigFile { tool: "atuin",      base: Base::XdgConfig, path: "atuin/config.toml" },
    ConfigFile { tool: "i3",         base: Base::XdgConfig, path: "i3/config" },
    ConfigFile { tool: "sway",       base: Base::XdgConfig, path: "sway/config" },
 
    // ---- System config (`/etc/<path>`) ------------------------------------
    ConfigFile { tool: "hosts",      base: Base::Etc, path: "hosts" },
    ConfigFile { tool: "dns",        base: Base::Etc, path: "resolv.conf" },
    ConfigFile { tool: "fstab",      base: Base::Etc, path: "fstab" },
    ConfigFile { tool: "passwd",     base: Base::Etc, path: "passwd" },
    ConfigFile { tool: "group",      base: Base::Etc, path: "group" },
    ConfigFile { tool: "sudo",       base: Base::Etc, path: "sudoers" },
    ConfigFile { tool: "cron",       base: Base::Etc, path: "crontab" },
    ConfigFile { tool: "bash",       base: Base::Etc, path: "bash.bashrc" },
    ConfigFile { tool: "sh",         base: Base::Etc, path: "profile" },
    ConfigFile { tool: "environment",base: Base::Etc, path: "environment" },
    ConfigFile { tool: "sshd",       base: Base::Etc, path: "ssh/sshd_config" },
    ConfigFile { tool: "ssh",        base: Base::Etc, path: "ssh/ssh_config" },
    ConfigFile { tool: "nginx",      base: Base::Etc, path: "nginx/nginx.conf" },
    ConfigFile { tool: "os-release", base: Base::Etc, path: "os-release" },
];

// TODO: This would be great for quick compares, but would need to be dynamic, not a const.
/// A flat list of just the relative path strings, for a plain `.contains()`:
///
/// ```
/// # use config_files::CONFIG_PATHS;
/// assert!(CONFIG_PATHS.contains(&".gitconfig"));
/// ```
// pub const CONFIG_PATHS: &[&str] = &{
//     [
//         // Perhaps this makes more sense to iterate over?
//         // ".bashrc",
//         // ".bash_profile",
//         // ".zshrc",
//         // ".profile",
//         // ".vimrc",
//         // ".gitconfig",
//         // ".tmux.conf",
//         // "starship.toml",
//         "hosts",
//         "resolv.conf",
//         "fstab",
//         "ssh/sshd_config",
//     ]
// };

/// All potential available config directory paths
pub fn all_potential_paths() -> Vec<&'static str> {
    CONFIG_FILES.iter().map(|c| c.path).collect()
}

/// Every config file that **actually** exists on this machine.
pub fn existing() -> Vec<&'static ConfigFile> {
    CONFIG_FILES.iter().filter(|c| c.exists()).collect()
}

/// Given a list of relative config `path` strings, return the matching
/// `ConfigFile` entries from `CONFIG_FILES`.
///
/// The returned references borrow from the static `CONFIG_FILES` array.
pub fn configs_from_paths(paths: Vec<String>) -> Vec<&'static ConfigFile> {
    let mut out: Vec<&'static ConfigFile> = Vec::new();
    for p in paths {
        if let Some(cfg) = CONFIG_FILES.iter().find(|c| c.path == p.as_str()) {
            out.push(cfg);
        }
    }
    out
}
