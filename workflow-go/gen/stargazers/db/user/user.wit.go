// Code generated by wit-bindgen-go. DO NOT EDIT.

// Package user represents the imported interface "stargazers:db/user".
package user

import (
	"go.bytecodealliance.org/cm"
)

// Stargazer represents the record "stargazers:db/user#stargazer".
//
//	record stargazer {
//		login: string,
//		repo: string,
//		description: option<string>,
//	}
type Stargazer struct {
	_           cm.HostLayout     `json:"-"`
	Login       string            `json:"login"`
	Repo        string            `json:"repo"`
	Description cm.Option[string] `json:"description"`
}

// Ordering represents the enum "stargazers:db/user#ordering".
//
//	enum ordering {
//		ascending,
//		descending
//	}
type Ordering uint8

const (
	OrderingAscending Ordering = iota
	OrderingDescending
)

var _OrderingStrings = [2]string{
	"ascending",
	"descending",
}

// String implements [fmt.Stringer], returning the enum case name of e.
func (e Ordering) String() string {
	return _OrderingStrings[e]
}

// MarshalText implements [encoding.TextMarshaler].
func (e Ordering) MarshalText() ([]byte, error) {
	return []byte(e.String()), nil
}

// UnmarshalText implements [encoding.TextUnmarshaler], unmarshaling into an enum
// case. Returns an error if the supplied text is not one of the enum cases.
func (e *Ordering) UnmarshalText(text []byte) error {
	return _OrderingUnmarshalCase(e, text)
}

var _OrderingUnmarshalCase = cm.CaseUnmarshaler[Ordering](_OrderingStrings[:])

// AddStarGetDescription represents the imported function "add-star-get-description".
//
// A user has stared a repo. Persist the user, relation and the repo if needed.
// Returns the user's description if already present.
//
//	add-star-get-description: func(login: string, repo: string) -> result<option<string>,
//	string>
//
//go:nosplit
func AddStarGetDescription(login string, repo string) (result cm.Result[OptionStringShape, cm.Option[string], string]) {
	login0, login1 := cm.LowerString(login)
	repo0, repo1 := cm.LowerString(repo)
	wasmimport_AddStarGetDescription((*uint8)(login0), (uint32)(login1), (*uint8)(repo0), (uint32)(repo1), &result)
	return
}

// RemoveStar represents the imported function "remove-star".
//
// A user has unstarred a repo. Delete the user if there are no other relations.
//
//	remove-star: func(login: string, repo: string) -> result<_, string>
//
//go:nosplit
func RemoveStar(login string, repo string) (result cm.Result[string, struct{}, string]) {
	login0, login1 := cm.LowerString(login)
	repo0, repo1 := cm.LowerString(repo)
	wasmimport_RemoveStar((*uint8)(login0), (uint32)(login1), (*uint8)(repo0), (uint32)(repo1), &result)
	return
}

// UpdateUserDescription represents the imported function "update-user-description".
//
// Update the description of a user.
// User must exist at this point, if not, the operation should fail.
//
//	update-user-description: func(username: string, description: string) -> result<_,
//	string>
//
//go:nosplit
func UpdateUserDescription(username string, description string) (result cm.Result[string, struct{}, string]) {
	username0, username1 := cm.LowerString(username)
	description0, description1 := cm.LowerString(description)
	wasmimport_UpdateUserDescription((*uint8)(username0), (uint32)(username1), (*uint8)(description0), (uint32)(description1), &result)
	return
}

// ListStargazers represents the imported function "list-stargazers".
//
// Return last few stargazers from the database.
//
//	list-stargazers: func(last: u8, repo: option<string>, ordering: ordering) -> result<list<stargazer>,
//	string>
//
//go:nosplit
func ListStargazers(last uint8, repo cm.Option[string], ordering Ordering) (result cm.Result[cm.List[Stargazer], cm.List[Stargazer], string]) {
	last0 := (uint32)(last)
	repo0, repo1, repo2 := lower_OptionString(repo)
	ordering0 := (uint32)(ordering)
	wasmimport_ListStargazers((uint32)(last0), (uint32)(repo0), (*uint8)(repo1), (uint32)(repo2), (uint32)(ordering0), &result)
	return
}
