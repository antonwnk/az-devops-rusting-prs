use anyhow::Result;
use azure_devops_rust_api::git;
use azure_devops_rust_api::Credential;
use log::info;
use std::env;
use std::sync::Arc;


const ORGANIZATION: &str = "IconProGmbH";
const PROJECT: &str = "ARES";


async fn get_repos(credential: Credential) -> Result<Vec<git::models::GitRepository>> {
    let git_client = git::ClientBuilder::new(credential).build();
    let repos = git_client.repositories_client()
        .list(ORGANIZATION, PROJECT)
        .into_future()
        .await?
        .value;
    Ok(repos)
}


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

    let repos = get_repos(credential.clone()).await?;
    
    for repo in repos.iter() {
        info!("{}", repo.name);
    }
    info!("{} repos found.", repos.len());

    Ok(())
}
