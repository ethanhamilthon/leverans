use std::str::FromStr;

use crate::config::MainConfig;
use anyhow::{anyhow, Result};

use super::deploy::{Deploy, DeployAction};

pub struct RollBackParams {
    pub main_config: String,
    pub last_deploys: Vec<(String, String)>,
    pub prelast_deploys: Vec<(String, String)>,
}

pub fn rollback(params: RollBackParams) -> Result<Vec<Deploy>> {
    let main_config = MainConfig::from_str(&params.main_config)
        .map_err(|e| anyhow!("cannot parse last config: {}", e.to_string()))?;
    let last_deploys = params
        .last_deploys
        .clone()
        .into_iter()
        .find(|(project_name, _)| project_name == &main_config.project)
        .map(|(_, d)| serde_json::from_str::<Vec<Deploy>>(&d).ok())
        .flatten()
        .ok_or(anyhow!("Last Deployments not found"))?;
    dbg!(&last_deploys);
    let prelast_deploys = params
        .prelast_deploys
        .clone()
        .into_iter()
        .find(|(project_name, _)| project_name == &main_config.project)
        .map(|(_, d)| serde_json::from_str::<Vec<Deploy>>(&d).ok())
        .flatten()
        .ok_or(anyhow!("Last Deployments not found"))?;
    dbg!(&prelast_deploys);
    let mut final_deploys = Vec::new();
    for mut d in last_deploys {
        match d.action {
            DeployAction::Update => {
                let mut prelast = prelast_deploys
                    .iter()
                    .find(|p| {
                        p.deployable.short_name == d.deployable.short_name
                            && p.deployable.project_name == d.deployable.project_name
                    })
                    .ok_or(anyhow!(
                        "Could not find state to rollback for {}",
                        d.deployable.short_name.clone()
                    ))?
                    .clone();

                prelast.action = DeployAction::Update;
                final_deploys.push(prelast);
            }
            DeployAction::Create => {
                if prelast_deploys.contains(&d) {
                    d.action = DeployAction::Nothing;
                } else {
                    d.action = DeployAction::Delete;
                }
                final_deploys.push(d);
            }
            DeployAction::Delete => {
                d.action = DeployAction::Create;
                final_deploys.push(d);
            }
            DeployAction::Nothing => {
                final_deploys.push(d);
            }
        }
    }
    dbg!(&final_deploys);
    Ok(final_deploys)
}
