use crate::exports::stargazers::workflow::workflow::Guest;
use stargazers::{account::account, db, llm::llm};
use wit_bindgen::generate;

generate!({ generate_all });
struct Component;
export!(Component);

impl Guest for Component {
    fn star_added(sender: String, repo: String) -> Result<(), String> {
        // Persist the user giving a star to the project.
        let description = db::user::link(&sender, &repo)?;
        if description.is_none() {
            // Fetch the account info from github.
            // TODO: account::info and get_settings_json should run in parallel
            let info = account::info(&sender)?;
            let settings_json = db::llm::get_settings_json()?;
            // Generate the user's description.
            let description = llm::respond(&info, &settings_json)?;
            db::user::user_update(&sender, &description)?;
        }
        Ok(())
    }

    fn star_removed(sender: String, repo: String) -> Result<(), String> {
        db::user::unlink(&sender, &repo)
    }
}
