use crate::{
    configs::ConfigList,
    utils::{BoxError, get_metadata_path, read_json},
};

use crate::init::Metadata;

pub fn generate_local_sync(mut configs: ConfigList) -> ConfigList {
    // Pull in all required configuration files into one location
    // stage them, then wait for user to push them to a repo

    // Get the original path where Metadata is located
    let metadata: Result<Metadata, BoxError> = read_json(get_metadata_path().unwrap());
    if Some(&metadata).is_some() {
        configs = metadata.unwrap().tracked;
    }
    configs
}
