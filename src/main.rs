mod config;
mod init;

fn main() {
    let user_responses = init::init_prompt(true);
    match user_responses {
        Ok(res) => {
            println!("{:#?}", res) // debug for later
        }
        Err(_) => todo!(),
    }
}
