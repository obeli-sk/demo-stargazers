# Stargazers

A sample [obelisk](https://github.com/obeli-sk/obelisk) workflow
that monitors a project stargazers using a webhook.
When a user stars the project, a webhook event is received.

On a *star added* event a new workflow execution is submitted.
First, an activity persists the GitHub username, then, a background check is started.
The background check is just a GitHub request getting
basic info on the user, and then an activity is called that
transforms the info into a summary using an external LLM service.
The summary is then persisted.

On a *star deleted* event an activity will delete the specific row from the database.
