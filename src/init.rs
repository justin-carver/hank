use cliclack::{confirm, input, intro, log, multiselect, outro};
use regex::Regex;
use std::{env, path::Path, process};

use console::style;

use crate::configs::{self, ConfigFile, configs_from_paths};
use crate::utils::{self, BoxError, write_json};

// TODO: These should probably make more sense, or take in some sort of data.
#[derive(Debug)]
pub enum PromptError {
    CoreInitFailure,
    CanceledPrompt,
    ResponseLength,
    SaveFailure,
    UnableToFindTool,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct InitResponses<'a> {
    gh_username: String,
    gh_repo_name: String,
    local_path: String,
    configs: Vec<&'a ConfigFile>,
}

impl InitResponses<'_> {
    /// Regex comparison against official GitHub username requirements.
    /// The [Regex] crate apparently does not support `lookaround` comparators,
    /// so the matching is a little more manual than normal regex.
    fn regex_gh_username(user: &String) -> bool {
        let rgx_gh = Regex::new(r"^[a-zA-Z0-9](?:[a-zA-Z0-9]|-[a-zA-Z0-9])*$").unwrap();
        if !user.is_empty() && user.len() <= 39 && rgx_gh.is_match(user) {
            true
        } else {
            false
        }
    }
    /// Regex processing against Unix-styled paths
    fn regex_unix_path(path: &String) -> bool {
        let rgx_path = Regex::new(
            r"^(?:~[\w.-]*(?:\/[\w.-]+)*|\/(?:[\w.-]+(?:\/[\w.-]+)*)?|[\w.-]+(?:\/[\w.-]+)*)\/?$",
        )
        .unwrap();
        if rgx_path.is_match(path) { true } else { false }
    }
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Meta {
    name: String,
    retries: u8,
}

// TODO: Need to do checks for things like Git repo settings, and metadata.json
/// Initializes the prompt for the first time user.
pub fn init_prompt(c: bool) -> Result<InitResponses<'static>, PromptError> {
    if c {
        intro(format!(
            "Hey, {}! Let's figure out what we need to sync!",
            env::var("USER").unwrap().replace("\"", "")
        ))
        .map_err(|_| PromptError::CoreInitFailure)?;
        let _ = cliclack::note(
            style(" Before we start... ").on_cyan().black(),
            "While Hank is very unopinionated, we want to make sure you control\n\
            when and where your config files get saved, stored, and synced.\n\
            To do so, we'll need some info first about finding your files.\n\
            By canceling this prompt, we won't push or init any settings.\n\n",
        );
        let username: String = input("What is your GitHub username?")
            .placeholder("john-smith")
            .validate(move |input: &String| {
                if input.is_empty() && !InitResponses::regex_gh_username(input) {
                    Err("Please specify a valid GitHub username.")
                } else if input.starts_with("@") {
                    Err("Do not include the @ symbol.")
                } else {
                    Ok(())
                }
            })
            .interact()
            .map_err(|_| PromptError::CanceledPrompt)?;
        let mut repo_name: String =
            input("What is the name of the repo hosting your config files? (dotfiles)")
                .placeholder("dotfiles")
                .required(false)
                .interact()
                .map_err(|e| PromptError::CanceledPrompt)?;
        if repo_name.is_empty() {
            repo_name = String::from("dotfiles");
        }
        let path: String =
            input("Where should we store your pending config changes before pushing?")
                .placeholder("~/local/pending/changes")
                .validate(move |input: &String| {
                    if input.is_empty() {
                        Err("Please enter a path.")
                    } else if !InitResponses::regex_unix_path(input) {
                        Err("Please enter a valid path")
                    } else {
                        Ok(())
                    }
                })
                .interact()
                .map_err(|_| PromptError::CanceledPrompt)?;

        let combined_path = [&path, "/meta.json"].concat();
        if utils::exists(combined_path) {
            let overwrite = confirm(format!(
                "There already exists a meta.json located in:\n\n\x1b[1;33m{}\x1b[0m\n\n{}",
                &path, "Do you want to overwite this file with your answers above?\n(All data will be lost)"
            ))
            .interact()
            .map_err(|e| PromptError::SaveFailure)?;
            if !overwrite {
                println!("Aborting init process, choices will be discarded.");
                process::exit(1);
            }
        }
        log::info("Let's scan for what configs you have on this machine already...")
            .map_err(|_| PromptError::CanceledPrompt)?;

        // The good stuff, finding what config files to start syncing
        let spinner = cliclack::spinner();
        spinner.start("Finding configuration files...");

        let mut tools: Vec<(String, String, String)> = Vec::new();
        let total_configs = configs::existing();
        for cfg in &total_configs {
            // We need to push them in a certain order for readability
            tools.push((
                cfg.path.to_string(), // true value
                cfg.tool.to_string(), // displayed text in prompt
                [cfg.base.to_string(), "/".to_string(), cfg.path.to_string()].concat(), // hint
            ));
        }
        tools.sort_by_key(|k| k.1.to_lowercase());
        spinner.stop(format!("Found {} files!", total_configs.len()));

        let found_configs = multiselect("Choose which configs to track.")
            .items(&tools)
            .interact()
            .map_err(|_| PromptError::UnableToFindTool)?;
        let tracked_configs = configs_from_paths(found_configs);

        // Try ending the prompts and sending data to disk
        let res: InitResponses = InitResponses {
            gh_username: username,
            gh_repo_name: repo_name,
            local_path: path,
            configs: tracked_configs,
        };
        // Write all responses to `meta.json` file, stored in the preferred local directory
        outro("All set! Use `hank list` to show tracked config changes.")
            .map_err(|_| PromptError::SaveFailure)?;
        Ok(parse_init_responses(res).unwrap())
    } else {
        Err(PromptError::CanceledPrompt)
    }
}

