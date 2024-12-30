use crate::exports::stargazers::workflow::workflow::Guest;
use stargazers::{
    account::account,
    db::{
        prompt,
        user::{self},
    },
    llm::llm,
};
use wit_bindgen::generate;

generate!({ generate_all });
struct Component;
export!(Component);

impl Guest for Component {
    fn star_added(sender: String, repo: String) {
        // Persist the user giving a star to the project.
        user::link(&sender, &repo);
        // Fetch the account info from github.
        let info = account::info(&sender);
        let prompt = prompt::get();
        let prompt = format!("{prompt}\n{info}");
        // Generate the user's description.
        let description = llm::respond(&prompt);
        user::user_update(&sender, &description);
    }

    fn star_removed(sender: String, repo: String) {
        user::unlink(&sender, &repo);
    }
}
