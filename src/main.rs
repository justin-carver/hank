// TODO: Should these get moved into a lib.rs?
mod configs;
mod init;
mod sync;
mod utils;

fn main() {
    let user_responses = init::init_prompt(false);
    match user_responses {
        Ok(res) => {
            if Some(&res).is_some() {
                let configs = sync::generate_local_sync(res.tracked);
                println!("{:#?}", configs);
            }
        }
        Err(e) => eprintln!("No 'Post-UserResponse' Method Configured!: {:#?}", e),
    }
}
