// import { starAdded as starAddedImported } from 'stargazers:workflow/workflow'; // search for `starAddedImported`

import { starAddedParallelSubmit } from 'stargazers:workflow-obelisk-ext/workflow';
import { accountInfo, listStargazers } from 'stargazers:github/account';
import { accountInfoSubmit, accountInfoAwaitNext } from 'stargazers:github-obelisk-ext/account';
import { getSettingsJson } from 'stargazers:db/llm';
import { getSettingsJsonSubmit, getSettingsJsonAwaitNext } from 'stargazers:db-obelisk-ext/llm';
import { addStarGetDescription, updateUserDescription, removeStar } from 'stargazers:db/user';
import { respond as llmRespond } from 'stargazers:llm/llm';
// Obelisk host utilities for workflows
import { joinSetCreateNamed, joinSetClose } from 'obelisk:workflow/workflow-support@4.0.0';
import { debug as log_debug, info as log_info, error as log_error } from 'obelisk:log/log@1.0.0'

console.log = function (...args) {
  const message = args.join(' ');
  log_info(message);
}
console.debug = function (...args) {
  const message = args.join(' ');
  log_debug(message);
}
console.error = function (...args) {
  const message = args.join(' ');
  log_error(message);
}

function stringify_error(e) {
  let errorDetails;
  if (e instanceof Error) {
    errorDetails = `${e.name}: ${e.message}\n${e.stack}`;
  } else if (typeof e === 'string') {
    errorDetails = e;
  } else {
    try {
      errorDetails = `Non-Error thrown: JSON: ${JSON.stringify(e)}, String: ${e}`;
    } catch {
      errorDetails = `Non-Error thrown: ${e}`;
    }
  }
  return errorDetails;
}

function unwrap(obj) {
  if (obj.tag === 'ok') {
    return obj.val;
  } else {
    throw info.val;
  }

}

