use crate::exports::stargazers::workflow::workflow::Guest;
use obelisk::{
    types::execution::{ExecutionError, ExecutionId},
    workflow::workflow_support::{new_join_set_named, ClosingStrategy},
};
use stargazers::{
    db,
    db_obelisk_ext::llm::{get_settings_json_await_next, get_settings_json_submit},
    github,
    github_obelisk_ext::account::{account_info_await_next, account_info_submit},
    llm::llm,
    workflow::workflow as imported_workflow,
    workflow_obelisk_ext::workflow as imported_workflow_ext,
};
use wit_bindgen::generate;

generate!({ generate_all });
struct Component;
export!(Component);

impl Guest for Component {
    fn star_added(login: String, repo: String) -> Result<(), String> {
        // 1. Persist the user giving a star to the project.
        let description = db::user::add_star_get_description(&login, &repo)?;
        if description.is_none() {
            // 2. Fetch the account info from GitHub.
            let info = github::account::account_info(&login)?;
            // 3. Fetch the prompt from the database.
            let settings_json = db::llm::get_settings_json()?;
            // 4. Generate the user's description.
            let description = llm::respond(&info, &settings_json)?;
            // 5. Persist the generated description.
            db::user::update_user_description(&login, &description)?;
        }
        Ok(())
    }

    fn star_added_parallel(login: String, repo: String) -> Result<(), String> {
        // Persist the user giving a star to the project.
        let description = db::user::add_star_get_description(&login, &repo)?;
        if description.is_none() {
            // Create two join sets for the two child workflows.
            let join_set_info =
                new_join_set_named(&format!("info_{login}"), ClosingStrategy::Complete);
            let join_set_settings =
                new_join_set_named(&format!("settings_{login}"), ClosingStrategy::Complete);
            // Submit the two child workflows asynchronously.
            account_info_submit(&join_set_info, &login);
            get_settings_json_submit(&join_set_settings);
            // Await the results.
            let info = account_info_await_next(&join_set_info)
                .map_err(err_to_string)?
                .1?;
            let settings_json = get_settings_json_await_next(&join_set_settings)
                .map_err(err_to_string)?
                .1?;
            // Generate the user's description.
            let description = llm::respond(&info, &settings_json)?;
            // Persist the generated description.
            db::user::update_user_description(&login, &description)?;
        }
        Ok(())
    }

    fn star_removed(login: String, repo: String) -> Result<(), String> {
        db::user::remove_star(&login, &repo)
    }

    fn backfill(repo: String) -> Result<(), String> {
        let page_size = 5;
        let mut cursor = None;
        while let Some(resp) =
            github::account::list_stargazers(&repo, page_size, cursor.as_deref())?
        {
            for login in &resp.logins {
                // Submit a child workflow
                imported_workflow::star_added(login, &repo)?;
            }
            if resp.logins.len() < usize::from(page_size) {
                break;
            }
            cursor = Some(resp.cursor);
        }
        Ok(())
    }

    fn backfill_parallel(repo: String) -> Result<(), String> {
        let page_size = 5;
        let mut cursor = None;
        while let Some(resp) =
            github::account::list_stargazers(&repo, page_size, cursor.as_deref())?
        {
            for login in &resp.logins {
                // No need to await the result of the child workflow.
                // When this execution completes, all join sets will be awaited.
                imported_workflow_ext::star_added_parallel_submit(
                    &new_join_set_named(login, ClosingStrategy::Complete),
                    login,
                    &repo,
                );
            }
            if resp.logins.len() < usize::from(page_size) {
                break;
            }
            cursor = Some(resp.cursor);
        }
        Ok(())
    }
}

fn err_to_string((_, err): (ExecutionId, ExecutionError)) -> String {
    format!("{err:?}")
}
