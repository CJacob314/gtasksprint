mod widgets;
mod toml_options;
use toml_options::TomlOptions;
use google_tasks1::TasksHub;
use widgets::*;

use std::{io,env};
use directories as dirs;
use chrono::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    /* TODO: Somehow let the user know how the TOML file should look and where it should be if it
     * doesn't exist! Maybe you generate a default one in the correct location and just let them
     * know where it is and what you generated?
     */

    let num_cols_opt = env::args().nth(1).map(|num_str| num_str.parse::<u16>()).and_then(|res| res.ok());
    
    rustls::crypto::ring::default_provider().install_default().unwrap();

    let base_dirs = if let Some(base_dirs) = dirs::BaseDirs::new() {
        base_dirs
    } else {
        return Err(io::Error::new(io::ErrorKind::NotFound, "No valid home directory path could be retrieved from the operating system!").into());
    };

    let secret_file = base_dirs.config_dir().join(GOOGLE_API_CREDS_JSON_FNAME);
    let token_disk_cache = base_dirs.data_local_dir().join(GOOGLE_API_TOKEN_CACHE_FNAME);
    let toml_config_file = base_dirs.config_dir().join(TOML_CONFIG_FNAME);

    let toml_table: TomlOptions = toml::from_str(&std::fs::read_to_string(&toml_config_file).map_err(|_| io::Error::new(io::ErrorKind::NotFound, format!("File {toml_config_file:?} should exist and contain the config for gtasksprint")))?)?;

    let secret = yup_oauth2::read_application_secret(&secret_file).await.unwrap_or_else(|_| panic!("File {secret_file:?} should be a client secret JSON file downloaded from the Google Cloud console"));

    let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
        secret,
        yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    )
        .persist_tokens_to_disk(token_disk_cache)
        .build().await.unwrap();

    let client = hyper_util::client::legacy::Client::builder(
        hyper_util::rt::TokioExecutor::new()
    ).build(
        hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .unwrap()
            .https_or_http()
            .enable_http1()
            .build()
    );
    let hub = TasksHub::new(client, auth);

    let max_due_date = Utc::now() + chrono::Duration::days(toml_table.tasks_config.max_due_future_days.unwrap_or(7) as _);
    let max_due_date_rfc3339 = max_due_date.to_rfc3339();

    let tasklists = hub.tasklists().list().doit().await?.1.items.expect("User should have at least one tasklist");
    let tasklist_id = tasklists
        .iter()
        .find(|tasklist| tasklist.title.as_ref().is_some_and(|title| title == &toml_table.tasks_config.tasks_list_name))
        .expect("TOML-configured tasks_config.tasks_list_name should the name of one of the user's task lists")
            .id.as_ref().expect("User's TOML-configured tasks_config.tasks_list_name task-list should have an ID");

    let result = hub.tasks().list(tasklist_id)
             .show_hidden(false)
             .show_deleted(false)
             .show_completed(false)
             .max_results(-10)
             .due_max(&max_due_date_rfc3339)
             .doit().await?;
    let tasks = result.1.items.expect("User's TOML-configured tasks_config.tasks_list_name should exist and have at least one task");

    let size = num_cols_opt
        .or_else(|| termsize::get().map(|sz| sz.cols))
        .expect("Program should either have been passed a width (in characters) as a CLI argument or should be able to get the current terminal from the OS");

    let boxed_text = Boxed::new(size, "Google Tasks", &tasks);
    boxed_text.draw(&mut io::stdout())?;

    Ok(())
}

const GOOGLE_API_TOKEN_CACHE_FNAME: &str = ".gtasksprint-token.cache";
const GOOGLE_API_CREDS_JSON_FNAME: &str = ".gtasksprint-creds.json";
const TOML_CONFIG_FNAME: &str = "gtasksprint.toml";
