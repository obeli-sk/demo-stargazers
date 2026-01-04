use crate::generated::exports::stargazers::workflow::workflow::Guest;
use generated::export;
use generated::obelisk::{types::execution::AwaitNextExtensionError, workflow::workflow_support};
use generated::stargazers::{
    db,
    db_obelisk_ext::llm::{get_settings_json_await_next, get_settings_json_submit},
    github,
    github_obelisk_ext::account::{account_info_await_next, account_info_submit},
    llm::llm,
    workflow_obelisk_ext::workflow as imported_workflow_ext,
};

mod generated {
    #![allow(clippy::empty_line_after_outer_attr)]
    include!(concat!(env!("OUT_DIR"), "/any.rs"));
}

struct Component;
export!(Component with_types_in generated);

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
            // Create two join sets for the two child executions.
            let join_set_info = workflow_support::join_set_create_named(&format!("info_{login}"))
                .expect("github login does not contain illegal characters");
            let join_set_settings =
                workflow_support::join_set_create_named(&format!("settings_{login}"))
                    .expect("github login does not contain illegal characters");
            // Submit the two child executions asynchronously.
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
            let got_whole_page = resp.logins.len() == usize::from(page_size);
            for login in resp.logins {
                // This is a direct function call, and will not be intercepted by Obelisk.
                Self::star_added(login, repo.clone())?;
                // To submit a child workflow instead, use:
                // stargazers::workflow::workflow::star_added(&login, &repo)?;
            }
            if !got_whole_page {
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
        // direct call to an activity
        {
            let mut join_set_batch = Vec::new();
            for login in &resp.logins {
                let join_set = workflow_support::join_set_create_named(login)
                    .expect("github login does not contain illegal characters");
                // `-submit`-ting child executions without `-await`-ing results
                imported_workflow_ext::star_added_parallel_submit(&join_set, login, &repo);
                join_set_batch.push(join_set);
            }
            if resp.logins.len() < usize::from(page_size) {
                // last batch gets closed here.
                break;
            }
            cursor = Some(resp.cursor);
            // `join_set_batch` is automatically closed here, as it is removed from the scope.
            // Parent workflow will block until all child workflows finish.
        }
        Ok(())
    }
}

fn err_to_string(err: AwaitNextExtensionError) -> String {
    format!("{err:?}")
}
