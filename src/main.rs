// TODO: Should these get moved into a lib.rs?
mod configs;
mod init;
mod utils;

fn main() {
    let user_responses = init::init_prompt(true);
    match user_responses {
        Ok(res) => {
            println!("{:#?}", res); // debug for later
        }
        Err(e) => eprintln!("Error: {:#?}", e),
    }
}
