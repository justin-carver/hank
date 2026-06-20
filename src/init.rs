use cliclack::{confirm, input, intro, log, note, outro};
use regex::Regex;
use std::{env, io, ops::ControlFlow::Break, path::Path, process};

use crate::config::{self, BoxError, read_or_create, write_json};

#[derive(Debug)]
pub enum PromptError {
    CoreInitFailure,
    CanceledPrompt,
    ResponseLength,
    SaveFailure,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct InitResponses {
    gh_username: String,
    gh_repo_name: String,
    local_path: String,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Config {
    name: String,
    retries: u8,
}

// TODO: Need to do checks for things like Git repo settings, and metadata.json
/// Initializes the prompt for the first time user.
pub fn init_prompt(c: bool) -> Result<InitResponses, PromptError> {
    let rgx_path = Regex::new(
        r"^(?:~[\w.-]*(?:\/[\w.-]+)*|\/(?:[\w.-]+(?:\/[\w.-]+)*)?|[\w.-]+(?:\/[\w.-]+)*)\/?$",
    )
    .unwrap();

    if c {
        intro(format!(
            "Hey, {}! Let's figure out what we need to sync!",
            env::var("USER").unwrap().replace("\"", "")
        ))
        .map_err(|_| PromptError::CoreInitFailure)?;
        let _ = cliclack::note(
            "Before we start...",
            "While Hank is very unopinionated, we want to make sure you control\n\
            when and where your config files get saved, stored, and synced.\n\
            By canceling this prompt, we won't push or init any settings.\n\n",
        );
        let username: String = input("What is your GitHub username? (Do not include @)")
            .placeholder("john-smith")
            .validate(|input: &String| {
                if input.is_empty() {
                    // TODO: Check GitHub's *actual* username requirements
                    Err("Please specify a valid GitHub username.")
                } else {
                    Ok(())
                }
            })
            .interact()
            .map_err(|e| PromptError::CanceledPrompt)?;
        let mut repo_name: String =
            input("What is the name of the repo hosting your config files?")
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
                        // FIXME: This needs to be resolved better. Need a function to regex map valid filepaths.
                    } else if !rgx_path.is_match(input) {
                        Err("Please enter a valid path")
                    } else {
                        Ok(())
                    }
                })
                .interact()
                .map_err(|e| PromptError::CanceledPrompt)?;

        let combined_path = [&path, "/meta.json"].concat();
        if config::exists(combined_path) {
            let overwrite = confirm(format!(
                "There already exists a meta.json located in:\n\n{}\n\n{}",
                &path, "Do you want to overwite this file with your answers above?"
            ))
            .interact()
            .map_err(|e| PromptError::SaveFailure)?;
            if !overwrite {
                println!("Aborting init process, choices will be discarded.");
                process::exit(1);
            }
        }
        let res: InitResponses = InitResponses {
            gh_username: username,
            gh_repo_name: repo_name,
            local_path: path,
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

// Unix-only path validation
#[test]
fn test_validate_unix_path_regex() {
    let rgx_path = Regex::new(
        r"^(?:~[\w.-]*(?:/[\w.-]+)*|/(?:[\w.-]+(?:/[\w.-]+)*)?|[\w.-]+(?:/[\w.-]+)*)/?$",
    )
    .unwrap();

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
        let got = rgx_path.is_match(input);
        assert_eq!(got, expected, "{input:?}: expected {expected}, got {got}");
    }
}
