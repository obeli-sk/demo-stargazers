// Code generated by wit-bindgen-go. DO NOT EDIT.

// Package execution represents the imported interface "obelisk:types/execution@1.1.0".
package execution

import (
	"go.bytecodealliance.org/cm"
)

// JoinSetID represents the imported resource "obelisk:types/execution@1.1.0#join-set-id".
//
//	resource join-set-id
type JoinSetID cm.Resource

// ResourceDrop represents the imported resource-drop for resource "join-set-id".
//
// Drops a resource handle.
//
//go:nosplit
func (self JoinSetID) ResourceDrop() {
	self0 := cm.Reinterpret[uint32](self)
	wasmimport_JoinSetIDResourceDrop((uint32)(self0))
	return
}

// ID represents the imported method "id".
//
//	id: func() -> string
//
//go:nosplit
func (self JoinSetID) ID() (result string) {
	self0 := cm.Reinterpret[uint32](self)
	wasmimport_JoinSetIDID((uint32)(self0), &result)
	return
}

// ExecutionID represents the record "obelisk:types/execution@1.1.0#execution-id".
//
//	record execution-id {
//		id: string,
//	}
type ExecutionID struct {
	_  cm.HostLayout `json:"-"`
	ID string        `json:"id"`
}

// DelayID represents the record "obelisk:types/execution@1.1.0#delay-id".
//
//	record delay-id {
//		id: string,
//	}
type DelayID struct {
	_  cm.HostLayout `json:"-"`
	ID string        `json:"id"`
}

// ExecutionError represents the variant "obelisk:types/execution@1.1.0#execution-error".
//
//	variant execution-error {
//		activity-trap(string),
//		permanent-timeout,
//	}
type ExecutionError cm.Variant[uint8, string, string]

// ExecutionErrorActivityTrap returns a [ExecutionError] of case "activity-trap".
func ExecutionErrorActivityTrap(data string) ExecutionError {
	return cm.New[ExecutionError](0, data)
}

// ActivityTrap returns a non-nil *[string] if [ExecutionError] represents the variant case "activity-trap".
func (self *ExecutionError) ActivityTrap() *string {
	return cm.Case[string](self, 0)
}

// ExecutionErrorPermanentTimeout returns a [ExecutionError] of case "permanent-timeout".
func ExecutionErrorPermanentTimeout() ExecutionError {
	var data struct{}
	return cm.New[ExecutionError](1, data)
}

// PermanentTimeout returns true if [ExecutionError] represents the variant case "permanent-timeout".
func (self *ExecutionError) PermanentTimeout() bool {
	return self.Tag() == 1
}

var _ExecutionErrorStrings = [2]string{
	"activity-trap",
	"permanent-timeout",
}

// String implements [fmt.Stringer], returning the variant case name of v.
func (v ExecutionError) String() string {
	return _ExecutionErrorStrings[v.Tag()]
}
