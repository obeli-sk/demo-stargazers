package main

import (
	"fmt"

	"go.bytecodealliance.org/cm"

	logbindings "github.com/obeli-sk/demo-stargazers/workflow-go/gen/obelisk/log/log"
	obeliskWorkflowSupport "github.com/obeli-sk/demo-stargazers/workflow-go/gen/obelisk/workflow/workflow-support"
	stargazersDbObeliskExtLlm "github.com/obeli-sk/demo-stargazers/workflow-go/gen/stargazers/db-obelisk-ext/llm"
	stargazersDbLlm "github.com/obeli-sk/demo-stargazers/workflow-go/gen/stargazers/db/llm"
	stargazersDbUser "github.com/obeli-sk/demo-stargazers/workflow-go/gen/stargazers/db/user"
	stargazersGithubObeliskExtAccount "github.com/obeli-sk/demo-stargazers/workflow-go/gen/stargazers/github-obelisk-ext/account"
	stargazersGithubAccount "github.com/obeli-sk/demo-stargazers/workflow-go/gen/stargazers/github/account"
	stargazersLlmLlm "github.com/obeli-sk/demo-stargazers/workflow-go/gen/stargazers/llm/llm"
	stargazersWorkflowObeliskExtWorkflow "github.com/obeli-sk/demo-stargazers/workflow-go/gen/stargazers/workflow-obelisk-ext/workflow"
	stargazersWorkflowExport "github.com/obeli-sk/demo-stargazers/workflow-go/gen/stargazers/workflow/workflow"
)

// Component implements the logic for the exported workflow functions.
type Component struct{}

// WIT: star-added: func(login: string, repo: string) -> result<_, string>
func (c *Component) StarAdded(login string, repo string) cm.Result[string, struct{}, string] {
	// Imported WIT: stargazers:db/user.add-star-get-description: func(login: string, repo: string) -> result<option<string>, string>
	resDescWrapped := stargazersDbUser.AddStarGetDescription(login, repo)
	if resDescWrapped.IsErr() {
		return cm.Err[cm.Result[string, struct{}, string]](*resDescWrapped.Err())
	}
	// resDescWrapped.OK() returns cm.Option[string]
	// .Some() on cm.Option[string] returns *string (or nil if the option was None)
	descriptionPtr := resDescWrapped.OK().Some()

	if descriptionPtr == nil { // If description was not yet set, generate it.
		// Imported WIT: stargazers:github/account.account-info: func(login: string) -> result<string, string>
		resInfoWrapped := stargazersGithubAccount.AccountInfo(login)
		if resInfoWrapped.IsErr() {
			return cm.Err[cm.Result[string, struct{}, string]](*resInfoWrapped.Err())
		}
		info := *resInfoWrapped.OK()

		// Imported WIT: stargazers:db/llm.get-settings-json: func() -> result<string, string>
		resSettingsWrapped := stargazersDbLlm.GetSettingsJSON()
		if resSettingsWrapped.IsErr() {
			return cm.Err[cm.Result[string, struct{}, string]](*resSettingsWrapped.Err())
		}
		settingsJson := *resSettingsWrapped.OK()

		// Imported WIT: stargazers:llm/llm.respond: func(user-prompt: string, settings-json: string) -> result<string, string>
		resLlmDescWrapped := stargazersLlmLlm.Respond(info, settingsJson)
		if resLlmDescWrapped.IsErr() {
			return cm.Err[cm.Result[string, struct{}, string]](*resLlmDescWrapped.Err())
		}
		llmDescription := *resLlmDescWrapped.OK()

		// Imported WIT: stargazers:db/user.update-user-description: func(username: string, description: string) -> result<_, string>
		resUpdateWrapped := stargazersDbUser.UpdateUserDescription(login, llmDescription)
		if resUpdateWrapped.IsErr() {
			return cm.Err[cm.Result[string, struct{}, string]](*resUpdateWrapped.Err())
		}
	}
	return cm.OK[cm.Result[string, struct{}, string]](struct{}{})
}

