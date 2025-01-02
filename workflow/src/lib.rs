use crate::exports::stargazers::workflow::workflow::Guest;
use stargazers::{account::account, db, llm::llm};
use wit_bindgen::generate;

generate!({ generate_all });
struct Component;
export!(Component);

impl Guest for Component {
    fn star_added(login: String, repo: String) -> Result<(), String> {
        // Persist the user giving a star to the project.
        let description = db::user::link_get_description(&login, &repo)?;
        if description.is_none() {
            // Fetch the account info from github.
            // TODO: account::info and get_settings_json should run in parallel
            let info = account::info(&login)?;
            // TODO: cache for 5 mins
            let settings_json = db::llm::get_settings_json()?;
            // Generate the user's description.
            let description = llm::respond(&info, &settings_json)?;
            db::user::user_update(&login, &description)?;
        }
        Ok(())
    }

    fn star_removed(login: String, repo: String) -> Result<(), String> {
        db::user::unlink(&login, &repo)
    }
}