fn parse_init_responses(res: InitResponses) -> Result<InitResponses, BoxError> {
    let path = Path::new(&res.local_path).join("meta.json");
    write_json(&path, &res)?;
    Ok(res)
}

// ::: Tests :::
// TODO: Should these be moved to their own /test folder?

// Unix-only path validation
#[test]
fn test_validate_unix_path_regex() {
    // (input, should_match)
    let cases: &[(&str, bool)] = &[
        // valid — home / absolute / relative
        ("/", true),
        ("/usr/local/bin", true),
        ("/etc/", true),
        ("~/.config", true),
        ("~/Documents/projects/", true),
        ("~alice/work", true), // ~user form
        ("./src", true),
        ("../tests/fixtures", true),
        ("node_modules/.bin", true),
        ("home/user/.ssh/", true),
        // invalid
        ("", false),          // empty
        ("//etc", false),     // leading double slash
        ("/foo//bar", false), // double slash mid-path
        ("foo bar/", false),  // space not in [\w.-]
        ("~~/x", false),      // ~ not allowed mid-name
    ];

    for &(input, expected) in cases {
        let got = InitResponses::regex_unix_path(&input.to_string());
        assert_eq!(got, expected, "{input:?}: expected {expected}, got {got}");
    }
}

// GitHub username validation
#[test]
fn test_validate_github_username_regex() {
    // (input, should_match)
    let cases: &[(&str, bool)] = &[
        ("john", true),
        ("john-smith", true),
        ("a", true),
        ("user123", true),
        ("user-name", true),
        ("something-like-a-really-long-username-1", true),
        ("", false),
        ("-starts", false),
        ("ends-", false),
        ("@john", false),
        ("john--doe", false),
        ("john_doe", false),
        ("john.doe", false),
        ("john_doe!", false),
    ];

    for &(input, expected) in cases {
        let got = !input.is_empty()
            && input.len() <= 39
            && InitResponses::regex_gh_username(&input.to_string());
        assert_eq!(
            got, expected,
            "{:?}: expected {}, got {}",
            input, expected, got
        );
    }

    // Too long (>39 chars) should be invalid
    let long = "a".repeat(40);
    assert!(
        !InitResponses::regex_gh_username(&long.to_string()),
        "40 'a's should be invalid"
    );
}
