// Code generated by wit-bindgen-go. DO NOT EDIT.

package user

import (
	"go.bytecodealliance.org/cm"
)

// This file contains wasmimport and wasmexport declarations for "stargazers:db".

//go:wasmimport stargazers:db/user add-star-get-description
//go:noescape
func wasmimport_AddStarGetDescription(login0 *uint8, login1 uint32, repo0 *uint8, repo1 uint32, result *cm.Result[OptionStringShape, cm.Option[string], string])

//go:wasmimport stargazers:db/user remove-star
//go:noescape
func wasmimport_RemoveStar(login0 *uint8, login1 uint32, repo0 *uint8, repo1 uint32, result *cm.Result[string, struct{}, string])

//go:wasmimport stargazers:db/user update-user-description
//go:noescape
func wasmimport_UpdateUserDescription(username0 *uint8, username1 uint32, description0 *uint8, description1 uint32, result *cm.Result[string, struct{}, string])

//go:wasmimport stargazers:db/user list-stargazers
//go:noescape
func wasmimport_ListStargazers(last0 uint32, repo0 uint32, repo1 *uint8, repo2 uint32, ordering0 uint32, result *cm.Result[cm.List[Stargazer], cm.List[Stargazer], string])
