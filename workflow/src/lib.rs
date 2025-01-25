use crate::exports::stargazers::workflow::workflow::Guest;
use stargazers::{
    account::account::{self, list_stargazers},
    db,
    llm::llm,
    workflow::workflow as imported_workflow,
};
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
            // account_info and get_settings_json should run in parallel. see branch `parallel`.
            let info = account::account_info(&login)?;
            // Fetch the prompt from the database.
            // TODO: cache for 5 mins
            let settings_json = db::llm::get_settings_json()?;
            // Generate the user's description.
            let description = llm::respond(&info, &settings_json)?;
            // Persist the generated description.
            db::user::user_update(&login, &description)?;
        }
        Ok(())
    }

    fn star_added_parallel(login: String, repo: String) -> Result<(), String> {
        // Persist the user giving a star to the project.
        let description = db::user::link_get_description(&login, &repo)?;
        if description.is_none() {
            // Parallel fetch account_info and get_settings_json
            let join_set_info = obelisk::workflow::workflow_support::new_join_set();
            let join_set_settings = obelisk::workflow::workflow_support::new_join_set();

            stargazers::account_obelisk_ext::account::account_info_submit(&join_set_info, &login);
            stargazers::db_obelisk_ext::llm::get_settings_json_submit(&join_set_settings);

            let (_, info_result) =
                stargazers::account_obelisk_ext::account::account_info_await_next(&join_set_info)
                    .map_err(|(_, e)| format!("{:?}", e))?;

            let info = info_result?;

            let (_, settings_result) =
                stargazers::db_obelisk_ext::llm::get_settings_json_await_next(&join_set_settings)
                    .map_err(|(_, e)| format!("{:?}", e))?;

            let settings_json = settings_result?;

            // Generate the user's description.
            let description = llm::respond(&info, &settings_json)?;
            // Persist the generated description.
            db::user::user_update(&login, &description)?;
        }
        Ok(())
    }

    fn star_removed(login: String, repo: String) -> Result<(), String> {
        db::user::unlink(&login, &repo)
    }

    fn backfill(repo: String) -> Result<(), String> {
        let mut cursor = None;
        while let Some(resp) = list_stargazers(&repo, cursor.as_deref())? {
            for login in resp.logins {
                // Submit a child workflow
                imported_workflow::star_added(&login, &repo)?;
            }
            cursor = Some(resp.cursor);
        }
        Ok(())
    }
}
