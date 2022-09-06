use anyhow::Result;
use azure_devops_rust_api::{git, Credential};
use log::info;
use std::env;
use std::sync::Arc;


#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let credential = match env::var("ADO_TOKEN") {
        Ok(token) => {
            info!("Authenticating with PAT from $ADO_TOKEN");
            Credential::from_pat(token)
        }
        Err(_) => {
            info!("Attempting to authenticate with Azure CLI");
            Credential::from_token_credential(Arc::new(azure_identity::AzureCliCredential {}))
        }
    };


    let organization = "IconProGmbH".to_owned();
    let project = "ARES".to_owned();

    let git_client = git::ClientBuilder::new(credential).build();
    
    let repos = git_client
        .repositories_client()
        .list(organization, project)
        .into_future()
        .await?
        .value;

    for repo in repos.iter() {
        info!("{}", repo.name);
    }
    info!("{} repos found.", repos.len());

    Ok(())
}
