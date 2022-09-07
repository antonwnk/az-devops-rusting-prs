use anyhow::Result;
use azure_devops_rust_api::git;
use azure_devops_rust_api::Credential;
use log::info;
use std::env;
use std::sync::Arc;


async fn get_repos(organization: &str, project: &str, credential: Credential) -> Result<Vec<git::models::GitRepository>> {
    let git_client = git::ClientBuilder::new(credential).build();
    let repos = git_client.repositories_client()
        .list(organization, project)
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

    let organization = env::var("ADO_ORGANIZATION").expect("Specify organization with $ADO_ORAGANIZATION.");
    let project = env::var("ADO_PROJECT").expect("Specify project with $ADO_PROJECT");

    let repos = get_repos(&organization, &project, credential.clone()).await?;
    
    for repo in repos.iter() {
        info!("{}", repo.name);
    }
    info!("{} repos found.", repos.len());

    Ok(())
}