export const workflow = {
  /**
   * Implements stargazers:workflow/workflow.star_added
   */
  starAdded(login, repo) {
    try {
      // 1. Persist the user giving a star to the project. Check if description exists.
      // WIT: add-star-get-description: func(login: string, repo: string) -> result<option<string>, string>
      const existingDescription = addStarGetDescription(login, repo);

      if (existingDescription === null || existingDescription === undefined) {
        console.log(`No description for ${login} on ${repo}, generating...`);

        // 2. Fetch the account info from GitHub.
        // WIT: account-info: func(login: string) -> result<string, string>
        const info = accountInfo(login);

        // 3. Fetch the prompt settings from the database.
        // WIT: get-settings-json: func() -> result<string, string>
        const settingsJson = getSettingsJson();

        // 4. Generate the user's description using the LLM.
        // WIT: respond: func(user-prompt: string, settings-json: string) -> result<string, string>
        const description = llmRespond(info, settingsJson);

        // 5. Persist the generated description.
        // WIT: update-user-description: func(username: string, description: string) -> result<_, string>
        updateUserDescription(login, description);
        console.log(`Generated and saved description for ${login}`);
      }
    } catch (error) {
      throw stringify_error(error);
    }
  },

  /**
   * Implements stargazers:workflow/workflow.star_added_parallel
   */
  starAddedParallel(login, repo) {
    try {

      // Persist the user giving a star to the project. Check if description exists.
      const existingDescription = addStarGetDescription(login, repo);

      if (existingDescription === null || existingDescription === undefined) {
        console.log(`No description for ${login} on ${repo}, generating in parallel...`);
        // Create two join sets for the two child executions (async operations).

        // WIT: join-set-create-named: func(name: string) -> result<join-set, join-set-create-error>;
        const joinSetInfo = joinSetCreateNamed(`info_${login}`);
        const joinSetSettings = joinSetCreateNamed(`settings_${login}`);

        // Submit the two child executions asynchronously using Obelisk extensions.
        // WIT: account-info-submit: func(join-set-id: borrow<join-set-id>, login: string) -> execution-id
        accountInfoSubmit(joinSetInfo, login);

        // WIT: get-settings-json-submit: func(join-set-id: borrow<join-set-id>) -> execution-id
        getSettingsJsonSubmit(joinSetSettings);

        // WIT: account-info-await-next: func(join-set-id: borrow<join-set-id>) -> result<tuple<execution-id, result<string, string>>, tuple<execution-id, execution-error>>
        let [_execId, info] = accountInfoAwaitNext(joinSetInfo);
        console.debug("Got info", JSON.stringify(info));
        info = unwrap(info);

        // WIT: get-settings-json-await-next: func(join-set-id: borrow<join-set-id>) -> result<tuple<execution-id, result<string, string>>, tuple<execution-id, execution-error>>
        let [_execId2, settingsJson] = getSettingsJsonAwaitNext(joinSetSettings);
        console.debug("Got settingsJson", JSON.stringify(settingsJson));
        settingsJson = unwrap(settingsJson);

        // Generate the user's description.
        const description = llmRespond(info, settingsJson);

        // Persist the generated description.
        updateUserDescription(login, description);
        console.log(`Generated and saved description in parallel for ${login}`);
      } else {
        console.log(`Description already exists for ${login} on ${repo}.`);
      }
    } catch (error) {
      throw stringify_error(error);
    }
  },

  /**
   * Implements stargazers:workflow/workflow.star_removed
   */
  starRemoved(login, repo) {
    try {
      // WIT: remove-star: func(login: string, repo: string) -> result<_, string>
      removeStar(login, repo);
      console.log(`Removed star for ${login} on ${repo}.`);
    } catch (error) {
      throw stringify_error(error);
    }
  },

  /**
   * Implements stargazers:workflow/workflow.backfill
   */
  backfill(repo) {
    try {
      console.log(`Starting backfill for ${repo}...`);
      const pageSize = 5;
      let cursor = null;

      while (true) {
        // WIT: list-stargazers: func(repo: string, page-size: u8, cursor: option<string>) -> result<option<stargazers>, string>;
        // WIT: record stargazers { logins: list<string>, cursor: string, }
        const resp = listStargazers(repo, pageSize, cursor);
        if (!resp) {
          console.log("No more stargazers found.");
          break;
        }
        const gotWholePage = resp.logins.length === pageSize;
        console.log(`Found ${resp.logins.length} stargazers (page size ${pageSize}).`);
        for (const login of resp.logins) {
          // Direct call to this component's starAdded function.
          console.log(`Processing ${login}...`);
          workflow.starAdded(login, repo);
          // Note: To submit a child execution use
          // starAddedImported(login, repo); // Calling the imported function
        }
        if (!gotWholePage) {
          console.log("Reached last page.");
          break;
        }
        cursor = resp.cursor;
        console.log(`Moving to next page with cursor ${cursor}`);
      }
      console.log(`Backfill for ${repo} completed.`);
    } catch (error) {
      throw stringify_error(error);
    }
  },

  /**
   * Implements stargazers:workflow/workflow.backfill_parallel
   */
  backfillParallel(repo) {
    try {
      console.log(`Starting parallel backfill for ${repo}...`);
      const pageSize = 5;
      let cursor = null; // WIT option<string> starts as null

      while (true) {
        // WIT: list-stargazers: func(...) -> result<option<stargazers>, string>;
        const resp = listStargazers(repo, pageSize, cursor);

        // Check if option<stargazers> was None
        if (!resp) {
          console.debug(`No more stargazers found.`);
          break;
        }

        // If resp exists, it's the stargazers object: { logins: [...], cursor: "..." }
        const gotWholePage = resp.logins.length === pageSize;
        console.log(`Found ${resp.logins.length} stargazers.`);
        let joinSetList = [];

        for (const login of resp.logins) {
          // Submit star_added_parallel as a child workflow using the Obelisk extension import.
          // No need to await; the engine handles join sets completion.
          console.log(`Submitting task for ${login}...`);
          // WIT: star-added-parallel-submit: func(join-set-id: borrow<join-set-id>, login: string, repo: string) -> execution-id
          let joinSet = joinSetCreateNamed(login);
          starAddedParallelSubmit(
            joinSet, // Create a join set per user
            login,
            repo
          );
          joinSetList.push(joinSet);
        }
        // Close all join sets of this batch for back-pressure.
        for (let joinSet of joinSetList) {
          // WIT: join-set-close: func(self: join-set);
          joinSetClose(joinSet);
        }

        if (!gotWholePage) {
          console.debug('Reached last page.');
          break;
        }
        cursor = resp.cursor;
        console.log(`Moving to next page with cursor ${cursor}`);
      }
      // Log completion of the submission loop. Actual task completion happens asynchronously, Obelisk runtime will wait until
      // all join sets are completed.
      console.debug(`Completed submission of tasks.`);
    } catch (error) {
      throw stringify_error(error);
    }
  }
};
