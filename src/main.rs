use anyhow::Result;
use azure_devops_rust_api::git;
use azure_devops_rust_api::core::*;
use azure_devops_rust_api::Credential;
use dotenv::dotenv;
use git2::Repository;

use log::info;
use std::env;
use std::sync::Arc;


async fn list_repos(organization: &str, project: &str, credential: &Credential) -> Result<Vec<git::models::GitRepository>> {
    let git_client = git::ClientBuilder::new(credential.clone()).build();
    let repos = git_client.repositories_client()
        .list(organization, project)
        .into_future()
        .await?
        .value;
    Ok(repos)
}


async fn list_contributors(organization: &str, project: &str, credential: &Credential) -> Result<Vec<String>> {
    let core_client = ClientBuilder::new(credential.clone()).build();
    let project_obj = core_client.projects_client()
            .get(organization, project)
            .into_future()
            .await?;
    let default_team = project_obj.default_team.unwrap();
    let team_id = default_team.id.unwrap();
    let members = core_client.teams_client().get_team_members_with_extended_properties(organization, project, &team_id).into_future().await?;

    let names = members.value.into_iter()
        .map(|member| member.identity)
        .flatten()
        .map(|identity| identity.unique_name)
        .collect();

    Ok(names)
}


fn get_local_repo(repository: &Option<Repository>) -> Result<String> {
    // if we're inside a repo
    if let Some(repo) = repository {
        // it should have the origin remote, else boom
        let remote = repo.find_remote("origin")?;
        // if the remote has a url
        if let Some(url) = remote.url() {
            // return that
            return Ok(url.to_owned())
        }
    }
    // else prompt it from the user
    Ok(String::from("adf"))
}

fn get_src_branch(repository: &Option<Repository>) -> Result<String> {
    // if we're inside a repo
    if let Some(repo) = repository {
        // we should be able to get a hold of the HEAD ref
        let head = repo.head()?;
        // if we're on a branch head
        if head.is_branch() {
            // then return the short name
            return Ok(head.shorthand().expect("Expected branch name to be valid UTF-8.").to_owned())
        }
    }

    // else prompt it from the user
    Ok(String::from("asdf"))
}


fn get_target_branch(repository: &Option<Repository>) -> Result<String> {
    // if we're inside a repo
    if let Some(repo) = repository {
        let target_branch = repo.find_remote("origin")?.default_branch()?;
        return Ok(target_branch.as_str().expect("Expected branch name to be valid UTF-8.").to_owned())
    }

    // else prompt it from the user
    Ok(String::from("asf"))
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
    
    let repo = Repository::open(env::current_dir()?).ok();

    let repo_url = get_local_repo(&repo);
    let source_branch = get_src_branch(&repo);
    let target_branch = get_target_branch(&repo);
    let contributor_list = list_contributors(&organization, &project, &credential).await?;

    // PR format:  <repo> <src_branch> <trg_branch> <title> <desc> <req_reviewers> <opt_reviewers>

    dbg!(repo_url.ok());

    // if let Ok(repo) = Repository::open(env::current_dir()?) {
    //     let remote = repo.find_remote("origin")?;
    //     let repo_url = remote.url().unwrap().to_owned(); 
        
    //     // let source_branch = 
        
    //     let kiki = repo.head()?.shorthand()
    //     dbg!(repo.head().unwrap().is_branch());
    //     dbg!(repo.head().unwrap().name());
    //     dbg!(repo.head().unwrap().shorthand());
    //     println!("{}", repo.head_detached().ok().unwrap());
    // }
    // info!("{}", repo_url);


    // let repos = get_repos(&organization, &project, &credential).await?;
    
    // for repo in repos.iter() {
    //     info!("{}", repo.name);
    // }
    // info!("{} repos found.", repos.len());
    
    // let members = get_contributors(&organization, &project, &credential).await?;

    // let member_names: Vec<String> = members.value.into_iter()
    //     .map(|member| member.identity)
    //     .flatten()
    //     .map(|identity| identity.unique_name)
    //     .collect();

    // println!("{:?}", member_names);

    // let prompty_res = MultiSelect::new("Select your witnesses:", member_names)
    //     .prompt()?;


    // println!("{:?}",  prompty_res);

    Ok(())
}
