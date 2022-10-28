use anyhow::Result;
use azure_devops_rust_api::core::models::TeamMemberList;
use azure_devops_rust_api::git;
use azure_devops_rust_api::core::*;
use azure_devops_rust_api::Credential;
use dotenv::dotenv;
use inquire::MultiSelect;
use log::info;
use std::env;
use std::sync::Arc;


async fn get_repos(organization: &str, project: &str, credential: &Credential) -> Result<Vec<git::models::GitRepository>> {
    let git_client = git::ClientBuilder::new(credential.clone()).build();
    let repos = git_client.repositories_client()
        .list(organization, project)
        .into_future()
        .await?
        .value;
    Ok(repos)
}


async fn get_contributors(organization: &str, project: &str, credential: &Credential) -> Result<TeamMemberList> {
    let core_client = ClientBuilder::new(credential.clone()).build();
    let project_obj = core_client.projects_client()
            .get(organization, project)
            .into_future()
            .await?;
    let default_team = project_obj.default_team.unwrap();
    let team_id = default_team.id.unwrap();
    let members = core_client.teams_client().get_team_members_with_extended_properties(organization, project, &team_id).into_future().await?;
    Ok(members)
}



#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
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

    let organization = env::var("ADO_ORGANIZATION").expect("Specify organization with $ADO_ORGANIZATION.");
    let project = env::var("ADO_PROJECT").expect("Specify project with $ADO_PROJECT");

    let repos = get_repos(&organization, &project, &credential).await?;
    
    for repo in repos.iter() {
        info!("{}", repo.name);
    }
    info!("{} repos found.", repos.len());
    
    let members = get_contributors(&organization, &project, &credential).await?;

    let member_names: Vec<String> = members.value.into_iter()
        .map(|member| member.identity)
        .flatten()
        .map(|identity| identity.unique_name)
        .collect();

    println!("{:?}", member_names);

    let prompty_res = MultiSelect::new("Select your witnesses:", member_names)
        .prompt()?;


    println!("{:?}",  prompty_res);

    Ok(())
}
