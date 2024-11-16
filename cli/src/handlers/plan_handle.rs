use std::{fs, path::Path};

use anyhow::{anyhow, Result};
use shared::{
    deployable::deploy::{Deploy, DeployAction, DeployTask},
    ok,
};

use crate::{
    api::API,
    data::{RemoteAuth, UserData},
    utils::open_file_as_string,
};

pub async fn handle_plan(
    single_filter: Option<String>,
    only: Option<Vec<String>>,
    file_name: String,
    context: String,
    to_build: Option<Vec<String>>,
) -> Result<(RemoteAuth, Vec<Deploy>)> {
    //println!("Plan");
    //dbg!(&single_filter);
    //dbg!(&only);
    //dbg!(&file_name);
    //dbg!(&context);
    //dbg!(&no_build);

    // prepare config
    let abs_path = fs::canonicalize(Path::new(&context))?;
    let config_path = abs_path.join(&file_name);
    let user = UserData::load_db(false).await?.load_current_user().await?;
    let raw_config = open_file_as_string(
        config_path
            .to_str()
            .ok_or(anyhow!("failed to convert path to string"))?,
    )?;
    let final_filter = if single_filter.is_some() && only.is_some() {
        let mut ffilter = only.clone().unwrap();
        ffilter.push(single_filter.unwrap());
        ffilter
    } else if single_filter.is_none() && only.is_some() {
        only.clone().unwrap()
    } else if single_filter.is_some() && only.is_none() {
        vec![single_filter.unwrap()]
    } else {
        vec![]
    };

    // get plan
    let deploys = API::new(&user.remote_url)?
        .get_plans(
            raw_config,
            user.remote_token.clone(),
            to_build,
            final_filter,
        )
        .await?;

    // print tasks
    println!("Tasks: ");

    // build tasks
    let build_tasks = deploys.iter().fold(vec![], |mut a, b| {
        b.client_tasks.iter().for_each(|task| {
            if let DeployTask::Build(b) = task {
                a.push(b.clone());
            };
        });
        a
    });
    if !build_tasks.is_empty() {
        println!("  Build - {}:", build_tasks.len());
        for task in build_tasks {
            println!("    - {}", task.short_name);
        }
    }

    // create tasks
    let create_tasks = deploys.iter().fold(vec![], |mut a, b| {
        if let DeployAction::Create = b.action {
            a.push(b.clone());
        }
        a
    });
    if !create_tasks.is_empty() {
        println!("  Create - {}:", create_tasks.len());
        for task in create_tasks {
            println!("    - {}", task.deployable.short_name);
        }
    }

    // update tasks
    let update_tasks = deploys.iter().fold(vec![], |mut a, b| {
        if let DeployAction::Update = b.action {
            a.push(b.clone());
        }
        a
    });
    if !update_tasks.is_empty() {
        println!("  Update - {}:", update_tasks.len());
        for task in update_tasks {
            println!("    - {}", task.deployable.short_name);
        }
    }

    // delete tasks
    let delete_tasks = deploys.iter().fold(vec![], |mut a, b| {
        if let DeployAction::Delete = b.action {
            a.push(b.clone());
        }
        a
    });
    if !delete_tasks.is_empty() {
        println!("  Delete - {}:", delete_tasks.len());
        for task in delete_tasks {
            println!("    - {}", task.deployable.short_name);
        }
    }
    ok!((user, deploys))
}
