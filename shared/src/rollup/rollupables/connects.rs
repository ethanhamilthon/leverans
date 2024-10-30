use anyhow::{anyhow, Result};

use crate::{err, ok};

use super::Rollupable;

impl Rollupable {
    pub fn get_connection(&self) -> Result<String> {
        match self {
            Rollupable::App(_rollupable_app) => {
                err!(anyhow!("get connection for app is not implemented yet"))
            }
            Rollupable::Database(rdb) => ok!(rdb.params.get_connection()?),
            Rollupable::Service(_rasrs) => {
                err!(anyhow!("get connection for app is not implemented yet"))
            }
        }
    }
}