// WIT: star-added-parallel: func(login: string, repo: string) -> result<_, string>
func (c *Component) StarAddedParallel(login string, repo string) cm.Result[string, struct{}, string] {
	// Imported WIT: stargazers:db/user.add-star-get-description: func(login: string, repo: string) -> result<option<string>, string>
	resDescWrapped := stargazersDbUser.AddStarGetDescription(login, repo)
	if resDescWrapped.IsErr() {
		return cm.Err[cm.Result[string, struct{}, string]](*resDescWrapped.Err())
	}
	descriptionPtr := resDescWrapped.OK().Some() // .OK() is cm.Option[string], .Some() is *string

	if descriptionPtr == nil { // If description was None
		joinSetInfoName := fmt.Sprintf("info_%s", login)
		// Imported WIT: obelisk:workflow/workflow-support.new-join-set-named: func(name: string, closing-strategy: closing-strategy) -> join-set-id
		joinSetInfoWrapped := obeliskWorkflowSupport.JoinSetCreateNamed(joinSetInfoName)
		joinSetInfo := *joinSetInfoWrapped.OK()

		joinSetSettingsName := fmt.Sprintf("settings_%s", login)
		joinSetSettingsWrapped := obeliskWorkflowSupport.JoinSetCreateNamed(joinSetSettingsName)
		joinSetSettings := *joinSetSettingsWrapped.OK()

		// Imported WIT: stargazers:github-obelisk-ext/account.account-info-submit: func(join-set-id: borrow<join-set-id>, login: string) -> execution-id
		_ = stargazersGithubObeliskExtAccount.AccountInfoSubmit(joinSetInfo, login)
		// Imported WIT: stargazers:db-obelisk-ext/llm.get-settings-json-submit: func(join-set-id: borrow<join-set-id>) -> execution-id
		_ = stargazersDbObeliskExtLlm.GetSettingsJSONSubmit(joinSetSettings)

		// Imported WIT: stargazers:github-obelisk-ext/account.account-info-await-next: func(join-set-id: borrow<join-set-id>) -> result<tuple<execution-id, result<string, string>>, tuple<execution-id, execution-error>>
		awaitInfoWrapped := stargazersGithubObeliskExtAccount.AccountInfoAwaitNext(joinSetInfo)
		if awaitInfoWrapped.IsErr() {
			errPtr := awaitInfoWrapped.Err()
			return cm.Err[cm.Result[string, struct{}, string]](errPtr.String())
		}
		infoTuplePtr := awaitInfoWrapped.OK() // Tuple of (execution-id, function result)
		innerInfoResult := infoTuplePtr.F1    // This is cm.Result[string, string]

		if innerInfoResult.IsErr() {
			return cm.Err[cm.Result[string, struct{}, string]](*innerInfoResult.Err())
		}
		info := *innerInfoResult.OK()

		// Imported WIT: stargazers:db-obelisk-ext/llm.get-settings-json-await-next: func(join-set-id: borrow<join-set-id>) -> result<tuple<execution-id, result<string, string>>, tuple<execution-id, execution-error>>
		awaitSettingsWrapped := stargazersDbObeliskExtLlm.GetSettingsJSONAwaitNext(joinSetSettings)
		if awaitSettingsWrapped.IsErr() {
			errPtr := awaitSettingsWrapped.Err()
			return cm.Err[cm.Result[string, struct{}, string]](errPtr.String())
		}
		settingsTuplePtr := awaitSettingsWrapped.OK() // Tuple of (execution-id, function result)
		innerSettingsResult := settingsTuplePtr.F1    // This is cm.Result[string, string]

		if innerSettingsResult.IsErr() {
			return cm.Err[cm.Result[string, struct{}, string]](*innerSettingsResult.Err())
		}
		settingsJson := *innerSettingsResult.OK()

		// Imported WIT: stargazers:llm/llm.respond: func(user-prompt: string, settings-json: string) -> result<string, string>
		resLlmDescWrapped := stargazersLlmLlm.Respond(info, settingsJson)
		if resLlmDescWrapped.IsErr() {
			return cm.Err[cm.Result[string, struct{}, string]](*resLlmDescWrapped.Err())
		}
		llmDescription := *resLlmDescWrapped.OK()

		// Imported WIT: stargazers:db/user.update-user-description: func(username: string, description: string) -> result<_, string>
		resUpdateWrapped := stargazersDbUser.UpdateUserDescription(login, llmDescription)
		if resUpdateWrapped.IsErr() {
			return cm.Err[cm.Result[string, struct{}, string]](*resUpdateWrapped.Err())
		}
	}
	return cm.OK[cm.Result[string, struct{}, string]](struct{}{})
}

// WIT: star-removed: func(login: string, repo: string) -> result<_, string>
func (c *Component) StarRemoved(login string, repo string) (result cm.Result[string, struct{}, string]) {
	// Imported WIT: stargazers:db/user.remove-star: func(login: string, repo: string) -> result<_, string>
	resRemoveWrapped := stargazersDbUser.RemoveStar(login, repo)

	if resRemoveWrapped.IsErr() {
		return cm.Err[cm.Result[string, struct{}, string]](*resRemoveWrapped.Err())
	}
	return cm.OK[cm.Result[string, struct{}, string]](struct{}{})
}

