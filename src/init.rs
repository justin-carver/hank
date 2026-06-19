use cliclack::{input, intro, log, note, outro};

#[derive(Debug)]
pub enum PromptError {
    CoreInitFailure,
    CanceledPrompt,
    ResponseLength,
    SaveFailure,
}

pub struct InitResponses {
    gh_username: String,
    gh_repo_name: String,
}

// TODO: Need to do checks for things like Git repo settings, and metadata.json
/// Initializes the prompt for hte first time ever.
pub fn init_prompt(c: bool) -> Result<InitResponses, PromptError> {
    if c {
        intro("Welcome, $USERNAME! Let's figure out what we need to sync!")
            .map_err(|_| PromptError::CoreInitFailure)?;
        let _ = cliclack::note(
            "Before we start...",
            "While Hank is very unopinionated, we want to make sure you control\n\
            when and where your config files get saved, stored, and synced.\n\
            By canceling this prompt, we won't push or init any settings.",
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
        // Prompt the user for start questions.
        let path: String =
            input("Where should we store your pending config changes before pushing?")
                .placeholder("~/local/pending/changes")
                .validate(|input: &String| {
                    if input.is_empty() {
                        Err("Please enter a path.")
                        // FIXME: This needs to be resolved better. Need a function to regex map valid filepaths.
                    } else if !input.starts_with("~/") {
                        Err("Please enter a relative path")
                    } else {
                        Ok(())
                    }
                })
                .interact()
                .map_err(|e| PromptError::CanceledPrompt)?;
        // Write all responses to HashMap
        outro("You're all set!").map_err(|_| PromptError::SaveFailure)?;
        Ok(InitResponses {
            gh_username: username,
            gh_repo_name: "String".to_string(),
        })
    } else {
        Err(PromptError::CanceledPrompt)
    }
}