// WIT: backfill: func(repo: string) -> result<_, string>
func (c *Component) Backfill(repo string) (result cm.Result[string, struct{}, string]) {
	pageSize := uint8(5)
	var cursorOpt cm.Option[string]
	cursorOpt = cm.None[string]()
	for {
		// Imported WIT: stargazers:github/account.list-stargazers: func(repo: string, page-size: u8, cursor: option<string>) -> result<option<stargazers>, string>
		resListWrapped := stargazersGithubAccount.ListStargazers(repo, pageSize, cursorOpt)
		if resListWrapped.IsErr() {
			return cm.Err[cm.Result[string, struct{}, string]](*resListWrapped.Err())
		}
		stargazersCMOption := resListWrapped.OK() // Returns cm.Option[stargazersGithubAccount.Stargazers]
		if stargazersCMOption.None() {
			break // No more stargazers
		}
		resp := *stargazersCMOption.Some()

		gotWholePage := uint(resp.Logins.Len()) == uint(pageSize)
		loginsSlice := resp.Logins.Slice()

		for _, stargazeLogin := range loginsSlice {
			// WIT: star-added: func(login: string, repo: string) -> result<_, string>
			starAddedRes := c.StarAdded(stargazeLogin, repo)

			if starAddedRes.IsErr() {
				// starAddedRes.Err() returns *string from the 3-param result.
				return cm.Err[cm.Result[string, struct{}, string]](*starAddedRes.Err())
			}
		}
		if !gotWholePage {
			break
		}
		newCursorVal := resp.Cursor
		cursorOpt = cm.Some[string](newCursorVal)
	}
	return cm.OK[cm.Result[string, struct{}, string]](struct{}{})
}

// WIT: backfill-parallel: func(repo: string) -> result<_, string>
func (c *Component) BackfillParallel(repo string) (result cm.Result[string, struct{}, string]) {
	pageSize := uint8(5)
	var cursorOpt cm.Option[string]
	cursorOpt = cm.None[string]()
	for {
		// Imported WIT: stargazers:github/account.list-stargazers: func(repo: string, page-size: u8, cursor: option<string>) -> result<option<stargazers>, string>
		resListWrapped := stargazersGithubAccount.ListStargazers(repo, pageSize, cursorOpt)
		if resListWrapped.IsErr() {
			return cm.Err[cm.Result[string, struct{}, string]](*resListWrapped.Err())
		}
		stargazersCMOption := resListWrapped.OK() // Returns cm.Option[stargazersGithubAccount.Stargazers]
		if stargazersCMOption.None() {
			break // No more stargazers
		}
		resp := *stargazersCMOption.Some()

		gotWholePage := uint(resp.Logins.Len()) == uint(pageSize)
		loginsSlice := resp.Logins.Slice()

		joinSetList := []obeliskWorkflowSupport.JoinSet{}

		for _, stargazeLogin := range loginsSlice {
			joinSetName := stargazeLogin
			// Imported WIT: obelisk:workflow/workflow-support.new-join-set-named: func(name: string, closing-strategy: closing-strategy) -> join-set-id
			joinSetForChildWrapped := obeliskWorkflowSupport.JoinSetCreateNamed(joinSetName)
			joinSetForChild := *joinSetForChildWrapped.OK()

			// Imported WIT: stargazers:workflow-obelisk-ext/workflow.star-added-parallel-submit: func(join-set-id: borrow<join-set-id>, login: string, repo: string) -> execution-id
			_ = stargazersWorkflowObeliskExtWorkflow.StarAddedParallelSubmit(joinSetForChild, stargazeLogin, repo)
			joinSetList = append(joinSetList, joinSetForChild)
		}
		for _, joinSet := range joinSetList {
			obeliskWorkflowSupport.JoinSetClose(joinSet)
		}
		if !gotWholePage {
			break
		}
		newCursorVal := resp.Cursor
		cursorOpt = cm.Some[string](newCursorVal)
	}
	return cm.OK[cm.Result[string, struct{}, string]](struct{}{})
}

func init() {
	componentInstance := &Component{}

	stargazersWorkflowExport.Exports.StarAdded = componentInstance.StarAdded
	stargazersWorkflowExport.Exports.StarAddedParallel = componentInstance.StarAddedParallel
	stargazersWorkflowExport.Exports.StarRemoved = componentInstance.StarRemoved
	stargazersWorkflowExport.Exports.Backfill = componentInstance.Backfill
	stargazersWorkflowExport.Exports.BackfillParallel = componentInstance.BackfillParallel
}

func main() {
	logbindings.Error("wasi:cli/run@0.2.0.run is exported only because of current TinyGo limitation")
	panic("wasi:cli/run@0.2.0.run is exported only because of current TinyGo limitation")
}
